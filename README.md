# Kloenk

Web Game/Engine written in Rust

Application is available online at https://hatsu.tech

Currently build for web using:
<!-- https://github.com/gfx-rs/wgpu/wiki/Running-on-the-Web-with-WebGPU-and-WebGL -->
RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --target web
Locally we use trunk to serve the application on the web page: https://trunkrs.dev/
``trunk serve'' will serve the application at localhost:8080
We run ``cargo run'' to run the application as a standalone client.

Notes for host deployment: Make sure firewall allows access on 80/443 for http/https
sudo ufw allow 80
sudo ufw allow 443

sudo ufw allow http
sudo ufw allow https

