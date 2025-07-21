// Makes sure Windows does not open terminal on release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod run;
mod application;
mod render;
mod state;
mod systems;
mod gui;

fn main() {
    run::run();
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen(start)]
    pub fn run() {
        crate::run::run();
    }
}