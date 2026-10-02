// Copyright (c) 2026 Blockmaster2706 <wynter@breedable.men>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use leptos::{prelude::*, task::spawn_local};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Clone, serde::Deserialize)]
struct UpdateInfo {
    version: String,
    notes: String,
}

#[component]
pub fn AppUpdates() -> impl IntoView {
    let (update, set_update) = signal(None::<UpdateInfo>);
    let (busy, set_busy) = signal(false);
    let (message, set_message) = signal(String::new());
    let check = move |manual: bool| {
        if busy.get_untracked() {
            return;
        }
        set_busy.set(true);
        spawn_local(async move {
            match invoke("check_app_update", JsValue::NULL).await {
                Ok(value) => match serde_wasm_bindgen::from_value::<Option<UpdateInfo>>(value) {
                    Ok(info) => {
                        if manual {
                            set_message.set(if info.is_some() {
                                String::new()
                            } else {
                                "No update available.".into()
                            });
                        }
                        set_update.set(info);
                    }
                    Err(error) => {
                        set_message.set(format!("Could not read update information: {error}"))
                    }
                },
                Err(error) if manual => {
                    set_message.set(error.as_string().unwrap_or_else(|| format!("{error:?}")))
                }
                Err(_) => {}
            }
            set_busy.set(false);
        });
    };
    if !cfg!(debug_assertions) {
        check(false);
    }
    let install = move |_| {
        if busy.get_untracked() {
            return;
        }
        set_busy.set(true);
        set_message.set("Downloading update. Maqai will restart after installation.".into());
        spawn_local(async move {
            if let Err(error) = invoke("install_app_update", JsValue::NULL).await {
                set_message.set(error.as_string().unwrap_or_else(|| format!("{error:?}")));
                set_busy.set(false);
            }
        });
    };
    view! {
        <div class="app-updates">
            <button type="button" disabled=move || busy.get() on:click=move |_| check(true)>"Check for updates"</button>
            <Show when=move || update.get().is_some()>
                <div class="update-notice" role="status">
                    <p>{move || update.get().map(|u| format!("Maqai {} is available.", u.version)).unwrap_or_default()}</p>
                    <details><summary>"Release notes"</summary><p>{move || update.get().map(|u| u.notes).unwrap_or_default()}</p></details>
                    <p>"Install before starting or after finishing your rooting session. Maqai will restart."</p>
                    <button type="button" disabled=move || busy.get() on:click=install>"Install update and restart"</button>
                </div>
            </Show>
            <Show when=move || !message.get().is_empty()><p class="update-message" role="status">{move || message.get()}</p></Show>
        </div>
    }
}
