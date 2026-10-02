// Copyright (c) 2026 Blockmaster2706 <wynter@breedable.men>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use leptos::{prelude::*, task::spawn_local};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
export async function subscribeState(callback) {
    return window.__TAURI__.event.listen('state-changed', event => callback(event.payload));
}
export async function fetchState() {
    return window.__TAURI__.core.invoke('get_state');
}
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = subscribeState)]
    async fn subscribe_state(callback: &js_sys::Function) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = fetchState)]
    async fn fetch_state() -> Result<JsValue, JsValue>;
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DeviceInfo {
    pub revision: u64,
    pub buildnumber: String,
    pub serial: String,
    pub product: String,
    pub state: String,
    pub firmware_compatible: bool,
    pub error: String,
    pub platform: String,
}

type Subscription = (js_sys::Function, Closure<dyn FnMut(JsValue)>);

impl DeviceInfo {
    pub fn is_supported_headset(&self) -> bool {
        matches!(
            self.product.trim(),
            "Quest 2" | "Quest Pro" | "Quest 3" | "Quest 3S"
        )
    }
}

pub fn use_backend_state() -> ReadSignal<DeviceInfo> {
    let (state, set_state) = signal(DeviceInfo::default());
    let subscription = StoredValue::new_local(None::<Subscription>);
    let disposed = StoredValue::new_local(false);

    on_cleanup(move || {
        disposed.set_value(true);
        subscription.update_value(|value| {
            if let Some((unlisten, _callback)) = value.take() {
                let _ = unlisten.call0(&JsValue::UNDEFINED);
            }
        });
    });

    spawn_local(async move {
        let apply = move |value: JsValue| match serde_wasm_bindgen::from_value::<DeviceInfo>(value)
        {
            Ok(snapshot) => set_state.update(|current| {
                if snapshot.revision >= current.revision {
                    *current = snapshot;
                }
            }),
            Err(error) => leptos::logging::error!("Could not decode backend state: {error}"),
        };
        let callback = Closure::<dyn FnMut(JsValue)>::new(apply);
        let unlisten = match subscribe_state(callback.as_ref().unchecked_ref()).await {
            Ok(value) => value.unchecked_into::<js_sys::Function>(),
            Err(error) => {
                leptos::logging::error!("Could not subscribe to backend state: {error:?}");
                return;
            }
        };
        if disposed.try_get_value().unwrap_or(true) {
            let _ = unlisten.call0(&JsValue::UNDEFINED);
            return;
        }
        subscription.set_value(Some((unlisten, callback)));
        match fetch_state().await {
            Ok(value) if !disposed.try_get_value().unwrap_or(true) => apply(value),
            Err(error) => leptos::logging::error!("Could not fetch backend state: {error:?}"),
            _ => {}
        }
    });

    state
}
