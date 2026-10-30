//! Read-only aliases for identified legacy Somfy components. Raw states stay intact.
use std::collections::BTreeMap;

use moli_core::{Access, Kind, PointSpec, Semantic, Unit, Value};
use serde_json::Value as Json;

pub(crate) fn normalized(
    model: Option<&str>,
    states: &BTreeMap<String, Json>,
) -> Vec<(PointSpec, Value)> {
    let mut out = Vec::new();
    let metric = match model {
        Some("somfythermostat:SomfyThermostatTemperatureSensor") => Some((
            "core:TemperatureState",
            "temperature",
            "pilotes.overkiz.temperature",
            Unit::Celsius,
            Semantic::Temperature,
        )),
        Some("somfythermostat:SomfyThermostatHumiditySensor") => Some((
            "core:RelativeHumidityState",
            "humidity",
            "pilotes.overkiz.humidite",
            Unit::Percent,
            Semantic::Humidity,
        )),
        Some("io:LightIOSystemSensor") => Some((
            "core:LuminanceState",
            "illuminance",
            "pilotes.overkiz.lumiere",
            Unit::Lux,
            Semantic::Illuminance,
        )),
        Some(
            "io:TotalElectricalEnergyConsumptionIOSystemSensor"
            | "io:HeatingElectricalEnergyConsumptionSensor"
            | "io:DHWElectricalEnergyConsumptionSensor"
            | "io:PlugsElectricalEnergyConsumptionSensor"
            | "io:OtherElectricalEnergyConsumptionSensor",
        ) => Some((
            "core:ElectricEnergyConsumptionState",
            "energy",
            "pilotes.overkiz.index",
            Unit::WattHour,
            Semantic::Energy,
        )),
        Some("somfythermostat:SomfyThermostatThermostatComponent") => Some((
            "core:TargetTemperatureState",
            "reported_setpoint",
            "pilotes.overkiz.consigne",
            Unit::Celsius,
            Semantic::Config,
        )),
        _ => None,
    };
    if let Some((state, key, label, unit, semantic)) = metric {
        let value = states
            .get(state)
            .and_then(|v| v.as_f64().or_else(|| v.as_str()?.parse().ok()));
        if let Some(value) = value.filter(|v| {
            v.is_finite()
                && (unit != Unit::Percent || (0.0..=100.0).contains(v))
                && (!matches!(unit, Unit::WattHour | Unit::Lux) || *v >= 0.0)
        }) {
            out.push((
                spec(
                    key,
                    label,
                    Kind::Numeric {
                        min: None,
                        max: None,
                        step: None,
                    },
                    Some(unit),
                    semantic,
                ),
                Value::Float(value),
            ));
        }
    }
    if let Some(setting) = setting(model, states) {
        out.push(setting);
    }
    out
}

fn setting(model: Option<&str>, states: &BTreeMap<String, Json>) -> Option<(PointSpec, Value)> {
    let setting = match model {
        Some("io:AtlanticElectricalHeaterIOComponent") => Some((
            "core:TargetHeatingLevelState",
            "heating_level",
            "pilotes.overkiz.niveau",
        )),
        Some("somfythermostat:SomfyThermostatThermostatComponent") => Some((
            "somfythermostat:HeatingModeState",
            "heating_mode",
            "pilotes.overkiz.mode",
        )),
        Some("io:DomesticHotWaterTankComponent") => Some((
            "io:AllowForceHeatingState",
            "force_heating",
            "pilotes.overkiz.force",
        )),
        Some("io:IORemoteController") => Some((
            "io:OneWayControllerButtonState",
            "last_button",
            "pilotes.overkiz.derniere_touche",
        )),
        _ => None,
    };
    if let Some((state, key, label)) = setting
        && let Some(value) = states.get(state).and_then(Json::as_str)
    {
        let word = match value {
            "eco" => Some("pilotes.overkiz.eco"),
            "comfort" => Some("pilotes.overkiz.confort"),
            "freeze" | "frostprotection" | "freezeMode" => Some("pilotes.overkiz.hors_gel"),
            "manualMode" => Some("pilotes.overkiz.manuel"),
            "auto" | "autoMode" => Some("pilotes.overkiz.auto"),
            "atHome" | "atHomeMode" => Some("pilotes.overkiz.present"),
            "awayMode" => Some("pilotes.overkiz.absent"),
            "sleepingMode" => Some("pilotes.overkiz.nuit"),
            "on" => Some("pilotes.overkiz.actif"),
            "off" => Some("pilotes.overkiz.inactif"),
            "up" => Some("pilotes.overkiz.montee"),
            "down" => Some("pilotes.overkiz.descente"),
            "stop" => Some("pilotes.overkiz.arret"),
            _ => None,
        };
        // Unknown native modes remain visible; never infer an active heating demand.
        return Some((
            spec(key, label, Kind::Text, None, Semantic::Config),
            Value::Text(word.map_or_else(|| value.to_owned(), moli_i18n::tr).into()),
        ));
    }
    None
}

