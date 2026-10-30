use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};

use moli_core::{Access, Device, DeviceId, InstanceId, Kind, PointSpec, Semantic, Unit, Value};
use moli_runtime::DriverCtx;
use serde_json::{Value as Json, json};

pub(crate) fn encode(text: &str) -> String {
    let mut encoded = String::new();
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

pub(crate) struct Equipment {
    pub url: String,
    name: String,
    model: Option<String>,
    controls: Vec<String>,
    states: BTreeMap<String, Json>,
    online: bool,
}
impl Equipment {
    pub fn parse(raw: &Json) -> Option<Self> {
        let url = raw["deviceURL"].as_str()?.to_owned();
        if url.is_empty() || url.len() > 512 {
            return None;
        }
        // A gateway may also expose a garage, lock or group: never infer shutter writes from generic commands.
        let shutter = raw["uiClass"] == "RollerShutter" && url.starts_with("io://");
        let controls = raw["definition"]["commands"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|c| shutter && c["nparams"] == 0)
            .filter_map(|c| c["commandName"].as_str())
            .filter(|name| ["open", "close", "stop"].contains(name))
            .map(str::to_owned)
            .collect();
        let mut equipment = Self {
            name: raw["label"]
                .as_str()
                .unwrap_or("Overkiz")
                .chars()
                .take(256)
                .collect(),
            model: raw["controllableName"].as_str().map(str::to_owned),
            url,
            controls,
            states: BTreeMap::new(),
            online: raw["available"].as_bool().unwrap_or(false),
        };
        equipment.merge(&raw["states"]);
        Some(equipment)
    }
    fn merge(&mut self, states: &Json) {
        for state in states.as_array().into_iter().flatten().take(256) {
            if let Some(name) = state["name"]
                .as_str()
                .filter(|name| name.len() <= 256 && !name.contains('/'))
            {
                let value = &state["value"];
                if value.is_boolean()
                    || value.is_number()
                    || value.as_str().is_some_and(|s| s.len() <= 1024)
                {
                    self.states.insert(name.to_owned(), value.clone());
                }
            }
        }
    }
    fn values(&self) -> Vec<(String, Value)> {
        let mut values: Vec<_> = self
            .states
            .iter()
            .map(|(name, value)| (format!("raw.{name}"), Value::from_json(value)))
            .collect();
        if let Some(closure) = self
            .states
            .get("core:ClosureState")
            .and_then(|value| {
                value
                    .as_f64()
                    .or_else(|| value.as_str()?.parse::<f64>().ok())
            })
            .filter(|n| (0.0..=100.0).contains(n))
        {
            values.push(("percent_control".into(), Value::Float(100.0 - closure)));
        }
        values
    }
    pub fn device(&self, id: &DeviceId, instance: &InstanceId, read_only: bool) -> Device {
        let mut points: Vec<_> = self
            .values()
            .iter()
            .map(|(key, value)| {
                let percent = key == "percent_control";
                PointSpec {
                    key: key.as_str().into(),
                    label: if percent {
                        moli_i18n::tr!("pilotes.overkiz.position").into()
                    } else {
                        key.as_str().into()
                    },
                    kind: match value {
                        Value::Bool(_) => Kind::Binary,
                        Value::Int(_) | Value::Float(_) => Kind::Numeric {
                            min: None,
                            max: None,
                            step: None,
                        },
                        _ => Kind::Text,
                    },
                    access: Access {
                        read: true,
                        write: false,
                    },
                    unit: percent.then_some(Unit::Percent),
                    semantic: Semantic::Other,
                }
            })
            .collect();
        points.splice(
            0..0,
            crate::readings::normalized(self.model.as_deref(), &self.states)
                .into_iter()
                .map(|(spec, _)| spec),
        );
        if !self.controls.is_empty() {
            points.push(PointSpec {
                key: "control".into(),
                label: moli_i18n::tr!("pilotes.overkiz.commande").into(),
                kind: Kind::Enum {
                    values: self.controls.iter().map(|c| c.as_str().into()).collect(),
                },
                access: Access {
                    read: false,
                    write: !read_only,
                },
                unit: None,
                semantic: Semantic::Control,
            });
        }
        points.push(PointSpec {
            key: "last_received".into(),
            label: moli_i18n::tr!("pilotes.overkiz.reception").into(),
            kind: Kind::Numeric {
                min: None,
                max: None,
                step: None,
            },
            access: Access {
                read: true,
                write: false,
            },
            unit: Some(Unit::Second),
            semantic: Semantic::Other,
        });
        Device {
            id: id.clone(),
            instance: instance.clone(),
            native_name: self.name.as_str().into(),
            manufacturer: Some("Somfy".into()),
            model: self.model.as_deref().map(Into::into),
            description: None,
            native_room: None,
            members: Vec::new(),
            points,
        }
    }
    pub fn publish(&self, ctx: &DriverCtx, id: &DeviceId) {
        for (key, value) in self.values() {
            ctx.set_state(id, &key, value);
        }
        for (spec, value) in crate::readings::normalized(self.model.as_deref(), &self.states) {
            ctx.set_state(id, &spec.key, value);
        }
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if let Ok(seconds) = i64::try_from(seconds) {
            ctx.set_state(id, "last_received", Value::Int(seconds));
        }
        ctx.set_availability(id, self.online);
    }
    pub fn event(&mut self, ctx: &DriverCtx, id: &DeviceId, event: &Json) {
        match event["name"].as_str() {
            Some("DeviceStateChangedEvent") => {
                self.merge(&event["deviceStates"]);
                self.publish(ctx, id);
            }
            Some("DeviceAvailableEvent") => {
                self.online = true;
                ctx.set_availability(id, true);
            }
            Some("DeviceUnavailableEvent") => {
                self.online = false;
                ctx.set_availability(id, false);
            }
            _ => (),
        }
    }
    pub fn order(&self, read_only: bool, key: &str, value: &Value) -> Option<Json> {
        let Value::Text(order) = value else {
            return None;
        };
        if read_only
            || !self.online
            || key != "control"
            || !self.controls.iter().any(|c| c == &**order)
        {
            return None;
        }
        Some(
            json!({"label":"Moli OS", "actions":[{"deviceURL":self.url,"commands":[{"name":order,"parameters":[]}]}]}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn shutter() -> Json {
        serde_json::from_str(include_str!(
            "../../../integrations/overkiz/fixtures/shutter.json"
        ))
        .unwrap()
    }
    #[test]
    fn closure_is_inverted_and_unknown_energy_is_never_assigned_a_unit() {
        let equipment = Equipment::parse(&shutter()).unwrap();
        let values: BTreeMap<_, _> = equipment.values().into_iter().collect();
        assert_eq!(values["percent_control"], Value::Float(75.0));
        assert_eq!(values["raw.core:TemperatureState"], Value::Float(20.5));
        let instance = InstanceId::from("overkiz");
        let device = equipment.device(
            &DeviceId::new(&instance, &encode(&equipment.url)),
            &instance,
            true,
        );
        assert!(device.points.iter().all(|p| !p.access.write));
        assert!(
            device
                .point("raw.core:TemperatureState")
                .unwrap()
                .unit
                .is_none()
        );
    }
    #[test]
    fn string_closure_deltas_update_position_and_reject_invalid_values() {
        let mut equipment = Equipment::parse(&shutter()).unwrap();
        for (closure, opening) in [("0", 100.0), ("100", 0.0), ("25.5", 74.5)] {
            equipment.merge(&json!([{"name":"core:ClosureState","value":closure}]));
            let values: BTreeMap<_, _> = equipment.values().into_iter().collect();
            assert_eq!(values["percent_control"], Value::Float(opening));
            assert_eq!(values["raw.core:ClosureState"], Value::from(closure));
        }
        for invalid in [
            json!("-1"),
            json!("101"),
            json!("NaN"),
            json!("inf"),
            json!(""),
            json!("closed"),
            json!(true),
        ] {
            equipment.merge(&json!([{"name":"core:ClosureState","value":invalid}]));
            assert!(
                !equipment
                    .values()
                    .iter()
                    .any(|(key, _)| key == "percent_control")
            );
        }
    }
    #[test]
    fn only_declared_io_shutter_commands_can_act() {
        let mut raw = shutter();
        let equipment = Equipment::parse(&raw).unwrap();
        assert!(
            equipment
                .order(true, "control", &Value::from("open"))
                .is_none()
        );
        assert!(
            equipment
                .order(false, "control", &Value::from("stop"))
                .is_none()
        );
        let payload = equipment
            .order(false, "control", &Value::from("open"))
            .unwrap();
        assert_eq!(payload["actions"][0]["commands"][0]["name"], "open");
        raw["uiClass"] = json!("GarageDoor");
        assert!(
            Equipment::parse(&raw)
                .unwrap()
                .order(false, "control", &Value::from("open"))
                .is_none()
        );
        raw["uiClass"] = json!("RollerShutter");
        raw["available"] = json!(false);
        assert!(
            Equipment::parse(&raw)
                .unwrap()
                .order(false, "control", &Value::from("open"))
                .is_none()
        );
    }
    #[test]
    fn deltas_preserve_unmentioned_state_and_ids_do_not_collide() {
        let mut equipment = Equipment::parse(&shutter()).unwrap();
        equipment.merge(&json!([{"name":"core:ClosureState","value":100}]));
        assert_eq!(equipment.states["core:TemperatureState"], 20.5);
        assert!(
            equipment
                .values()
                .contains(&("percent_control".into(), Value::Float(0.0)))
        );
        assert_eq!(encode("a+b &é"), "a%2Bb%20%26%C3%A9");
        assert_ne!(encode("a/b"), encode("a_b"));
    }
}
