use crate::{ACCOUNT, Account, Config, Kind, ZONE, miio::Miio};
use anyhow::{anyhow, ensure};
use moli_core::{Access, Device, DeviceId, Kind as PointKind, PointSpec, Semantic, Unit, Value};
use moli_runtime::{BoxFuture, Driver, DriverCtx};
use serde::{Deserialize, Serialize};
use serde_json::{Value as Json, json};
use std::time::Duration;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Zone {
    pub bounds: [u16; 4],
    pub dock: [u16; 2],
    pub map_index: u32,
    pub frame: String,
    pub confirmed_at: u64,
}
impl Zone {
    pub fn valid(&self) -> bool {
        let [x1, y1, x2, y2] = self.bounds;
        self.frame.len() == 64
            && self.frame.bytes().all(|b| b.is_ascii_hexdigit())
            && x1 < x2
            && y1 < y2
            && x2 <= 51200
            && y2 <= 51200
            && (500..=10000).contains(&(x2 - x1))
            && (500..=10000).contains(&(y2 - y1))
    }
    pub fn params(&self) -> anyhow::Result<Json> {
        ensure!(self.valid(), "invalid kitchen rectangle");
        let [x1, y1, x2, y2] = self.bounds;
        Ok(json!([[x1, y1, x2, y2, 1]]))
    }
}
#[derive(Debug)]
pub struct Vacuum {
    config: Config,
}
impl Vacuum {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        config.validate()?;
        Ok(Self { config })
    }
    async fn connect(config: &Config, account: &Account) -> anyhow::Result<(Miio, Json)> {
        let mut client = Miio::connect(
            config.host,
            config.port,
            account.token.as_deref().unwrap_or_default(),
        )
        .await?;
        let info = client.read("miIO.info").await?;
        ensure!(
            info["model"].as_str() == Some("rockrobo.vacuum.v1"),
            "unexpected vacuum model"
        );
        Ok((client, info))
    }
    async fn recover(config: &Config, account: &Account) -> anyhow::Result<(Miio, Json, Account)> {
        let devices = account.cloud().await?.devices().await?;
        let (target, current) = Self::recovered_target(config, account, &devices)?;
        let (mut client, info) = Self::connect(&target, &current).await?;
        let status = client.read("get_status").await?;
        ensure!(
            status.as_array().is_some_and(|a| !a.is_empty()),
            "vacuum status unavailable"
        );
        Ok((client, info, current))
    }
    fn recovered_target(
        config: &Config,
        account: &Account,
        devices: &[Json],
    ) -> anyhow::Result<(Config, Account)> {
        let matches: Vec<_> = devices
            .iter()
            .filter(|d| {
                d["did"].as_str() == Some(&account.device)
                    && d["model"].as_str() == Some(&account.model)
            })
            .collect();
        let [device] = matches.as_slice() else {
            anyhow::bail!("vacuum identity unavailable");
        };
        let mut current = account.clone();
        current.token = device["token"].as_str().map(str::to_owned);
        ensure!(
            current.valid(Kind::Vacuum),
            "invalid recovered vacuum account"
        );
        let mut target = config.clone();
        target.host = device["localip"]
            .as_str()
            .ok_or_else(|| anyhow!("vacuum address unavailable"))?
            .parse()?;
        target.validate()?;
        Ok((target, current))
    }
    pub async fn verify(config: &Config, account: &Account) -> anyhow::Result<Json> {
        ensure!(account.valid(Kind::Vacuum), "invalid vacuum account");
        let mut client = Miio::connect(
            config.host,
            config.port,
            account.token.as_deref().unwrap_or_default(),
        )
        .await?;
        let info = client.read("miIO.info").await?;
        ensure!(
            info["model"].as_str() == Some("rockrobo.vacuum.v1"),
            "unexpected vacuum model"
        );
        let status = client.read("get_status").await?;
        ensure!(
            status.as_array().is_some_and(|a| !a.is_empty()),
            "vacuum status unavailable"
        );
        Ok(json!({"model":info["model"],"firmware":info["fw_ver"]}))
    }
}
fn spec(
    key: &str,
    label: &str,
    kind: PointKind,
    semantic: Semantic,
    unit: Option<Unit>,
    write: bool,
) -> PointSpec {
    PointSpec {
        key: key.into(),
        label: moli_i18n::tr(label).into(),
        kind,
        semantic,
        unit,
        access: Access { read: true, write },
    }
}
fn numeric(min: f64, max: f64) -> PointKind {
    PointKind::Numeric {
        min: Some(min),
        max: Some(max),
        step: None,
    }
}
fn device(ctx: &DriverCtx, id: &DeviceId, config: &Config, writable: bool) -> Device {
    Device {
        id: id.clone(),
        instance: ctx.instance().clone(),
        native_name: "Mi Robot Vacuum".into(),
        manufacturer: Some("Xiaomi".into()),
        model: Some("rockrobo.vacuum.v1".into()),
        description: None,
        native_room: config.room.as_deref().map(Into::into),
        members: Vec::new(),
        points: vec![
            spec(
                "battery",
                "xiaomi.vacuum.battery",
                numeric(0.0, 100.0),
                Semantic::Battery,
                Some(Unit::Percent),
                false,
            ),
            spec(
                "state",
                "xiaomi.vacuum.state",
                numeric(0.0, 100.0),
                Semantic::Other,
                None,
                false,
            ),
            spec(
                "clean_area",
                "xiaomi.vacuum.area",
                numeric(0.0, 10000.0),
                Semantic::Other,
                Some(Unit::Other("m²".into())),
                false,
            ),
            spec(
                "clean_time",
                "xiaomi.vacuum.time",
                numeric(0.0, 86400.0),
                Semantic::Other,
                Some(Unit::Second),
                false,
            ),
            spec(
                "error",
                "xiaomi.vacuum.error",
                numeric(0.0, 10000.0),
                Semantic::Other,
                None,
                false,
            ),
            spec(
                "firmware",
                "xiaomi.vacuum.firmware",
                PointKind::Text,
                Semantic::Config,
                None,
                false,
            ),
            spec(
                "kitchen_ready",
                "xiaomi.vacuum.kitchen_ready",
                PointKind::Binary,
                Semantic::Config,
                None,
                false,
            ),
            spec(
                "action",
                "xiaomi.vacuum.action",
                PointKind::Enum {
                    values: ["pause", "dock", "clean", "kitchen"]
                        .map(Into::into)
                        .to_vec(),
                },
                Semantic::Control,
                None,
                writable,
            ),
        ],
    }
}
fn regular_command(action: &str) -> Option<&'static str> {
    match action {
        "pause" => Some("app_pause"),
        "dock" => Some("app_charge"),
        "clean" => Some("app_start"),
        _ => None,
    }
}
fn publish(ctx: &DriverCtx, id: &DeviceId, status: &Json) {
    for (key, source, divisor) in [
        ("battery", "battery", 1.0),
        ("state", "state", 1.0),
        ("clean_area", "clean_area", 1_000_000.0),
        ("clean_time", "clean_time", 1.0),
        ("error", "error_code", 1.0),
    ] {
        let value = status[source]
            .as_f64()
            .filter(|v| v.is_finite() && *v >= 0.0)
            .map_or(Value::Null, |v| Value::Float(v / divisor));
        ctx.set_state(id, key, value);
    }
}
impl Driver for Vacuum {
    fn kind(&self) -> &'static str {
        "xiaomi-vacuum"
    }
    fn run<'a>(&'a self, ctx: &'a mut DriverCtx) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let id = ctx.device_id("vacuum");
            ctx.upsert_device(device(ctx, &id, &self.config, false));
            loop {
                let secret = ctx.secret(ACCOUNT);
                let account = secret
                    .as_deref()
                    .and_then(|s| serde_json::from_str::<Account>(s).ok())
                    .filter(|a| a.valid(Kind::Vacuum));
                let Some(account) = account else {
                    ctx.upsert_device(device(ctx, &id, &self.config, false));
                    ctx.set_availability(&id, false);
                    ctx.wait_for(moli_i18n::tr!("xiaomi.account.wait"));
                    tokio::select! {()=tokio::time::sleep(Duration::from_secs(3))=>{},command=ctx.next_command()=>{if let Some(c)=command{c.reply(Err(moli_i18n::tr!("xiaomi.vacuum.read_only")));}else{return Ok(());}}}
                    continue;
                };
                let (mut client, info, account) = match Self::connect(&self.config, &account).await
                {
                    Ok((client, info)) => (client, info, account),
                    Err(_) => Self::recover(&self.config, &account).await?,
                };
                ensure!(
                    info["model"].as_str() == Some("rockrobo.vacuum.v1"),
                    "unexpected vacuum model"
                );
                ctx.upsert_device(device(ctx, &id, &self.config, true));
                ctx.set_state(&id, "firmware", Value::from_json(&info["fw_ver"]));
                let mut poll = tokio::time::interval(Duration::from_secs(self.config.poll_s));
                poll.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                loop {
                    tokio::select! {
                     _=poll.tick()=>{
                      if ctx.secret(ACCOUNT)!=secret{break;}
                      let reply=client.read("get_status").await?;let status=reply.get(0).ok_or_else(||anyhow!("invalid vacuum status"))?;
                      publish(ctx,&id,status);
                      let zone=ctx.secret(ZONE).and_then(|s|serde_json::from_str::<Zone>(&s).ok()).is_some_and(|z|z.valid());ctx.set_state(&id,"kitchen_ready",Value::Bool(zone));
                      ctx.set_availability(&id,true);ctx.ready();
                     }
                     command=ctx.next_command()=>{
                      let Some(command)=command else{return Ok(());};if command.abandoned(){continue;}
                      if ctx.secret(ACCOUNT)!=secret {command.reply(Err(moli_i18n::tr!("xiaomi.vacuum.read_only")));continue;}
                      let (method,params)=match (&*command.key,&command.value){
                       ("action",Value::Text(v)) if &**v!="kitchen"=>{
                        let Some(method)=regular_command(v) else {command.reply(Err(moli_i18n::tr!("xiaomi.vacuum.invalid")));continue;};
                        (method,json!([]))
                       },
                       ("action",Value::Text(v)) if &**v=="kitchen"=>{
                        let Some(zone)=ctx.secret(ZONE).and_then(|s|serde_json::from_str::<Zone>(&s).ok()).filter(|z: &Zone| z.valid() && z.confirmed_at > 0) else {command.reply(Err(moli_i18n::tr!("xiaomi.zone.required")));continue;};
                        let status=client.read("get_status").await?;
                        if !status.get(0).and_then(|s|s["state"].as_u64()).is_some_and(|s|matches!(s,3|8|10)) {command.reply(Err(moli_i18n::tr!("xiaomi.vacuum.busy")));continue;}
                        let result=crate::map::current(&self.config,&account).await.and_then(|map|{map.rebase(&zone)?.params()});
                        if let Ok(params)=result {("app_zoned_clean",params)} else {command.reply(Err(moli_i18n::tr!("xiaomi.zone.changed")));continue;}
                       }
                       _=>{command.reply(Err(moli_i18n::tr!("xiaomi.vacuum.invalid")));continue;}
                      };
                      if command.abandoned(){continue;}
                      // No retries or whole-floor fallback: an unanswered motor command is uncertain.
                      match client.call(method,params).await{
                       Ok(_)=>{command.reply(Ok(()));let status=client.read("get_status").await?;if let Some(s)=status.get(0){publish(ctx,&id,s);}}
                       Err(e)=>{command.reply(Err(moli_i18n::tr!("xiaomi.vacuum.failed")));return Err(e);}
                      }
                     }
                    }
                }
            }
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recovery_is_bound_to_device_identity_and_local_addresses() {
        let config: Config =
            serde_json::from_value(json!({"host":"127.0.0.1","relay_host":"127.0.0.1"})).unwrap();
        let account = Account {
            user: "123".into(),
            region: "de".into(),
            device: "456".into(),
            model: "rockrobo.vacuum.v1".into(),
            relay_config: "xiaomi:\n  \"123\": dummy-token\n".into(),
            token: Some("01".repeat(16)),
        };
        let device = json!({"did":"456","model":"rockrobo.vacuum.v1","localip":"192.168.1.10","token":"02".repeat(16)});
        let (target, recovered) =
            Vacuum::recovered_target(&config, &account, std::slice::from_ref(&device)).unwrap();
        assert_eq!(target.host.to_string(), "192.168.1.10");
        assert_eq!(recovered.token, Some("02".repeat(16)));
        assert!(Vacuum::recovered_target(&config, &account, &[]).is_err());
        assert!(
            Vacuum::recovered_target(&config, &account, &[device.clone(), device.clone()]).is_err()
        );
        for (key, value) in [
            ("did", "999"),
            ("model", "other"),
            ("token", "bad"),
            ("localip", "8.8.8.8"),
        ] {
            let mut bad = device.clone();
            bad[key] = json!(value);
            assert!(Vacuum::recovered_target(&config, &account, &[bad]).is_err());
        }
    }
    #[test]
    fn regular_cleaning_is_explicit_and_never_a_kitchen_fallback() {
        assert_eq!(regular_command("clean"), Some("app_start"));
        assert_eq!(regular_command("pause"), Some("app_pause"));
        assert_eq!(regular_command("dock"), Some("app_charge"));
        assert_eq!(regular_command("kitchen"), None);
        assert_eq!(regular_command("unknown"), None);
    }
    #[test]
    fn kitchen_is_one_bounded_rectangle_never_global() {
        let mut z = Zone {
            bounds: [20000, 21000, 24700, 24300],
            dock: [25500, 25500],
            map_index: 1,
            frame: "a".repeat(64),
            confirmed_at: 1,
        };
        assert_eq!(
            z.params().unwrap(),
            json!([[20000, 21000, 24700, 24300, 1]])
        );
        z.bounds[0] = 25000;
        assert!(z.params().is_err());
        z.bounds = [0, 0, 51201, 3000];
        assert!(!z.valid());
    }
}
