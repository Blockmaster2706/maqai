#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceState {
    Connected,
    Unauthorized,
    Offline,
    Bootloader,
    Sideload,
    #[default]
    Disconnected,
}
