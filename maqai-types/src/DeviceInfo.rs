use crate::DeviceState;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DeviceInfo {
    pub revision: u64,
    pub buildnumber: String,
    pub serial: String,
    pub product: String,
    pub state: DeviceState,
    pub firmware_compatible: bool,
    pub error: String,
    pub platform: String,
}

impl DeviceInfo {
    pub fn is_supported_headset(&self) -> bool {
        matches!(
            self.product.trim(),
            "Quest 2" | "Quest Pro" | "Quest 3" | "Quest 3S"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_info_uses_the_tauri_state_wire_format() {
        let info = DeviceInfo {
            state: DeviceState::Connected,
            product: "Quest 3".into(),
            ..Default::default()
        };
        let value = serde_json::to_value(&info).unwrap();
        assert_eq!(value["state"], "connected");
        assert_eq!(
            serde_json::from_value::<DeviceInfo>(value).unwrap().state,
            info.state
        );
    }
}
