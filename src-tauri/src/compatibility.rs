// Copyright (c) 2026 Blockmaster2706 <wynter@breedable.men>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

pub fn supported(model: &str, build: &str) -> bool {
    maqai_types::update_mappings::mappings()
        .device(model)
        .is_some_and(|device| device.supports(build))
}

#[cfg(test)]
mod tests {
    use super::supported;

    #[test]
    fn firmware_boundaries_and_invalid_models() {
        for (model, min, max) in [
            ("Quest 2", 52106880032500150u64, 52242990024200150u64),
            ("Quest Pro", 51360500027200340, 51503870024400340),
            ("Quest 3", 51943020036500520, 52433670036000520),
            ("Quest 3S", 2921110037200610, 3814840024700610),
        ] {
            assert!(!supported(model, &(min - 1).to_string()));
            assert!(supported(model, &min.to_string()));
            assert!(supported(model, &max.to_string()));
            assert!(!supported(model, &(max + 1).to_string()));
        }
        assert!(supported("Quest 3S", "3814840023000610"));
        assert!(!supported("Phone", "52242990024200150"));
        assert!(!supported("Quest 2", "unknown"));
    }
}
