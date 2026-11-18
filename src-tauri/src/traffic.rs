use serde_json::Value;

const ADAPTER: &str = "pch-native-v3";
const ONCOMING_ADAPTER: &str = "pch-native-v4";

pub fn adapter(entry: &Value) -> Option<&'static str> {
    if entry["conditionState"] != "pch_dry" {
        return None;
    }
    let root = super::library::package_root(entry).ok()?;
    let descriptor = super::read_json(root.join("fhcm-map.json")).ok()?;
    match descriptor["trafficAdapter"].as_str() {
        Some(ADAPTER) => Some(ADAPTER),
        Some(ONCOMING_ADAPTER) => Some(ONCOMING_ADAPTER),
        _ => None,
    }
}

pub fn supported(entry: &Value) -> bool {
    adapter(entry).is_some()
}

pub fn settings(adapter: &str, profile: Option<&Value>) -> Result<&'static str, String> {
    match profile.and_then(Value::as_str) {
        Some("off") => Ok("off"),
        Some("light") => Ok(include_str!("../../traffic-profiles/light.txt")),
        Some("medium") => Ok(include_str!("../../traffic-profiles/medium.txt")),
        Some("heavy") if adapter == ONCOMING_ADAPTER => {
            Ok(include_str!("../../traffic-profiles/heavy-oncoming.txt"))
        }
        Some("heavy") => Ok(include_str!("../../traffic-profiles/heavy.txt")),
        _ => Err("Select a PCH traffic profile".into()),
    }
}

pub fn suppress_startup(source: &str) -> Result<String, String> {
    let load = r#"<behavior id="LoadInitialTrafficCars" waitforload="false" />"#;
    let wait = r#"<behavior id="WaitForInitialTrafficCars" />"#;
    if source.matches(load).count() != 1 || source.matches(wait).count() != 2 {
        return Err("PCH civilian startup flow changed; Off could not be applied".into());
    }
    Ok(source.replace(load, "").replace(wait, ""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn selects_saved_pch_controller_files() {
        for (id, budget) in [("light", 6), ("medium", 14), ("heavy", 30)] {
            let profile = json!(id);
            let text = settings(ADAPTER, Some(&profile)).unwrap();
            assert!(text.lines().any(|line| line == format!("pool_budget={budget}")));
        }
        let oncoming = settings(ONCOMING_ADAPTER, Some(&json!("heavy"))).unwrap();
        assert!(oncoming.lines().any(|line| line == "pool_budget=42"));
        assert!(oncoming.lines().any(|line| line == "oncoming_target=26"));
        assert_eq!(settings(ONCOMING_ADAPTER, Some(&json!("off"))).unwrap(), "off");
        assert!(settings(ADAPTER, Some(&json!("custom"))).is_err());
    }

    #[test]
    fn off_suppresses_only_civilian_load_behaviors() {
        let flow = include_str!("../../conditions-integration/fixtures/pacific.xml");
        let rendered = suppress_startup(flow).unwrap();
        for behavior in ["LoadInitialTrafficCars", "WaitForInitialTrafficCars"] {
            assert!(!rendered.contains(behavior));
        }
        for state in ["rogue_load_racer_cars", "global.restore_car_position"] {
            assert_eq!(rendered.matches(state).count(), flow.matches(state).count());
        }
        assert!(suppress_startup(&rendered).is_err());
    }
}
