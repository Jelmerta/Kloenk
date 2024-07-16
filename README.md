# Kloenk

Surely the start of a new MMORPG (Rust)

Currently build for web using:
<!-- https://github.com/gfx-rs/wgpu/wiki/Running-on-the-Web-with-WebGPU-and-WebGL -->
RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --target web
<!-- Look into https://trunkrs.dev/ as an alternative to wasm-pack -->

Additionally, we can run the project locally by using trunk to serve the application.
``trunk serve'' will serve the application at localhost:8080
