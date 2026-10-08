use leptos::prelude::*;
use maqai_types::DeviceState;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
export async function rebootSideloadFromOs(args, callback) {
    const channel = new window.__TAURI__.core.Channel();
    channel.onmessage = callback;
    return window.__TAURI__.core.invoke('reboot_sideload_from_os', { ...args, onProgress: channel });
}
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = rebootSideloadFromOs)]
    async fn reboot_sideload_from_os(
        args: JsValue,
        callback: &js_sys::Function,
    ) -> Result<JsValue, JsValue>;
}

#[component]
pub fn FirmwareUpdater() -> impl IntoView {
    let backend_state = crate::backend_state::use_backend_state();

    view!{
        <h1>"Update Firmware"</h1>
        <Show when=move || backend_state.get().state == DeviceState::Disconnected>
            "Please turn on your Quest while holding the volume down button. Then, select \"sideload update\" using the volume buttons and confirm using the power button. Then plug it into your PC using a USB Cable."
        </Show>
        <Show when=move || backend_state.get().state == DeviceState::Connected>
            "To install updates to your device it has to be in sideload mode. Do you want maqai to reboot your device to sideload mode?"
            <button on:click=move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = reboot_sideload_from_os(JsValue::NULL, &js_sys::Function::new_no_args("console.log('progress')")).await;
                });
            }>"Reboot to Sideload Mode"</button>
        </Show>
        <Show when=move || backend_state.get().state == DeviceState::Bootloader>
            "Please select \"sideload update\" using the volume buttons on your device, then confirm using the power button."
        </Show>
        <Show when=move || backend_state.get().state == DeviceState::Bootloader>
            "Device in Sideload Mode"
        </Show>
    }
}
