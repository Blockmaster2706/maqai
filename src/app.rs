// Copyright (c) 2026 Blockmaster2706 <wynter@breedable.men>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::sidebar::Sidebar;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[component]
pub fn App() -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (greet_msg, set_greet_msg) = signal(String::new());

    let update_name = move |ev| {
        let v = event_target_value(&ev);
        set_name.set(v);
    };

    let greet = move |ev: SubmitEvent| {
        ev.prevent_default();
        spawn_local(async move {
            let name = name.get_untracked();
            if name.is_empty() {
                return;
            }

            let args = serde_wasm_bindgen::to_value(&GreetArgs { name: &name }).unwrap();
            // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
            let new_msg = invoke("greet", args).await.as_string().unwrap();
            set_greet_msg.set(new_msg);
        });
    };

    view! {
        <main class="container">
            <Sidebar />
            <div class="app-content">
                <p>"Click on the Tauri and Leptos logos to learn more."</p>

                <form class="row" on:submit=greet>
                    <input
                        id="greet-input"
                        placeholder="Enter a name..."
                        on:input=update_name
                    />
                    <button type="submit">"Greet"</button>
                </form>
                <p>{ move || greet_msg.get() }</p>
            </div>
            <footer>
                <span>"Built by Blockmaster2706. "<a href="https://github.com/Blockmaster2706/maqai/blob/main/LICENSE" target="_blank" rel="noopener noreferrer">"License"</a></span>
                <span>"Get Help via "<a href="https://discord.gg/qhwxvqrg2r" target="_blank" rel="noopener noreferrer">"discord"</a>" or "<a href="mailto:wynter@breedable.men" target="_blank" rel="noopener noreferrer">"email"</a>"."</span>
            </footer>
        </main>
    }
}
