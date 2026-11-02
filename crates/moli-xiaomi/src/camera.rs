use crate::{ACCOUNT, Account, Config, Kind, cloud::encode, restore_relay};
use anyhow::ensure;
use moli_core::Device;
use moli_runtime::{BoxFuture, Driver, DriverCtx};
use std::time::Duration;
#[derive(Debug)]
pub struct Camera {
    config: Config,
}
impl Camera {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        config.validate()?;
        Ok(Self { config })
    }
}
impl Driver for Camera {
    fn kind(&self) -> &'static str {
        "xiaomi-camera"
    }
    fn run<'a>(&'a self, ctx: &'a mut DriverCtx) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let id = ctx.device_id("camera");
            let video = self
                .config
                .relay()
                .video(ctx.instance().as_str(), None, "h265")?;
            ctx.provide_video(&id, video.clone());
            ctx.upsert_device(Device {
                id: id.clone(),
                instance: ctx.instance().clone(),
                native_name: "Mi Camera 2K".into(),
                manufacturer: Some("Xiaomi".into()),
                model: Some("isa.camera.hlc7".into()),
                description: None,
                native_room: self.config.room.as_deref().map(Into::into),
                members: Vec::new(),
                points: Vec::new(),
            });
            let mut last = None;

            loop {
                let secret = ctx.secret(ACCOUNT);
                let account = secret
                    .as_deref()
                    .and_then(|s| serde_json::from_str::<Account>(s).ok())
                    .filter(|a| a.valid(Kind::Camera));
                if let Some(account) = account {
                    restore_relay(&self.config, &account).await?;
                    if last != secret {
                        ensure!(
                            account.model == "isa.camera.hlc7",
                            "unsupported Xiaomi camera"
                        );
                        let source = format!(
                            "xiaomi://{}:{}@{}?did={}&model={}&subtype=hd&audio=0#backchannel=0",
                            account.user,
                            account.region,
                            self.config.host,
                            encode(&account.device),
                            encode(&account.model)
                        );
                        video.set_source(Some(source)).await;
                        last = secret;
                    }
                    ctx.set_availability(&id, true);
                    ctx.ready();
                } else {
                    video.set_source(None).await;
                    last = None;
                    ctx.set_availability(&id, false);
                    ctx.wait_for(moli_i18n::tr!("xiaomi.account.wait"));
                }
                tokio::select! {()=ctx.cancelled()=>return Ok(()),()=tokio::time::sleep(Duration::from_secs(15))=>{}}
            }
        })
    }
}
