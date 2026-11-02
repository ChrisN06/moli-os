//! Mi-Light UDP v6 RGBW zones. A bridge ACK confirms delivery, never the bulb's state.
use std::{net::Ipv4Addr, time::Duration};

use anyhow::{bail, ensure};
use moli_core::{Access, Device, Kind, PointSpec, Semantic, Unit, Value};
use moli_i18n::tr;
use moli_runtime::{BoxFuture, Driver, DriverCtx};
use serde::Deserialize;
use tokio::{net::UdpSocket, time::timeout};

const OPEN: [u8; 27] = [
    0x20, 0, 0, 0, 0x16, 2, 0x62, 0x3a, 0xd5, 0xed, 0xa3, 1, 0xae, 8, 0x2d, 0x46, 0x61, 0x41, 0xa7,
    0xf6, 0xdc, 0xaf, 0xd3, 0xe6, 0, 0, 0x1e,
];
const WAIT: Duration = Duration::from_secs(2);

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub host: Ipv4Addr,
    #[serde(default = "port")]
    pub port: u16,
    pub zones: Vec<u8>,
    #[serde(default)]
    pub room: Option<String>,
}
fn port() -> u16 {
    5987
}

#[derive(Debug)]
pub struct Milight {
    config: Config,
}
impl Milight {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        ensure!(
            (config.host.is_private() || config.host.is_loopback()) && config.port > 0,
            "Mi-Light needs a local unicast address"
        );
        ensure!(
            !config.zones.is_empty() && config.zones.len() <= 4,
            "Mi-Light needs 1 to 4 zones"
        );
        let mut seen = std::collections::HashSet::new();
        ensure!(
            config
                .zones
                .iter()
                .all(|&z| (1..=4).contains(&z) && seen.insert(z)),
            "Mi-Light zones must be unique, from 1 to 4; all-zones commands are forbidden"
        );
        Ok(Self { config })
    }
}

fn spec(
    key: &str,
    label: String,
    kind: Kind,
    unit: Option<Unit>,
    semantic: Semantic,
    write: bool,
) -> PointSpec {
    PointSpec {
        key: key.into(),
        label: label.into(),
        kind,
        access: Access { read: true, write },
        unit,
        semantic,
    }
}

impl Driver for Milight {
    fn kind(&self) -> &'static str {
        "milight"
    }
    fn run<'a>(&'a self, ctx: &'a mut DriverCtx) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            for zone in &self.config.zones {
                let id = ctx.device_id(&zone.to_string());
                ctx.upsert_device(Device {
                    id: id.clone(),
                    instance: ctx.instance().clone(),
                    native_name: format!("MI-Light {zone}").into(),
                    manufacturer: Some("Mi-Light".into()),
                    model: Some("RGBW (UDP v6)".into()),
                    description: Some(tr!("pilotes.milight.assumed").into()),
                    native_room: self.config.room.as_deref().map(Into::into),
                    members: Vec::new(),
                    points: vec![
                        spec(
                            "on",
                            tr!("pilotes.milight.on"),
                            Kind::Binary,
                            None,
                            Semantic::OnOff,
                            true,
                        ),
                        spec(
                            "brightness",
                            tr!("pilotes.milight.brightness"),
                            Kind::Numeric {
                                min: Some(0.0),
                                max: Some(100.0),
                                step: Some(1.0),
                            },
                            Some(Unit::Percent),
                            Semantic::Brightness,
                            true,
                        ),
                        spec(
                            "color",
                            tr!("pilotes.milight.color"),
                            Kind::Text,
                            None,
                            Semantic::Color,
                            true,
                        ),
                        spec(
                            "state_estimated",
                            tr!("pilotes.milight.assumed"),
                            Kind::Binary,
                            None,
                            Semantic::Config,
                            false,
                        ),
                    ],
                });
                // Neither a restart nor a bridge reply reveals the lamp's real state.
                for key in ["on", "brightness", "color"] {
                    ctx.set_state(&id, key, Value::Null);
                }
                ctx.set_state(&id, "state_estimated", Value::Bool(true));
                ctx.set_availability(&id, false);
            }
            let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
            socket.connect((self.config.host, self.config.port)).await?;
            let mut session = Session::open(socket).await?;
            for id in ctx.devices() {
                ctx.set_availability(&id, true);
            }
            ctx.ready();
            let mut refresh = tokio::time::interval(Duration::from_secs(60));
            refresh.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            refresh.tick().await;
            loop {
                tokio::select! {
                    command = ctx.next_command() => {
                        let Some(command) = command else { return Ok(()); };
                        if command.abandoned() { continue; }
                        let zone = self.config.zones.iter().copied().find(|z| ctx.device_id(&z.to_string()) == command.device.id);
                        let Some(zone) = zone else { command.reply(Err("unknown Mi-Light zone".into())); continue; };
                        let packets = match commands(zone, &command.key, &command.value) {
                            Ok(p) => p,
                            Err(e) => { command.reply(Err(e.to_string())); continue; }
                        };
                        for body in packets {
                            if command.abandoned() { break; }
                            if let Err(e) = session.send(body).await {
                                command.reply(Err(tr!("pilotes.milight.ack_failed")));
                                return Err(e); // No replay: the supervisor drains pending commands.
                            }
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                        if command.abandoned() {
                            for key in ["on", "brightness", "color"] { ctx.set_state(&command.device.id, key, Value::Null); }
                        } else {
                            let on = command.value != Value::Bool(false) && !(command.key.as_ref() == "brightness" && command.value.as_f64() == Some(0.0));
                            ctx.set_state(&command.device.id, "on", Value::Bool(on));
                            ctx.set_state(&command.device.id, &command.key, command.value.clone());
                            command.reply(Ok(()));
                        }
                    }
                    _ = refresh.tick() => { session.refresh().await?; }
                }
            }
        })
    }
}

