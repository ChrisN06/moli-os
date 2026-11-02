//! Bounded RR-map decoding. Map URLs and cloud keys stay server-side.
use crate::{Account, Config, Zone, miio::Miio};
use anyhow::{anyhow, ensure};
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use http_body_util::Full;
use serde::Serialize;
use serde_json::json;
use std::io::Read;
#[derive(Clone, Debug, Serialize)]
pub struct Map {
    pub index: u32,
    pub frame: String,
    /// Millimetres in the vacuum reference frame; image dimensions are pixels.
    pub left: u32,
    pub bottom: u32,
    pub width: u32,
    pub height: u32,
    pub dock: [u16; 2],
    pub robot: Option<[u16; 2]>,
    pub pixels: String,
}
impl Map {
    pub fn matches(&self, z: &Zone) -> bool {
        z.valid()
            && self.index == z.map_index
            && self.frame == z.frame
            && self
                .dock
                .iter()
                .zip(z.dock)
                .all(|(a, b)| a.abs_diff(b) <= 100)
            && u32::from(z.bounds[0]) >= self.left
            && u32::from(z.bounds[1]) >= self.bottom
            && u32::from(z.bounds[2]) <= self.left + self.width * 50
            && u32::from(z.bounds[3]) <= self.bottom + self.height * 50
    }
}
fn u16_at(b: &[u8], i: usize) -> anyhow::Result<u16> {
    Ok(u16::from_le_bytes(
        b.get(i..i + 2)
            .ok_or_else(|| anyhow!("truncated map"))?
            .try_into()?,
    ))
}
fn u32_at(b: &[u8], i: usize) -> anyhow::Result<u32> {
    Ok(u32::from_le_bytes(
        b.get(i..i + 4)
            .ok_or_else(|| anyhow!("truncated map"))?
            .try_into()?,
    ))
}
fn position(b: &[u8]) -> anyhow::Result<[u16; 2]> {
    Ok([u16::try_from(u32_at(b, 0)?)?, u16::try_from(u32_at(b, 4)?)?])
}
fn fingerprint(fields: [u32; 6], pixels: &[u8]) -> String {
    let mut digest = ring::digest::Context::new(&ring::digest::SHA256);
    for field in fields {
        digest.update(&field.to_le_bytes());
    }
    // Ignore room IDs and cleaning coverage; retain walls, floor and unknown space.
    let geometry: Vec<u8> = pixels
        .iter()
        .map(|p| match p & 7 {
            0 => 0,
            1 => 1,
            _ => 2,
        })
        .collect();
    digest.update(&geometry);
    digest
        .finish()
        .as_ref()
        .iter()
        .flat_map(|b| {
            const HEX: &[u8] = b"0123456789abcdef";
            [
                char::from(HEX[usize::from(b >> 4)]),
                char::from(HEX[usize::from(b & 15)]),
            ]
        })
        .collect()
}
pub fn parse(bytes: &[u8]) -> anyhow::Result<Map> {
    let mut raw = Vec::new();
    if bytes.starts_with(&[0x1f, 0x8b]) {
        flate2::read::GzDecoder::new(bytes)
            .take(2 * 1024 * 1024 + 1)
            .read_to_end(&mut raw)?;
    } else {
        ensure!(bytes.len() <= 2 * 1024 * 1024, "map too large");
        raw.extend_from_slice(bytes);
    }
    ensure!(
        raw.len() <= 2 * 1024 * 1024 && raw.starts_with(b"rr"),
        "unsupported map format"
    );
    let index = u32_at(&raw, 12)?;
    let mut offset = usize::from(u16_at(&raw, 2)?);
    ensure!(offset >= 20 && offset <= raw.len(), "invalid map header");
    let (mut image, mut dock, mut robot) = (None, None, None);
    while offset < raw.len() {
        let kind = u16_at(&raw, offset)?;
        let head = usize::from(u16_at(&raw, offset + 2)?);
        let len = usize::try_from(u32_at(&raw, offset + 4)?)?;
        ensure!((8..=128).contains(&head), "invalid map block");
        let end = offset
            .checked_add(head)
            .and_then(|n| n.checked_add(len))
            .ok_or_else(|| anyhow!("invalid map length"))?;
        ensure!(end <= raw.len(), "truncated map block");
        let data = &raw[offset + head..end];
        match kind {
            1 => dock = Some(position(data)?),
            8 => robot = Some(position(data)?),
            2 => {
                ensure!(head >= 24, "invalid image header");
                let base = offset + head - 16;
                let bottom = u32_at(&raw, base)?;
                let left = u32_at(&raw, base + 4)?;
                let height = u32_at(&raw, base + 8)?;
                let width = u32_at(&raw, base + 12)?;
                ensure!(
                    width > 0
                        && height > 0
                        && width <= 1024
                        && height <= 1024
                        && left <= 1024 - width
                        && bottom <= 1024 - height
                        && u64::from(width) * u64::from(height) == u64::try_from(len)?,
                    "invalid map dimensions"
                );
                image = Some((left * 50, bottom * 50, width, height, data.to_vec()));
            }
            _ => {}
        }
        offset = end;
    }
    let (left, bottom, width, height, pixels) =
        image.ok_or_else(|| anyhow!("map image unavailable"))?;
    let dock = dock.ok_or_else(|| anyhow!("map dock unavailable"))?;
    ensure!(
        dock != [0, 0] && dock.iter().all(|p| *p <= 51200),
        "map dock reference unavailable"
    );
    let frame = fingerprint(
        [
            left,
            bottom,
            width,
            height,
            u32::from(dock[0]),
            u32::from(dock[1]),
        ],
        &pixels,
    );
    Ok(Map {
        index,
        frame,
        left,
        bottom,
        width,
        height,
        dock,
        robot,
        pixels: B64.encode(pixels),
    })
}
fn map_host(host: &str) -> bool {
    host.ends_with(".fds.api.xiaomi.com") || host.ends_with(".fds.api.mi-img.com")
}
pub async fn current(config: &Config, account: &Account, client: &mut Miio) -> anyhow::Result<Map> {
    config.validate()?;
    let result = client.call("get_map_v1", json!([])).await?;
    let name = result
        .get(0)
        .and_then(serde_json::Value::as_str)
        .filter(|s| !s.is_empty() && *s != "retry" && s.len() <= 2048)
        .ok_or_else(|| anyhow!("map not ready"))?;
    let url = if name.starts_with("http://") || name.starts_with("https://") {
        name.to_owned()
    } else {
        let result = account
            .cloud()
            .await?
            .call("/home/getmapfileurl", json!({"obj_name":name}))
            .await?;
        result["url"]
            .as_str()
            .ok_or_else(|| anyhow!("map URL unavailable"))?
            .to_owned()
    };
    let url = url.replacen("http://", "https://", 1);
    let uri: http::Uri = url.parse().map_err(|_| anyhow!("invalid map URL"))?;
    ensure!(
        uri.scheme_str() == Some("https")
            && uri.port_u16().is_none_or(|p| p == 443)
            && uri.host().is_some_and(map_host),
        "untrusted map host"
    );
    let req = http::Request::builder()
        .uri(
            uri.path_and_query()
                .map_or("/", http::uri::PathAndQuery::as_str),
        )
        .body(Full::new(moli_net::Body::new()))?;
    let (status, bytes) = moli_net::web(
        uri.host().unwrap_or_default(),
        443,
        true,
        req,
        std::time::Duration::from_secs(15),
        2 * 1024 * 1024,
    )
    .await
    .map_err(|_| anyhow!("map download unavailable"))?;
    ensure!(status.is_success(), "map download refused");
    parse(&bytes)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;
    fn fixture(pixel: u8) -> Vec<u8> {
        let mut raw = vec![0; 20];
        raw[..2].copy_from_slice(b"rr");
        raw[2..4].copy_from_slice(&20u16.to_le_bytes());
        raw[12..16].copy_from_slice(&4u32.to_le_bytes());
        raw.extend(1u16.to_le_bytes());
        raw.extend(8u16.to_le_bytes());
        raw.extend(8u32.to_le_bytes());
        raw.extend(5025u32.to_le_bytes());
        raw.extend(10025u32.to_le_bytes());
        raw.extend(2u16.to_le_bytes());
        raw.extend(24u16.to_le_bytes());
        raw.extend(4u32.to_le_bytes());
        for value in [200u32, 100, 2, 2] {
            raw.extend(value.to_le_bytes());
        }
        raw.extend([1, pixel, 0, 2]);
        raw
    }
    #[test]
    fn rr_fixture_keeps_coordinates_and_detects_geometry_changes() {
        let map = parse(&fixture(2)).unwrap();
        assert_eq!(
            (map.left, map.bottom, map.width, map.height),
            (5000, 10000, 2, 2)
        );
        assert_eq!(map.dock, [5025, 10025]);
        assert_eq!(B64.decode(&map.pixels).unwrap(), [1, 2, 0, 2]);
        assert_eq!(map.frame, parse(&fixture(3)).unwrap().frame);
        assert_ne!(map.frame, parse(&fixture(1)).unwrap().frame);
        let mut bad = fixture(2);
        bad.truncate(bad.len() - 1);
        assert!(parse(&bad).is_err());
        let mut zip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        zip.write_all(&fixture(2)).unwrap();
        assert_eq!(parse(&zip.finish().unwrap()).unwrap().frame, map.frame);
    }
    #[test]
    fn malformed_blocks_and_external_urls_are_refused() {
        assert!(parse(b"rr").is_err());
        assert!(parse(&[0; 100]).is_err());
        assert!(!map_host("fds.api.xiaomi.com.attacker.invalid"));
        assert!(!map_host("127.0.0.1"));
    }
    #[test]
    fn changed_dock_or_map_prevents_kitchen_cleaning() {
        let m = Map {
            index: 4,
            frame: "a".repeat(64),
            left: 20000,
            bottom: 20000,
            width: 200,
            height: 200,
            dock: [25500, 25500],
            robot: None,
            pixels: String::new(),
        };
        let mut z = Zone {
            bounds: [21000, 21000, 25700, 24300],
            dock: m.dock,
            map_index: 4,
            frame: m.frame.clone(),
            confirmed_at: 1,
        };
        assert!(m.matches(&z));
        z.frame = "b".repeat(64);
        assert!(!m.matches(&z));
        z.frame.clone_from(&m.frame);
        z.map_index = 5;
        assert!(!m.matches(&z));
        z.map_index = 4;
        z.dock[0] += 500;
        assert!(!m.matches(&z));
    }
}
