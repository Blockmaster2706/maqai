use leptos::prelude::*;
use maqai_types::DeviceState;

#[component]
pub fn FirmwareUpdater() -> impl IntoView {
    let backend_state = crate::backend_state::use_backend_state();

    view!{
        <h1>"Update Firmware"</h1>
        <Show when=move || backend_state.get().state == DeviceState::Disconnected>
            "Please turn on your Quest while holding the volume down button. Then, select \"sideload update\" using the volume buttons and confirm using the power button. Then plug it into your PC using a USB Cable."
        </Show>
        <Show when=move || backend_state.get().state == DeviceState::Connected>
            "Please turn off your quest and turn it back on while holding the volume down button. Then, select \"sideload update\" using the volume buttons and confirm using the power button. Then plug it into your PC using a USB Cable."
        </Show>
        <Show when=move || backend_state.get().state == DeviceState::Bootloader>
            "Please select \"sideload update\" using the volume buttons on your device, then confirm using the power button."
        </Show>
        <Show when=move || backend_state.get().state == DeviceState::Bootloader>
            "Device in Sideload Mode"
        </Show>
    }
}
