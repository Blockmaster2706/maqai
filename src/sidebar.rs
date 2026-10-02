use leptos::prelude::*;

#[component]
pub fn sidebar() -> impl IntoView {
    let backend_state = crate::backend_state::use_backend_state();

    let check_version_compatible = move || {
        let build = backend_state.get().buildnumber.as_str().trim().parse::<u64>();
        let product = backend_state.get().product;

        // If we can't read the product, show no warning yet.
        if product.is_empty() {
            return true;
        }
        
        if product == "Quest 2" {
            if let Ok(build) = build {
                if build >= 52106880032500150 && build < 52242990035800150 {
                    return true;
                }
            }
        }
        if product == "Quest Pro" {
            if let Ok(build) = build {
                if build >= 51360500027200340 && build < 51503870035800340 {
                    return true;
                }
            }
        }
        if product == "Quest 3" {
            if let Ok(build) = build {
                if build >= 51943020036500520 && build < 52433670048800520 {
                    return true;
                }
            }
        }
        if product == "Quest 3S" {
            if let Ok(build) = build {
                if build >= 2921110037200610 && build < 3814840024700611 {
                    return true;
                }
            }
        }
        false
    };

    let get_productname_or_generic = move || {
        let product = backend_state.get().product;
        if product.is_empty() {
            match backend_state.get().state.as_str() {
                "unauthorized" => "Unauthorized Device".to_string(),
                "bootloader" => "Device in Bootloader Mode".to_string(),
                "sideload" => "Device in Sideload Mode".to_string(),
                _ => product,
            }
        } else {
            product
        }
    };

    view! {
        <div class="sidebar">
                <h2>"Maqai"</h2>

                <button class="menuentry" href="#">"Getting Started"</button>
                <button class="menuentry" href="#">"Settings"</button>
                <button class="menuentry" href="#">"Settings"</button>
                <button class="menuentry" href="#">"Settings"</button>
                <button class="menuentry" href="#">"Settings"</button>

                <div class="sidebar-footer">
                    <div class="version-warning" class:hidden=move || check_version_compatible()>
                        <p>"Your firmware version is incompatible with Singularity!"</p>
                    </div>

                    <div
                        class="device-info"
                        class:disconnected=move || matches!(backend_state.get().state.as_str(), "" | "disconnected")
                        class:connected=move || backend_state.get().state == "connected"
                        class:unauthorized=move || backend_state.get().state == "unauthorized"
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