fn spec(key: &str, label: &str, kind: Kind, unit: Option<Unit>, semantic: Semantic) -> PointSpec {
    PointSpec {
        key: key.into(),
        label: moli_i18n::tr(label).into(),
        kind,
        access: Access {
            read: true,
            write: false,
        },
        unit,
        semantic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn identified_metrics_accept_numbers_and_text_without_guessing_unknown_units() {
        for (model, state, value, unit, key) in [
            (
                "somfythermostat:SomfyThermostatTemperatureSensor",
                "core:TemperatureState",
                json!(22.5),
                Unit::Celsius,
                "temperature",
            ),
            (
                "somfythermostat:SomfyThermostatHumiditySensor",
                "core:RelativeHumidityState",
                json!("61.7"),
                Unit::Percent,
                "humidity",
            ),
            (
                "io:LightIOSystemSensor",
                "core:LuminanceState",
                json!("50"),
                Unit::Lux,
                "illuminance",
            ),
            (
                "io:HeatingElectricalEnergyConsumptionSensor",
                "core:ElectricEnergyConsumptionState",
                json!("123456"),
                Unit::WattHour,
                "energy",
            ),
        ] {
            let states = BTreeMap::from([(state.to_owned(), value.clone())]);
            let result = normalized(Some(model), &states);
            assert_eq!(result.len(), 1);
            assert_eq!(&*result[0].0.key, key);
            assert_eq!(result[0].0.unit, Some(unit));
            assert!(!result[0].0.access.write);
            assert!(matches!(result[0].1, Value::Float(_)));
            assert_eq!(states[state], value);
            assert!(normalized(Some("modbus:UnknownComponent"), &states).is_empty());
        }
    }

    #[test]
    fn invalid_readings_and_tariff_subtotals_never_become_energy() {
        for value in [
            json!("NaN"),
            json!("inf"),
            json!("-1"),
            json!(true),
            json!("bad"),
        ] {
            let states = BTreeMap::from([("core:ElectricEnergyConsumptionState".into(), value)]);
            assert!(
                normalized(Some("io:PlugsElectricalEnergyConsumptionSensor"), &states).is_empty()
            );
        }
        let states = BTreeMap::from([("core:ConsumptionTariff0State".into(), json!(1000))]);
        assert!(normalized(Some("io:PlugsElectricalEnergyConsumptionSensor"), &states).is_empty());
        let states = BTreeMap::from([("core:RelativeHumidityState".into(), json!(101))]);
        assert!(
            normalized(
                Some("somfythermostat:SomfyThermostatHumiditySensor"),
                &states
            )
            .is_empty()
        );
    }

    #[test]
    fn settings_are_read_only_and_unknown_modes_remain_literal() {
        for mode in ["eco", "comfort", "future-mode"] {
            let states = BTreeMap::from([("core:TargetHeatingLevelState".into(), json!(mode))]);
            let result = normalized(Some("io:AtlanticElectricalHeaterIOComponent"), &states);
            assert_eq!(result[0].0.semantic, Semantic::Config);
            assert!(!result[0].0.access.write);
            if mode == "future-mode" {
                assert_eq!(result[0].1, Value::from(mode));
            }
        }
    }
}
