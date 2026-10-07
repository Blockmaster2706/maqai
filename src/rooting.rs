use leptos::{prelude::*, task::spawn_local};
use maqai_types::DeviceState;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
export async function installSingularity(args, callback) {
    const channel = new window.__TAURI__.core.Channel();
    channel.onmessage = callback;
    return window.__TAURI__.core.invoke('install_singularity', { ...args, onProgress: channel });
}
export async function restartAdb() {
    return window.__TAURI__.core.invoke('restart_adb');
}
export async function verifyRoot(serial) {
    return window.__TAURI__.core.invoke('verify_root', { serial });
}
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = installSingularity)]
    async fn install_singularity(
        args: JsValue,
        callback: &js_sys::Function,
    ) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = restartAdb)]
    async fn restart_adb() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = verifyRoot)]
    async fn verify_root(serial: &str) -> Result<JsValue, JsValue>;
}

#[derive(Serialize)]
struct InstallArgs {
    serial: String,
    model: String,
    build: String,
}

#[derive(Clone, Default, Deserialize)]
struct InstallProgress {
    message: String,
    downloaded: u64,
    total: u64,
}

#[component]
pub fn RootingFlow() -> impl IntoView {
    let device = crate::backend_state::use_backend_state();
    let (step, set_step) = signal(0u8);
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(String::new());
    let (release, set_release) = signal(String::new());
    let (progress, set_progress) = signal(InstallProgress::default());
    let (restarting, set_restarting) = signal(false);
    let (target_serial, set_target_serial) = signal(String::new());
    let (root_output, set_root_output) = signal(String::new());
    let ready = move || {
        let current = device.get();
        current.state == DeviceState::Connected
            && current.error.is_empty()
            && !current.serial.is_empty()
            && !current.buildnumber.is_empty()
            && current.firmware_compatible
    };
    let connection_message = move || {
        let current = device.get();
        if !current.error.is_empty() {
            return current.error;
        }
        match current.state {
            DeviceState::Unauthorized => "Your Quest is connected but USB debugging is not authorized. Put on the headset, select Always allow from this computer, then Allow.".into(),
            DeviceState::Offline => "Your Quest is connected but offline. Wake it, reconnect the USB cable, and approve any USB debugging prompt.".into(),
            DeviceState::Bootloader | DeviceState::Sideload => "Your Quest is in bootloader or sideload mode. Start Horizon OS normally before continuing.".into(),
            DeviceState::Connected if current.product.is_empty() || current.buildnumber.is_empty() => "Reading the headset model and firmware. Keep it awake and connected.".into(),
            DeviceState::Connected if !current.is_supported_headset() => format!("{} detected. Maqai supports Meta Quest 2, Quest Pro, Quest 3, and Quest 3S. Disconnect this device and connect your Quest headset instead.", current.product),
            DeviceState::Connected if !current.firmware_compatible => format!("{} firmware {} is outside the supported Singularity range. Installation is unavailable for this device.", current.product, current.buildnumber),
            DeviceState::Connected => format!("{} connected. Firmware {} is compatible with Singularity.", current.product, current.buildnumber),
            DeviceState::Disconnected => "No USB headset detected. Connect your Quest with a USB data cable and keep it awake. A charging-only cable will not work.".into(),
        }
    };

    let restart = move |_| {
        if busy.get_untracked() || restarting.get_untracked() {
            return;
        }
        set_restarting.set(true);
        set_error.set(String::new());
        spawn_local(async move {
            if let Err(error) = restart_adb().await {
                set_error.set(error.as_string().unwrap_or_else(|| format!("{error:?}")));
            }
            set_restarting.set(false);
        });
    };

    let install = move |_| {
        if busy.get_untracked() || restarting.get_untracked() || !ready() {
            return;
        }
        let current = device.get_untracked();
        set_target_serial.set(current.serial.clone());
        set_busy.set(true);
        set_error.set(String::new());
        set_progress.set(InstallProgress {
            message: "Preparing installation…".into(),
            ..Default::default()
        });
        spawn_local(async move {
            let args = InstallArgs {
                serial: current.serial,
                model: current.product,
                build: current.buildnumber,
            };
            let callback = Closure::<dyn FnMut(JsValue)>::new(move |value| {
                if let Ok(update) = serde_wasm_bindgen::from_value::<InstallProgress>(value) {
                    set_progress.set(update);
                }
            });
            let result = match serde_wasm_bindgen::to_value(&args) {
                Ok(args) => install_singularity(args, callback.as_ref().unchecked_ref()).await,
                Err(error) => Err(JsValue::from_str(&error.to_string())),
            };
            match result {
                Ok(value) => {
                    set_release.set(value.as_string().unwrap_or_default());
                    set_step.set(3);
                }
                Err(error) => {
                    set_error.set(error.as_string().unwrap_or_else(|| format!("{error:?}")))
                }
            }
            set_busy.set(false);
        });
    };

    let can_verify = move || {
        device.with(|current| {
            current.state == DeviceState::Connected
                && current.error.is_empty()
                && current.is_supported_headset()
                && !current.serial.is_empty()
                && current.serial == target_serial.get()
        })
    };
    let verify = move |_| {
        if busy.get_untracked() || !can_verify() {
            return;
        }
        set_busy.set(true);
        set_error.set(String::new());
        let serial = target_serial.get_untracked();
        spawn_local(async move {
            match verify_root(&serial).await {
                Ok(value) => {
                    set_root_output.set(value.as_string().unwrap_or_default());
                    set_step.set(11);
                }
                Err(error) => {
                    set_error.set(error.as_string().unwrap_or_else(|| format!("{error:?}")))
                }
            }
            set_busy.set(false);
        });
    };

    view! {
        <section class="rooting-flow" aria-busy=move || busy.get()>
            <h1>"Root your Meta Quest"</h1>
            <p class="step-indicator">{move || format!("Step {} of 14", step.get() + 1)}</p>
            <Show when=move || step.get() == 0>
                <h2>"Before we start"</h2>
                <p>"A firmware update can remove Singularity compatibility. Avoid updating the Quest during this process."</p>
                <ol class="preparation-checklist">
                    <li>"In the headset's Settings, open Software Update and turn off automatic updates and any option to power on the headset for updates. Menu names vary by firmware. Avoid selecting Update or an update-on-shutdown option."</li>
                    <li>"Prefer a Wi-Fi network without internet access. If your phone supports it, turn off its mobile data and Wi-Fi internet connection, then enable its Wi-Fi hotspot. Keep the hotspot itself on and connect the Quest to it."</li>
                    <li>"If your phone requires mobile data to enable a hotspot, use a router with its internet/WAN connection unplugged instead. Check with another device that the network has no internet access."</li>
                    <li>"Disconnect or forget the Quest's usual internet-connected networks so it cannot reconnect automatically. Keep the computer online separately so Maqai can download the APK."</li>
                </ol>
                <p>"Turning off automatic update settings alone does not guarantee that an update cannot occur. An offline Wi-Fi network also prevents new downloads; it cannot undo an update already downloaded."</p>
                <button type="button" on:click=move |_| set_step.set(1)>"Preparation complete — continue"</button>
            </Show>
            <Show when=move || step.get() == 1>
                <h2>"Connect and authorize your Quest"</h2>
                <p>"Connect only your Quest to this computer using a USB data cable. Put on the headset and allow USB debugging when prompted."</p>
                <p role="status">{connection_message}</p>
                <Show when=ready>
                    <button class="continue-button" type="button" disabled=move || restarting.get() on:click=move |_| { set_error.set(String::new()); set_step.set(2); }>"Continue to installation"</button>
                </Show>
            </Show>
            <Show when=move || step.get() == 2>
                <h2>"Download and install Singularity"</h2>
                <p>"Maqai will download the latest stable Singularity APK from GitHub, verify its checksum, and install it with the permissions required for Wireless ADB."</p>
                <p>"Keep your Quest awake and USB connected."</p>
                <Show when=move || !busy.get() && !ready()><p role="status">{connection_message}</p></Show>
                <button type="button" disabled=move || busy.get() || restarting.get() || !ready() on:click=install>
                    {move || if busy.get() { "Installation in progress…" } else if error.get().is_empty() { "Download and install" } else { "Retry download and installation" }}
                </button>
                <Show when=move || busy.get()>
                    <p role="status">{move || progress.get().message}</p>
                    <Show when=move || { progress.get().total > 0 }>
                        <progress aria-label="APK download progress" max=move || progress.get().total value=move || progress.get().downloaded />
                        <p>{move || { let p = progress.get(); format!("{}% downloaded", p.downloaded * 100 / p.total.max(1)) }}</p>
                    </Show>
                </Show>
                <button type="button" disabled=move || busy.get() on:click=move |_| { set_error.set(String::new()); set_step.set(1); }>"Back to connection"</button>
                <Show when=|| cfg!(debug_assertions)>
                    <button type="button" disabled=move || busy.get() || restarting.get()
                        on:click=move |_| {
                            set_error.set(String::new());
                            set_release.set(String::new());
                            set_target_serial.set(device.get_untracked().serial);
                            set_step.set(3);
                        }>"Skip download and install (dev)"</button>
                </Show>
            </Show>
            <Show when=move || ((1..=2).contains(&step.get()) || step.get() == 10) && !busy.get() && device.with(|current| {
                current.state != DeviceState::Connected || !current.error.is_empty()
                    || current.product.is_empty() || current.buildnumber.is_empty()
            })>
                <details class="connection-help" open=move || !ready()>
                    <summary>"Headset not detected? Troubleshoot the connection"</summary>
                    <ol>
                        <li>"Try a known USB data cable, another USB port, and a direct connection without a hub."</li>
                        <li>"Wake and unlock the Quest. Confirm Developer Mode or your alternative ADB setup is enabled."</li>
                        <li>"Reconnect USB and accept Allow USB debugging inside the headset. Select Always allow from this computer."</li>
                        <li>"Disconnect extra Android phones and headsets. Maqai needs only one Quest connected over USB."</li>
                    </ol>
                    <Show when=move || device.get().platform == "windows">
                        <p>"Windows: install the "<a href="https://developers.meta.com/horizon/downloads/package/oculus-adb-drivers/" target="_blank" rel="noopener noreferrer">"Meta Oculus ADB driver"</a>". Extract the download, right-click android_winusb.inf, choose Install, then reconnect USB. Check Device Manager if the Quest still has a driver warning."</p>
                    </Show>
                    <Show when=move || device.get().platform == "linux">
                        <p>"Linux: install Android udev rules for your distribution (android-udev-rules on Arch; android-sdk-platform-tools-common on Debian/Ubuntu). Follow the package's group instructions, log out and back in, and reconnect the Quest. Do not run Maqai as root."</p>
                    </Show>
                    <Show when=move || device.get().platform == "macos">
                        <p>"macOS: no Meta Windows driver is needed. Check the USB data cable and adapter, connect directly where possible, and approve USB debugging inside the Quest."</p>
                    </Show>
                    <p>"If another tool is using ADB, close it and restart the ADB server. This also disconnects other ADB sessions on this computer."</p>
                    <button type="button" disabled=move || restarting.get() on:click=restart>
                        {move || if restarting.get() { "Restarting ADB…" } else { "Restart ADB server" }}
                    </button>
                </details>
            </Show>
            <Show when=move || step.get() == 3>
                <h2>"Open Singularity"</h2>
                <Show when=move || !release.get().is_empty()>
                    <p>"Singularity "{move || release.get()}" was installed successfully."</p>
                </Show>
                <Show when=move || release.get().is_empty()>
                    <p>"Download and installation skipped for development."</p>
                </Show>
                <p>"In your headset, open Library → Unknown Sources → Singularity. Leave Root on Boot off for this first root."</p>
                <button type="button" on:click=move |_| set_step.set(4)>"Singularity is open"</button>
            </Show>
            <Show when=move || step.get() == 4>
                <h2>"Prepare Wi-Fi"</h2>
                <p>"Connect the Quest to the Wi-Fi network without internet access prepared earlier, and keep it awake. Leave USB connected unless Singularity tells you to unplug it."</p>
                <button type="button" on:click=move |_| set_step.set(5)>"Wi-Fi is on and the headset is awake"</button>
            </Show>
            <Show when=move || step.get() == 5>
                <h2>"Start Wireless ADB setup"</h2>
                <p>"Tap Setup Wireless ADB in Singularity and follow its in-app guide. Do not press Root Now yet."</p>
                <button type="button" on:click=move |_| set_step.set(6)>"Wireless ADB setup is open"</button>
            </Show>
            <Show when=move || step.get() == 6>
                <h2>"Complete permission and pairing prompts"</h2>
                <p>"If Wireless debugging opens, enable it and approve the current Wi-Fi network."</p>
                <p>"If Singularity asks for a code, choose Pair device with pairing code and enter the current six-digit code into Singularity. Generate a fresh code if it expires. Follow the app's current instructions if your screens differ."</p>
                <button type="button" on:click=move |_| set_step.set(7)>"Prompts completed, or no prompts were needed"</button>
            </Show>
            <Show when=move || step.get() == 7>
                <h2>"Confirm Wireless ADB is connected"</h2>
                <p>"Return to Singularity and wait for it to report connected or setup successful. Maqai's USB status does not confirm this connection."</p>
                <details><summary>"Wireless ADB is not connecting"</summary><p>"Check Wi-Fi and Developer Mode. Close and reopen Singularity and retry setup. Restart the headset if needed, then follow Singularity's current prompts."</p></details>
                <button type="button" on:click=move |_| set_step.set(5)>"Retry Wireless ADB setup"</button>
                <button type="button" on:click=move |_| set_step.set(8)>"Singularity reports Wireless ADB connected"</button>
            </Show>
            <Show when=move || step.get() == 8>
                <h2>"Root Now"</h2>
                <p>"Press Root Now in Singularity and let it finish. If it reports failure and offers Retry or Root Now again, retry a few times. If failures continue, fully power off and on, then check the release notes."</p>
                <p>"Success should trigger a soft reboot. Do not press Root Now again after success."</p>
                <button type="button" on:click=move |_| set_step.set(9)>"Singularity succeeded and the Quest soft rebooted"</button>
            </Show>
            <Show when=move || step.get() == 9>
                <h2>"Grant Singularity root access"</h2>
                <p>"After the soft reboot, open Library → Unknown Sources → Singularity-Magisk. In Superuser, grant or enable access for Singularity. Fully close Singularity, then reopen it."</p>
                <p>"If Singularity-Magisk is missing, check Singularity's result and the current release notes. Do not repeat Root Now after a reported success."</p>
                <button type="button" on:click=move |_| set_step.set(10)>"Singularity has root permission — verify root"</button>
            </Show>
            <Show when=move || step.get() == 10>
                <h2>"Verify root"</h2>
                <p>"Reconnect the same Quest over USB, keep it awake, and approve any new USB debugging prompt."</p>
                <p>"Click Verify root, then watch inside the headset for a Shell or ADB root-access prompt. Select Grant and retry if the first check fails or times out."</p>
                <Show when=move || !can_verify()>
                    <p role="status">"Waiting for the original Quest to reconnect and authorize USB debugging."</p>
                    <p>{connection_message}</p>
                </Show>
                <button type="button" disabled=move || busy.get() || !can_verify() on:click=verify>
                    {move || if busy.get() { "Checking root…" } else { "Verify root" }}
                </button>
            </Show>
            <Show when=move || step.get() == 11>
                <h2>"Root verified"</h2>
                <p role="status">"The ADB shell returned UID 0 on Quest "{move || target_serial.get()}". Root access is confirmed."</p>
                <pre class="root-result">{move || root_output.get()}</pre>
                <p>"Keep update prevention in place. Do not press Root Now again after this successful root."</p>
                <button type="button" on:click=move |_| set_step.set(12)>"Continue to post-root setup"</button>
            </Show>
            <Show when=move || step.get() == 12>
                <h2>"Post-root: prevent firmware updates"</h2>
                <p>"Reopen Singularity in the headset and go to AIO Tweaks → Utils."</p>
                <ol class="preparation-checklist">
                    <li>"Turn on Disable Meta telemetry."</li>
                    <li>"Turn on Disable the update engine."</li>
                </ol>
                <details class="connection-help">
                    <summary>"Optional: additional blocking"</summary>
                    <p>"The guide also recommends disabling Meta Backdoor. For stricter blocking:"</p>
                    <ol>
                        <li>"Enable Meta domain blocker with Updates."</li>
                        <li>"Open Set up for the Domain Blocker."</li>
                        <li>"Enable Updates/Firmware and Telemetry/Analytics."</li>
                        <li>"Enable the blocker on boot."</li>
                    </ol>
                    <p>"Domain blocking can affect Meta online services. These additional settings are optional."</p>
                </details>
                <p>"Labels may differ between Singularity versions. "<a href="https://guides.mxr.lol/how-to-root-without-dev-mode/#post-root" target="_blank" rel="noopener noreferrer">"Read the original guide's step 17"</a>" if your screens differ."</p>
                <button type="button" on:click=move |_| set_step.set(13)>"Telemetry and update engine disabled — finish"</button>
            </Show>
            <Show when=move || step.get() == 13>
                <h2>"Setup complete"</h2>
                <p>"Root was verified on Quest "{move || target_serial.get()}" and you confirmed the post-root settings."</p>
                <p>"Do not press Root Now again after this successful root."</p>
            </Show>
            <Show when=move || !error.get().is_empty()><p class="flow-error" role="alert">{move || error.get()}</p></Show>
        </section>
    }
}
