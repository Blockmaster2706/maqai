use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Debug, serde::Deserialize)]
pub struct UpdateMappings {
    pub devices: BTreeMap<String, DeviceUpdates>,
}

#[derive(Debug, serde::Deserialize)]
pub struct DeviceUpdates {
    pub min_incremental: String,
    pub max_incremental: String,
    pub updates: Vec<FirmwareUpdate>,
}

#[derive(Debug, serde::Deserialize)]
pub struct FirmwareUpdate {
    pub incremental: String,
    /// Full firmware version verified from the archive or package.
    pub version: String,
    /// Calendar build date in YYYY-MM-DD format, using the archive's timezone.
    pub build_date: String,
    /// Explicit download location for updates hosted outside the archive.
    pub url: Option<String>,
}

impl DeviceUpdates {
    /// Inclusive FUGU bounds, also applicable to a proposed sideload target.
    /// A build does not need to appear in the download catalog to be supported.
    pub fn supports(&self, incremental: &str) -> bool {
        let (Ok(build), Ok(min), Ok(max)) = (
            incremental.trim().parse::<u64>(),
            self.min_incremental.parse::<u64>(),
            self.max_incremental.parse::<u64>(),
        ) else {
            return false;
        };
        (min..=max).contains(&build)
    }
}

impl UpdateMappings {
    pub fn device(&self, model: &str) -> Option<&DeviceUpdates> {
        let key = match model.trim() {
            "Quest 2" => "quest2",
            "Quest Pro" => "questpro",
            "Quest 3" => "quest3",
            "Quest 3S" => "quest3s",
            _ => return None,
        };
        self.devices.get(key)
    }
}

pub fn mappings() -> &'static UpdateMappings {
    static MAPPINGS: OnceLock<UpdateMappings> = OnceLock::new();
    MAPPINGS.get_or_init(|| {
        serde_json::from_str(include_str!("update-mappings.json"))
            .expect("bundled firmware mappings must be valid")
    })
}

#[cfg(test)]
mod tests {
    use super::mappings;

    #[test]
    fn inclusive_fugu_boundaries() {
        for (model, min, max) in [
            ("Quest 2", 52106880032500150u64, 52242990024200150u64),
            ("Quest Pro", 51360500027200340, 51503870024400340),
            ("Quest 3", 51943020036500520, 52433670036000520),
            ("Quest 3S", 2921110037200610, 3814840024700610),
        ] {
            let device = mappings().device(model).unwrap();
            assert!(!device.supports(&(min - 1).to_string()));
            assert!(device.supports(&min.to_string()));
            assert!(device.supports(&max.to_string()));
            assert!(!device.supports(&(max + 1).to_string()));
            assert!(!device.supports("unknown"));
            assert!(!device.supports("18446744073709551616"));
        }
        assert!(mappings().device("Phone").is_none());
    }

    #[test]
    fn catalog_support_is_limited_to_v201_through_v207() {
        for device in mappings().devices.values() {
            for update in &device.updates {
                if device.supports(&update.incremental) {
                    let major: u32 = update.version.split('.').next().unwrap().parse().unwrap();
                    assert!((201..=207).contains(&major));
                }
            }
        }
    }

    #[test]
    fn unlisted_quest_3s_builds_inside_the_range_are_supported() {
        let device = mappings().device(" Quest 3S ").unwrap();
        let build = "3814840023000610";
        assert!(!device
            .updates
            .iter()
            .any(|update| update.incremental == build));
        assert!(device.supports(build));
        assert!(device.supports(" 3814840024700610 "));
    }

    #[test]
    fn hosted_quest_3s_endpoint_has_an_explicit_download_url() {
        let device = mappings().device("Quest 3S").unwrap();
        let update = device
            .updates
            .iter()
            .find(|update| update.incremental == "3814840024700610")
            .unwrap();
        assert!(device.supports(&update.incremental));
        assert_eq!(
            update.url.as_deref(),
            Some("https://cdn.faerber.dev/files/q3s_3814840024700610.zip")
        );
        assert_eq!(update.version, "207.0.0.218.1234.1051346098");
    }
}