#[derive(Debug)]
struct Session {
    socket: UdpSocket,
    id: [u8; 2],
    sequence: u8,
}
impl Session {
    async fn open(socket: UdpSocket) -> anyhow::Result<Self> {
        let mut s = Self {
            socket,
            id: [0, 0],
            sequence: 0,
        };
        s.refresh().await?;
        Ok(s)
    }
    async fn refresh(&mut self) -> anyhow::Result<()> {
        let mut response = [0u8; 128];
        self.socket.send(&OPEN).await?;
        let n = timeout(WAIT, self.socket.recv(&mut response)).await??;
        ensure!(
            n >= 22 && n == usize::from(response[4]) + 5 && response[..4] == [0x28, 0, 0, 0],
            "invalid Mi-Light session reply"
        );
        self.id.copy_from_slice(&response[19..21]);
        Ok(())
    }
    async fn send(&mut self, body: [u8; 10]) -> anyhow::Result<()> {
        self.sequence = self.sequence.wrapping_add(1);
        self.socket
            .send(&packet(self.id, self.sequence, body))
            .await?;
        let mut reply = [0u8; 128];
        let n = timeout(WAIT, self.socket.recv(&mut reply)).await??;
        ensure!(
            n == 8
                && reply[..6] == [0x88, 0, 0, 0, 3, 0]
                && reply[6] == self.sequence
                && reply[7] == 0,
            "invalid Mi-Light acknowledgement"
        );
        Ok(())
    }
}
fn packet(session: [u8; 2], sequence: u8, body: [u8; 10]) -> [u8; 22] {
    let mut out = [0u8; 22];
    out[..10].copy_from_slice(&[0x80, 0, 0, 0, 0x11, session[0], session[1], 0, sequence, 0]);
    out[10..20].copy_from_slice(&body);
    out[21] = out[10..21].iter().fold(0u8, |sum, &b| sum.wrapping_add(b));
    out
}
fn body(zone: u8, kind: u8, value: u8) -> [u8; 10] {
    [0x31, 0, 0, 7, kind, value, 0, 0, 0, zone]
}
fn commands(zone: u8, key: &str, value: &Value) -> anyhow::Result<Vec<[u8; 10]>> {
    ensure!((1..=4).contains(&zone), "invalid Mi-Light zone");
    let on = body(zone, 3, 1);
    match (key, value) {
        ("on", Value::Bool(on)) => Ok(vec![body(zone, 3, if *on { 1 } else { 2 })]),
        ("brightness", _) => {
            let v = value
                .as_f64()
                .filter(|v| v.is_finite() && (0.0..=100.0).contains(v) && v.fract() == 0.0)
                .ok_or_else(|| anyhow::anyhow!("invalid brightness"))?;
            if v == 0.0 {
                return Ok(vec![body(zone, 3, 2)]);
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let level = v as u8; // Validated integer in 1..=100.
            Ok(vec![on, body(zone, 2, level)])
        }
        ("color", Value::Text(hex)) => {
            let digits = hex
                .strip_prefix('#')
                .filter(|s| s.len() == 6 && s.bytes().all(|b| b.is_ascii_hexdigit()))
                .ok_or_else(|| anyhow::anyhow!("invalid RGB color"))?;
            let number = u32::from_str_radix(digits, 16)?;
            #[allow(clippy::cast_possible_truncation)]
            let rgb = [(number >> 16) as u8, (number >> 8) as u8, number as u8];
            if rgb.iter().all(|&v| v == rgb[0]) {
                ensure!(rgb[0] == 255, tr!("pilotes.milight.color_hint"));
                return Ok(vec![on, body(zone, 3, 5)]);
            }
            let [r, g, b] = rgb.map(f64::from);
            let max = r.max(g).max(b);
            let delta = max - r.min(g).min(b);
            let hue = if rgb[0] >= rgb[1] && rgb[0] >= rgb[2] {
                ((g - b) / delta).rem_euclid(6.0)
            } else if rgb[1] >= rgb[2] {
                (b - r) / delta + 2.0
            } else {
                (r - g) / delta + 4.0
            };
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let wheel = (hue / 6.0 * 255.0).round() as u8;
            let color = 0xffu8.wrapping_sub(wheel).wrapping_sub(0x37);
            Ok(vec![
                on,
                [0x31, 0, 0, 7, 1, color, color, color, color, zone],
            ])
        }
        _ => bail!("unsupported Mi-Light command"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rgbw_encoding_and_validation() {
        let on = commands(1, "on", &Value::Bool(true)).unwrap()[0];
        assert_eq!(on, [0x31, 0, 0, 7, 3, 1, 0, 0, 0, 1]);
        let p = packet([0xab, 0xcd], 42, on);
        assert_eq!(&p[..10], &[0x80, 0, 0, 0, 0x11, 0xab, 0xcd, 0, 42, 0]);
        assert_eq!(p[21], 0x3d);
        assert_eq!(
            commands(2, "color", &Value::from("#ffffff")).unwrap()[1],
            body(2, 3, 5)
        );
        assert_eq!(
            commands(2, "brightness", &Value::Float(40.0)).unwrap()[1],
            body(2, 2, 40)
        );
        assert!(commands(0, "on", &Value::Bool(true)).is_err());
        assert!(commands(1, "brightness", &Value::Float(f64::NAN)).is_err());
        assert!(commands(1, "color", &Value::from("invalid")).is_err());
        assert!(commands(1, "color", &Value::from("#000000")).is_err());
        let config = |host, zones| Config {
            host,
            port: 5987,
            zones,
            room: None,
        };
        assert!(Milight::new(config(Ipv4Addr::LOCALHOST, vec![1, 2])).is_ok());
        assert!(Milight::new(config(Ipv4Addr::LOCALHOST, vec![1, 1])).is_err());
        assert!(Milight::new(config(Ipv4Addr::new(8, 8, 8, 8), vec![1])).is_err());
    }
    #[tokio::test]
    async fn session_and_ack_are_bound_to_the_expected_sequence() {
        let server = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let target = server.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let mut b = [0u8; 128];
            let (n, peer) = server.recv_from(&mut b).await.unwrap();
            assert_eq!(&b[..n], &OPEN);
            let mut response = [0u8; 22];
            response[0] = 0x28;
            response[4] = 17;
            response[19] = 0xab;
            response[20] = 0xcd;
            server.send_to(&response, peer).await.unwrap();
            for valid in [true, false] {
                let (n, peer) = server.recv_from(&mut b).await.unwrap();
                assert_eq!(n, 22);
                assert_eq!(&b[5..7], &[0xab, 0xcd]);
                let seq = if valid { b[8] } else { b[8].wrapping_add(1) };
                server
                    .send_to(&[0x88, 0, 0, 0, 3, 0, seq, 0], peer)
                    .await
                    .unwrap();
            }
        });
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        socket.connect(target).await.unwrap();
        let mut session = Session::open(socket).await.unwrap();
        session.send(body(1, 3, 1)).await.unwrap();
        assert!(session.send(body(2, 3, 2)).await.is_err());
        task.await.unwrap();
    }
}
