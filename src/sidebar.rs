// Copyright (c) 2026 Blockmaster2706 <wynter@breedable.men>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn sidebar() -> impl IntoView {
    let navigate_home = use_navigate();
    let navigate_updater = use_navigate();
    let backend_state = crate::backend_state::use_backend_state();

    let check_version_compatible = move || {
        let device = backend_state.get();
        device.state != "connected" || device.product.is_empty() || device.firmware_compatible
    };

    let get_productname_or_generic = move || {
        let product = backend_state.get().product;
        if product.is_empty() {
            match backend_state.get().state.as_str() {
                "unauthorized" => "Unauthorized Device".to_string(),
                "offline" => "Offline Device".to_string(),
                "bootloader" => "Device in Bootloader Mode".to_string(),
                "sideload" => "Device in Sideload Mode".to_string(),
                "disconnected" => "No Device".to_string(),
                _ => product,
            }
        } else {
            product
        }
    };

    view! {
        <div class="sidebar">
                <h2>"Maqai"</h2>

                <button class="menuentry" on:click=move |_| navigate_home("/", Default::default())>"Getting Started"</button>
                <button class="menuentry" on:click=move |_| navigate_updater("/Updater", Default::default())>"Update Firmware"</button>
                <crate::updates::AppUpdates />

                <div class="sidebar-footer">
                    <div class="version-warning" class:hidden=move || check_version_compatible()>
                        <p>{move || if backend_state.get().is_supported_headset() {
                            "Your firmware version is incompatible with Singularity!"
                        } else {
                            "Unsupported device. Connect a Meta Quest headset."
                        }}</p>
                    </div>

                    <div
                        class="device-info"
                        class:disconnected=move || matches!(backend_state.get().state.as_str(), "" | "disconnected")
                        class:connected=move || backend_state.get().state == "connected"
                        class:unauthorized=move || backend_state.get().state == "unauthorized"
                        class:offline=move || backend_state.get().state == "offline"
                        class:bootloader=move || backend_state.get().state == "bootloader"
                        class:sideload=move || backend_state.get().state == "sideload"
                    >
                        <p>{move || get_productname_or_generic()} " detected"</p>
                        <p hidden=move || backend_state.get().state != "unauthorized">"Please put on your headset and accept the request."</p>
                        <div class="buildnumber">"Firmware Version"</div>
                        <div class="buildnumber">{move || backend_state.get().buildnumber}</div>
                    </div>
                </div>
            </div>
    }
}
