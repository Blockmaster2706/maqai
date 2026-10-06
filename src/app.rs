// Copyright (c) 2026 Blockmaster2706 <wynter@breedable.men>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use leptos::prelude::*;
use leptos_router::{components::*, path};

use crate::{getting_started::GettingStarted, sidebar::Sidebar};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="container">
            <Sidebar />
            <div class="app-content">
                <Router>
                    <Routes fallback=|| "Page not found.">
                        <Route path=path!("/") view=GettingStarted/>
                    </Routes>
                </Router>
            </div>
            <footer>
                <span>"Built by Blockmaster2706. "<a href="https://github.com/Blockmaster2706/maqai/blob/main/LICENSE" target="_blank" rel="noopener noreferrer">"License"</a>", "<a href="https://github.com/Blockmaster2706/maqai" target="_blank" rel="noopener noreferrer">"Source Code"</a></span>
                <span>"Get Help via "<a href="https://discord.gg/qhwxvqrg2r" target="_blank" rel="noopener noreferrer">"discord"</a>" or "<a href="mailto:wynter@breedable.men" target="_blank" rel="noopener noreferrer">"email"</a>"."</span>
            </footer>
        </main>
    }
}
