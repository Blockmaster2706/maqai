// Copyright (c) 2026 Blockmaster2706 <wynter@breedable.men>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use maqai_lib::adb;

fn main() {

    const ADB_COMMAND: &str = "devices";
    match adb::run_adb_command(ADB_COMMAND) {
        Ok(output) => println!("ADB Output: {}", output),
        Err(e) => eprintln!("ADB Error: {}", e),
    }

    maqai_lib::run()
}
