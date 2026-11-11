//! Cloud session used only for discovery/map metadata. Fixed Xiaomi TLS hosts;
//! upstream bodies, cookies and ticket URLs never enter errors or logs.
use anyhow::{anyhow, ensure};
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use http::{HeaderMap, Method, Request, Uri};
use http_body_util::Full;
use moli_net::Body;
use ring::{
    digest,
    rand::{SecureRandom, SystemRandom},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, time::Duration};
const WAIT: Duration = Duration::from_secs(15);
pub fn encode(s: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            out.push(char::from(b));
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}
pub fn form(values: &BTreeMap<&str, String>) -> String {
    values
        .iter()
        .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}
async fn request(
    host: &str,
    path: &str,
    method: Method,
    cookie: &str,
    body: String,
) -> anyhow::Result<(http::StatusCode, HeaderMap, Body)> {
    let req = Request::builder()
        .method(method)
        .uri(path)
        .header("Cookie", cookie)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header(
            "User-Agent",
            "Android-7.1.1-1.0.0-ONEPLUS A3010-136-2EFD19A99C52A6B9",
        )
        .header("MIOT-ENCRYPT-ALGORITHM", "ENCRYPT-RC4")
        .body(Full::new(Body::from(body)))?;
    moli_net::web_with_headers(host, 443, true, req, WAIT, 2 * 1024 * 1024)
        .await
        .map_err(|_| anyhow!("Xiaomi cloud unavailable"))
}
fn login_json(body: &[u8]) -> anyhow::Result<Value> {
    let text = std::str::from_utf8(body).map_err(|_| anyhow!("invalid Xiaomi session"))?;
    serde_json::from_str(
        text.strip_prefix("&&&START&&&")
            .ok_or_else(|| anyhow!("invalid Xiaomi session"))?,
    )
    .map_err(|_| anyhow!("invalid Xiaomi session"))
}
fn allowed_ticket(uri: &Uri) -> bool {
    uri.scheme_str() == Some("https")
        && uri.port_u16().is_none_or(|p| p == 443)
        && uri.host().is_some_and(|h| {
            h == "account.xiaomi.com" || h == "api.io.mi.com" || h.ends_with(".api.io.mi.com")
        })
}
fn supported_call(path: &str, data: &Value) -> bool {
    matches!(
        path,
        "/v2/home/device_list_page" | "/home/device_list" | "/home/getmapfileurl"
    ) || (path.strip_prefix("/home/rpc/").is_some_and(|device| {
        !device.is_empty() && device.len() <= 32 && device.bytes().all(|b| b.is_ascii_digit())
    }) && data["id"].as_u64().is_some_and(|id| {
        (1..=9999).contains(&id) && *data == json!({"id":id,"method":"get_map_v1","params":[]})
    }))
}
pub struct Cloud {
    cookies: String,
    security: Vec<u8>,
    host: &'static str,
}
impl Cloud {
    pub async fn login(user: &str, pass: &str, region: &str) -> anyhow::Result<Self> {
        ensure!(
            !user.is_empty() && user.bytes().all(|b| b.is_ascii_digit()) && valid_token(pass),
            "invalid Xiaomi account token"
        );
        let (_, _, body) = request(
            "account.xiaomi.com",
            "/pass/serviceLogin?_json=true&sid=xiaomiio",
            Method::GET,
            &format!("userId={user}; passToken={pass}"),
            String::new(),
        )
        .await?;
        let login = login_json(&body)?;
        let security = B64
            .decode(
                login["ssecurity"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Xiaomi session expired"))?,
            )
            .map_err(|_| anyhow!("invalid Xiaomi session"))?;
        ensure!(
            security.len() == 16 || security.len() == 32,
            "invalid Xiaomi session key"
        );
        let mut location = login["location"]
            .as_str()
            .ok_or_else(|| anyhow!("Xiaomi session expired"))?
            .to_owned();
        let mut cookies = BTreeMap::new();
        cookies.insert("userId".to_owned(), user.to_owned());
        for _ in 0..6 {
            let uri: Uri = location
                .parse()
                .map_err(|_| anyhow!("invalid Xiaomi redirect"))?;
            ensure!(allowed_ticket(&uri), "untrusted Xiaomi redirect");
            let raw_cookie = cookies
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("; ");
            let (status, headers, _) = request(
                uri.host().unwrap_or_default(),
                uri.path_and_query()
                    .map_or("/", http::uri::PathAndQuery::as_str),
                Method::GET,
                &raw_cookie,
                String::new(),
            )
            .await?;
            for h in headers.get_all("set-cookie") {
                if let Ok(text) = h.to_str()
                    && let Some((k, v)) = text.split(';').next().and_then(|s| s.split_once('='))
                    && matches!(k, "userId" | "cUserId" | "serviceToken")
                {
                    cookies.insert(k.to_owned(), v.to_owned());
                }
            }
            if !status.is_redirection() {
                break;
            }
            headers
                .get("location")
                .and_then(|s| s.to_str().ok())
                .ok_or_else(|| anyhow!("invalid Xiaomi redirect"))?
                .clone_into(&mut location);
        }
        ensure!(
            cookies.contains_key("serviceToken"),
            "Xiaomi session expired"
        );
        let cookies = cookies
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("; ");
        let host = match region {
            "de" => "de.api.io.mi.com",
            "cn" => "api.io.mi.com",
            _ => return Err(anyhow!("unsupported Xiaomi region")),
        };
        Ok(Self {
            cookies,
            security,
            host,
        })
    }
    pub async fn call(&self, path: &str, data: Value) -> anyhow::Result<Value> {
        ensure!(
            supported_call(path, &data),
            "unsupported Xiaomi cloud operation"
        );
        let mut nonce = [0; 12];
        SystemRandom::new()
            .fill(&mut nonce[..8])
            .map_err(|_| anyhow!("randomness unavailable"))?;
        let minutes = moli_core::now_ms() / 60_000;
        nonce[8..].copy_from_slice(&u32::try_from(minutes)?.to_be_bytes());
        let key = digest::digest(
            &digest::SHA256,
            &[self.security.as_slice(), &nonce].concat(),
        );
        let data = data.to_string();
        let hash = signature(path, &data, None, key.as_ref());
        let encrypted = B64.encode(rc4(key.as_ref(), data.as_bytes()));
        let encrypted_hash = B64.encode(rc4(key.as_ref(), hash.as_bytes()));
        let sig = signature(path, &encrypted, Some(&encrypted_hash), key.as_ref());
        let params = BTreeMap::from([
            ("data", encrypted),
            ("rc4_hash__", encrypted_hash),
            ("signature", sig),
            ("_nonce", B64.encode(nonce)),
        ]);
        let (status, _, body) = request(
            self.host,
            &format!("/app{path}"),
            Method::POST,
            &self.cookies,
            form(&params),
        )
        .await?;
        ensure!(status.is_success(), "Xiaomi cloud request refused");
        let encrypted = B64
            .decode(body.trim_ascii())
            .map_err(|_| anyhow!("invalid Xiaomi cloud reply"))?;
        let v: Value = serde_json::from_slice(&rc4(key.as_ref(), &encrypted))
            .map_err(|_| anyhow!("invalid Xiaomi cloud reply"))?;
        ensure!(
            v["code"].as_i64() == Some(0),
            "Xiaomi cloud request refused"
        );
        Ok(v["result"].clone())
    }
    pub async fn devices(&self) -> anyhow::Result<Vec<Value>> {
        let v = self
            .call(
                "/v2/home/device_list_page",
                json!({"limit":200,"get_split_device":true}),
            )
            .await?;
        ensure!(
            !v["has_more"].as_bool().unwrap_or(false),
            "Xiaomi discovery requires pagination"
        );
        let mut devices = v["list"]
            .as_array()
            .ok_or_else(|| anyhow!("invalid Xiaomi device list"))?
            .clone();
        if devices.iter().any(|d| {
            d["model"] == "rockrobo.vacuum.v1"
                && d["token"].as_str().is_none_or(|token| {
                    crate::miio::token(token).is_err() || token.bytes().all(|b| b == b'0')
                })
        }) {
            let old = self
                .call(
                    "/home/device_list",
                    json!({"getVirtualModel":false,"getHuamiDevices":0}),
                )
                .await?;
            if let Some(list) = old["list"].as_array() {
                for device in &mut devices {
                    if let Some(other) = list.iter().find(|d| d["did"] == device["did"]) {
                        *device = other.clone();
                    }
                }
            }
        }
        Ok(devices)
    }
}
fn signature(path: &str, data: &str, hash: Option<&str>, key: &[u8]) -> String {
    let mut text = format!("POST&{path}&data={data}");
    if let Some(hash) = hash {
        text.push_str("&rc4_hash__=");
        text.push_str(hash);
    }
    text.push('&');
    text.push_str(&B64.encode(key));
    B64.encode(digest::digest(&digest::SHA1_FOR_LEGACY_USE_ONLY, text.as_bytes()).as_ref())
}
fn rc4(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut s = [0u8; 256];
    for (i, b) in s.iter_mut().enumerate() {
        *b = u8::try_from(i).unwrap_or_default();
    }
    let mut j = 0usize;
    for i in 0..256 {
        j = (j + usize::from(s[i]) + usize::from(key[i % key.len()])) % 256;
        s.swap(i, j);
    }
    let (mut i, mut j) = (0usize, 0usize);
    let mut out = Vec::with_capacity(data.len());
    for n in 0..1024 + data.len() {
        i = (i + 1) % 256;
        j = (j + usize::from(s[i])) % 256;
        s.swap(i, j);
        let k = s[(usize::from(s[i]) + usize::from(s[j])) % 256];
        if n >= 1024 {
            out.push(data[n - 1024] ^ k);
        }
    }
    out
}
fn valid_token(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 4096
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-.:/+=".contains(&b))
}
/// Only go2rtc's scalar token field, not a general YAML parser. Fail closed on
/// aliases, folded values and escapes; support a structured parser if its format changes.
pub fn pass_token(config: &str, user: &str) -> Option<String> {
    let mut inside = false;
    for line in config.lines() {
        if line.trim() == "xiaomi:" {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if !line.is_empty() && !line.starts_with(char::is_whitespace) {
            break;
        }
        if let Some((key, value)) = line.trim().split_once(':')
            && key.trim_matches(['\'', '"']) == user
        {
            let value = value.trim().trim_matches(['\'', '"']);
            return valid_token(value).then(|| value.to_owned());
        }
    }
    None
}
impl std::fmt::Debug for Cloud {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Cloud { [redacted] }")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn token_scope_and_redirects_are_bounded() {
        assert_eq!(
            pass_token(
                "api:\n  listen: ':1984'\nxiaomi:\n  \"123\": dummy-token\n",
                "123"
            ),
            Some("dummy-token".into())
        );
        assert!(pass_token("other:\n  '123': stolen\n", "123").is_none());
        assert!(pass_token("xiaomi:\n  '123': *alias\n", "123").is_none());
        assert!(allowed_ticket(
            &"https://sts.api.io.mi.com/sts?ticket=fictive"
                .parse()
                .unwrap()
        ));
        assert!(!allowed_ticket(
            &"https://api.io.mi.com.attacker.invalid/".parse().unwrap()
        ));
        assert!(!allowed_ticket(
            &"http://account.xiaomi.com/".parse().unwrap()
        ));
    }
    #[test]
    fn rc4_drop_matches_independent_fixture() {
        // RFC 6229, 40-bit key, offset 1024.
        assert_eq!(
            rc4(&[1, 2, 3, 4, 5], &[0; 16]),
            [
                0x30, 0xab, 0xbc, 0xc7, 0xc2, 0x0b, 0x01, 0x60, 0x9f, 0x23, 0xee, 0x2d, 0x5f, 0x6b,
                0xb7, 0xdf
            ]
        );
    }
}

#[cfg(test)]
mod map_rpc_tests {
    use super::*;
    #[test]
    fn cloud_rpc_allows_only_a_bounded_read_only_map_request() {
        let map = json!({"id":1,"method":"get_map_v1","params":[]});
        assert!(supported_call("/home/rpc/12345678", &map));
        for device in [
            "",
            "../123",
            "1?other=2",
            "robot",
            "123456789012345678901234567890123",
        ] {
            assert!(!supported_call(&format!("/home/rpc/{device}"), &map));
        }
        for id in [0, 10000] {
            assert!(!supported_call(
                "/home/rpc/12345678",
                &json!({"id":id,"method":"get_map_v1","params":[]})
            ));
        }
        assert!(supported_call(
            "/home/rpc/12345678",
            &json!({"id":9999,"method":"get_map_v1","params":[]})
        ));
        for method in ["app_start", "app_charge", "app_zoned_clean", "get_status"] {
            assert!(!supported_call(
                "/home/rpc/12345678",
                &json!({"id":1,"method":method,"params":[]})
            ));
        }
        assert!(!supported_call(
            "/home/rpc/12345678",
            &json!({"id":1,"method":"get_map_v1","params":[1]})
        ));
    }
}
