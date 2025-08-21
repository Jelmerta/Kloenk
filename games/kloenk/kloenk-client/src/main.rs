// Makes sure Windows does not open terminal on release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod application;
mod gui;
mod render;
mod run;
mod state;
mod systems;

fn main() {
    run::run();
}

#[cfg(target_family = "wasm")]
mod wasm {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen(start)]
    pub fn run() {
        crate::run::run();
    }
}
