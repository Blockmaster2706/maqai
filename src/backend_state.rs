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
}

type Subscription = (js_sys::Function, Closure<dyn FnMut(JsValue)>);

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
        let apply = move |value: JsValue| {
            match serde_wasm_bindgen::from_value::<DeviceInfo>(value) {
                Ok(snapshot) => set_state.update(|current| {
                    if snapshot.revision >= current.revision {
                        *current = snapshot;
                    }
                }),
                Err(error) => leptos::logging::error!("Could not decode backend state: {error}"),
            }
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
