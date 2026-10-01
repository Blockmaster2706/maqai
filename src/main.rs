// Copyright (c) 2026 Blockmaster2706 <wynter@breedable.men>
// SPDX-License-Identifier: GPL-3.0-or-later

mod app;

use app::*;
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}
