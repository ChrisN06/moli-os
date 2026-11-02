//! Legacy miIO framing. Tokens never enter Debug, errors or logs.
use aes::{
    Aes128,
    cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray},
};
use anyhow::{Context as _, anyhow, ensure};
use md5::{Digest, Md5};
use serde_json::{Value, json};
use std::{
    net::Ipv4Addr,
    time::{Duration, Instant},
};
use tokio::{net::UdpSocket, time::timeout};
const WAIT: Duration = Duration::from_secs(4);
pub struct Miio {
    socket: UdpSocket,
    token: [u8; 16],
    device: [u8; 4],
    time: u32,
    opened: Instant,
    id: u32,
}
pub fn token(text: &str) -> anyhow::Result<[u8; 16]> {
    ensure!(
        text.len() == 32 && text.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid miIO token"
    );
    let mut out = [0; 16];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16)?;
    }
    Ok(out)
}
fn crypt(token: &[u8; 16], input: &[u8], decrypt: bool) -> anyhow::Result<Vec<u8>> {
    let key = Md5::digest(token);
    let iv = Md5::digest([key.as_slice(), token].concat());
    let cipher = Aes128::new(GenericArray::from_slice(&key));
    let mut previous: [u8; 16] = iv.into();
    let mut out = input.to_vec();
    if decrypt {
        ensure!(
            !out.is_empty() && out.len().is_multiple_of(16),
            "invalid miIO payload"
        );
    } else {
        let pad = 16 - out.len() % 16;
        out.extend(std::iter::repeat_n(u8::try_from(pad)?, pad));
    }
    for block in out.as_chunks_mut::<16>().0 {
        if decrypt {
            let encrypted: [u8; 16] = *block;
            cipher.decrypt_block(GenericArray::from_mut_slice(block));
            for (x, y) in block.iter_mut().zip(previous) {
                *x ^= y;
            }
            previous = encrypted;
        } else {
            for (x, y) in block.iter_mut().zip(previous) {
                *x ^= y;
            }
            cipher.encrypt_block(GenericArray::from_mut_slice(block));
            previous.copy_from_slice(block);
        }
    }
    if decrypt {
        let pad = usize::from(*out.last().ok_or_else(|| anyhow!("invalid padding"))?);
        ensure!(
            (1..=16).contains(&pad)
                && out[out.len() - pad..]
                    .iter()
                    .all(|b| usize::from(*b) == pad),
            "invalid miIO padding"
        );
        out.truncate(out.len() - pad);
    }
    Ok(out)
}
fn packet(token: &[u8; 16], device: [u8; 4], time: u32, payload: &[u8]) -> anyhow::Result<Vec<u8>> {
    let encrypted = crypt(token, payload, false)?;
    let length = u16::try_from(32 + encrypted.len())?;
    let mut p = vec![0x21, 0x31];
    p.extend(length.to_be_bytes());
    p.extend([0; 4]);
    p.extend(device);
    p.extend(time.to_be_bytes());
    let checksum = Md5::digest([p.as_slice(), token, &encrypted].concat());
    p.extend(checksum);
    p.extend(encrypted);
    Ok(p)
}
fn unpack(token: &[u8; 16], device: [u8; 4], p: &[u8], id: u32) -> anyhow::Result<Value> {
    ensure!(
        p.len() > 32 && p[..2] == [0x21, 0x31] && p[8..12] == device,
        "invalid miIO reply"
    );
    ensure!(
        usize::from(u16::from_be_bytes([p[2], p[3]])) == p.len(),
        "invalid miIO length"
    );
    let expected = Md5::digest([&p[..16], token, &p[32..]].concat());
    ensure!(
        expected
            .iter()
            .zip(&p[16..32])
            .fold(0u8, |x, (a, b)| x | (a ^ b))
            == 0,
        "miIO checksum mismatch"
    );
    let mut plain = crypt(token, &p[32..], true)?;
    while plain.last() == Some(&0) {
        plain.pop();
    }
    let v: Value = serde_json::from_slice(&plain).map_err(|_| anyhow!("invalid miIO JSON"))?;
    ensure!(
        v["id"].as_u64() == Some(u64::from(id)),
        "miIO sequence mismatch"
    );
    ensure!(v.get("error").is_none(), "miIO command refused");
    Ok(v["result"].clone())
}
impl Miio {
    pub async fn connect(host: Ipv4Addr, port: u16, text: &str) -> anyhow::Result<Self> {
        ensure!(
            (host.is_private() || host.is_loopback()) && port > 0,
            "miIO requires a local unicast target"
        );
        let token = token(text)?;
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
        socket.connect((host, port)).await?;
        let mut hello = [0xff; 32];
        hello[..4].copy_from_slice(&[0x21, 0x31, 0, 32]);
        socket.send(&hello).await?;
        // Only the header: discovery payloads must never become a token extraction path.
        let mut header = [0; 16];
        let n = timeout(WAIT, socket.recv(&mut header))
            .await
            .context("miIO discovery timed out")??;
        ensure!(
            n == 16 && header[..4] == [0x21, 0x31, 0, 32],
            "miIO hello failed"
        );
        Ok(Self {
            socket,
            token,
            device: header[8..12].try_into()?,
            time: u32::from_be_bytes(header[12..16].try_into()?),
            opened: Instant::now(),
            id: 0,
        })
    }
    pub async fn call(&mut self, method: &str, params: Value) -> anyhow::Result<Value> {
        self.id = self
            .id
            .checked_add(1)
            .ok_or_else(|| anyhow!("miIO sequence exhausted"))?;
        let mut data = serde_json::to_vec(&json!({"id":self.id,"method":method,"params":params}))?;
        data.push(0);
        let now = self
            .time
            .saturating_add(u32::try_from(self.opened.elapsed().as_secs()).unwrap_or(u32::MAX))
            // Legacy miIO expects the device clock advanced by one second.
            .saturating_add(1);
        self.socket
            .send(&packet(&self.token, self.device, now, &data)?)
            .await?;
        let mut reply = vec![0; 65536];
        let n = timeout(WAIT, self.socket.recv(&mut reply))
            .await
            .context("miIO authenticated reply timed out")??;
        let result = unpack(&self.token, self.device, &reply[..n], self.id)?;
        self.time = u32::from_be_bytes(reply[12..16].try_into()?);
        self.opened = Instant::now();
        Ok(result)
    }
}
impl std::fmt::Debug for Miio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Miio { [redacted] }")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_openssl_aes_cbc_and_md5_public_fixture() {
        // Independently generated with OpenSSL AES-128-CBC, PKCS#7 and Python hashlib.
        let key = token("000102030405060708090a0b0c0d0e0f").unwrap();
        let raw = "2131004000000000010203040000007b7098c3b6b8b89556f6262ece6e2b6e2be1870988f4616f875fbcbfc737c856e6dd5e5305e8ad2f544a755856b84c5db0";
        let expected: Vec<u8> = raw
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
            .collect();
        let plain = b"{\"id\":1,\"result\":[\"ok\"]}\0";
        assert_eq!(packet(&key, [1, 2, 3, 4], 123, plain).unwrap(), expected);
        assert_eq!(
            unpack(&key, [1, 2, 3, 4], &expected, 1).unwrap(),
            json!(["ok"])
        );
    }
    #[tokio::test]
    async fn local_udp_handshake_and_read_only_queries() {
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let port = socket.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let mut bytes = [0; 4096];
            let (n, peer) = socket.recv_from(&mut bytes).await.unwrap();
            assert_eq!(n, 32);
            let mut hello = [0; 32];
            hello[..4].copy_from_slice(&[0x21, 0x31, 0, 32]);
            hello[8..12].copy_from_slice(&[1, 2, 3, 4]);
            hello[12..16].copy_from_slice(&123u32.to_be_bytes());
            socket.send_to(&hello, peer).await.unwrap();
            let mut methods = Vec::new();
            for (expected_time, reply_time) in [(124u32, 130u32), (131, 140)] {
                let (n, peer) = socket.recv_from(&mut bytes).await.unwrap();
                assert_eq!(&bytes[12..16], &expected_time.to_be_bytes());
                let plain = crypt(&[7; 16], &bytes[32..n], true).unwrap();
                assert_eq!(plain.last(), Some(&0));
                let query: Value = serde_json::from_slice(&plain[..plain.len() - 1]).unwrap();
                methods.push(query["method"].as_str().unwrap().to_owned());
                let response =
                    serde_json::to_vec(&json!({"id":query["id"],"result":[{"state":8}]})).unwrap();
                socket
                    .send_to(
                        &packet(&[7; 16], [1, 2, 3, 4], reply_time, &response).unwrap(),
                        peer,
                    )
                    .await
                    .unwrap();
            }
            methods
        });
        let mut client = Miio::connect(std::net::Ipv4Addr::LOCALHOST, port, &"07".repeat(16))
            .await
            .unwrap();
        assert_eq!(
            client.call("miIO.info", json!([])).await.unwrap()[0]["state"],
            8
        );
        assert_eq!(
            client.call("get_status", json!([])).await.unwrap()[0]["state"],
            8
        );
        assert_eq!(server.await.unwrap(), ["miIO.info", "get_status"]);
    }
    #[test]
    fn framing_authentication_sequence_and_padding() {
        let t = [7; 16];
        let device = [1, 2, 3, 4];
        let p = packet(&t, device, 123, br#"{"id":1,"result":["ok"]}"#).unwrap();
        assert_eq!(unpack(&t, device, &p, 1).unwrap(), json!(["ok"]));
        assert!(unpack(&t, device, &p, 2).is_err());
        let mut corrupt = p;
        corrupt[35] ^= 1;
        assert!(unpack(&t, device, &corrupt, 1).is_err());
        assert!(token("bad").is_err());
        assert!(crypt(&t, &[0; 16], true).is_err());
    }
}
