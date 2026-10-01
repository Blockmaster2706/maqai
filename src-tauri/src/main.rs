// Copyright (c) 2026 Blockmaster2706 <wynter@breedable.men>
// SPDX-License-Identifier: GPL-3.0-or-later

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    maqai_lib::run()
}
