// Copyright (c) 2026 Blockmaster2706 <wynter@breedable.men>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
struct GettingStartedFlow {
    initial_setup_complete: Option<bool>,
    developer_mode_enabled: Option<bool>,
    adb_enabled: Option<bool>,
    flow_completed: bool,
}

#[component]
pub fn GettingStarted() -> impl IntoView {
    let (flow, set_flow) = signal(GettingStartedFlow::default());

    let answer_initial_setup = move |answer| {
        set_flow.update(|flow| {
            flow.initial_setup_complete = Some(answer);
            flow.developer_mode_enabled = None;
            flow.adb_enabled = None;
            flow.flow_completed = false;
        });
    };

    let answer_developer_mode = move |answer| {
        set_flow.update(|flow| {
            flow.developer_mode_enabled = Some(answer);
            flow.adb_enabled = None;
            flow.flow_completed = flow.initial_setup_complete == Some(true) && answer;
        });
    };

    let answer_adb = move |answer| {
        set_flow.update(|flow| {
            flow.adb_enabled = Some(answer);
            flow.flow_completed = flow.initial_setup_complete == Some(true) && answer;
        });
    };

    view!(
        <form class="getting-started-flow" class:flow-completed=move || flow.get().flow_completed on:submit=move |ev: SubmitEvent| {
                ev.prevent_default();
            }>
                <h1>"Getting Started"</h1>
                    <fieldset class="question-group">
                        <legend>"Did you complete the initial setup?"</legend>
                        <div class="answer-options">
                            <label>
                                <input type="radio" name="initial-setup" value="yes"
                                    prop:checked=move || flow.get().initial_setup_complete == Some(true)
                                    on:change=move |_| answer_initial_setup(true) />
                                "Yes"
                            </label>
                            <label>
                                <input type="radio" name="initial-setup" value="no"
                                    prop:checked=move || flow.get().initial_setup_complete == Some(false)
                                    on:change=move |_| answer_initial_setup(false) />
                                "No"
                            </label>
                        </div>
                    </fieldset>
                    <Show when=move || flow.get().initial_setup_complete == Some(false)>
                        <p>"Please join our Discord for help with skipping the initial setup."</p>
                    </Show>
                    <Show when=move || flow.get().initial_setup_complete == Some(true)>
                        <fieldset class="question-group">
                            <legend>"Did you enable developer mode?"</legend>
                            <div class="answer-options">
                                <label>
                                    <input type="radio" name="developer-mode" value="yes"
                                        prop:checked=move || flow.get().developer_mode_enabled == Some(true)
                                        on:change=move |_| answer_developer_mode(true) />
                                    "Yes"
                                </label>
                                <label>
                                    <input type="radio" name="developer-mode" value="no"
                                        prop:checked=move || flow.get().developer_mode_enabled == Some(false)
                                        on:change=move |_| answer_developer_mode(false) />
                                    "No"
                                </label>
                            </div>
                        </fieldset>
                    </Show>
                    <Show when=move || flow.get().developer_mode_enabled == Some(false)>
                        <fieldset class="question-group">
                            <legend>"Did you enable ADB another way (i.e. SideQuest)?"</legend>
                            <div class="answer-options">
                                <label>
                                    <input type="radio" name="adb-enabled" value="yes"
                                        prop:checked=move || flow.get().adb_enabled == Some(true)
                                        on:change=move |_| answer_adb(true) />
                                    "Yes"
                                </label>
                                <label>
                                    <input type="radio" name="adb-enabled" value="no"
                                        prop:checked=move || flow.get().adb_enabled == Some(false)
                                        on:change=move |_| answer_adb(false) />
                                    "No"
                                </label>
                            </div>
                        </fieldset>
                        <Show when=move || flow.get().adb_enabled == Some(false)>
                            <p>"Maqai currently works only with Devices that already have ADB enabled. Please use "<a href="https://guides.mxr.lol/how-to-root-without-dev-mode/" target="_blank" rel="noopener noreferrer">"this guide"</a> " instead or ask in the Discord for help."</p>
                        </Show>
                    </Show>
                </form>
                <Show when=move || flow.get().flow_completed>
                    <crate::rooting::RootingFlow />
                </Show>
    )
}
