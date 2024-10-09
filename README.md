# Kloenk

Web Game/Engine written in Rust

Application is available online at https://hatsu.tech

Currently build for web using:
<!-- https://github.com/gfx-rs/wgpu/wiki/Running-on-the-Web-with-WebGPU-and-WebGL -->
<!-- RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --target web -->
<!-- Locally we use trunk to serve the application on the web page: https://trunkrs.dev/ -->
<!-- ``trunk serve'' will serve the application at localhost:8080 -->
<!-- We run ``cargo run'' to run the application as a standalone client. -->
``https://www.rust-lang.org/tools/install``
``rustup update``
For local development, we want to use bacon with clippy instead of building every time. Clippy does not easily allow
building of wasm though...

Notes for host deployment: Make sure firewall allows access on 80/443 for http/https
sudo ufw allow 80
sudo ufw allow 443

sudo ufw allow http
sudo ufw allow https

Useful rust tools to improve project:

- ``cargo build --timings`` produces a report showing crate compile times
- ``cargo update`` to update latest compatible semantic version
- ``cargo fmt`` to format the project
- ``cargo tree`` to see dependency graph. ``cargo tree --duplicate`` can be used to find dependencies with multiple
  versions.
- ``cargo features prune`` to show only the features used by our project. Other features can be disabled
- ``cargo audit`` to scan CVEs
- ``cargo +nightly udeps --all-targets`` to find unused/duplicate dependencies
- ``cargo clippy`` for linting tips, small code improvements
- ``cargo outdated`` or ``cargo outdated --depth 1`` to find new versions of dependencies. Alternatively, cargo-machete
  can be used.
- ``cargo bloat --release -n 100`` optionally with ``--crates`` to figure out functions and dependencies that contribute
  most to binary size
  Other things to consider:
- Switching to other linker may be faster (mold is optimized for linux, lld is an option)
    - ``cargo +nightly rustc --bin kloenk_bin -- -Z time-passes`` to figure out time taken for linker
- Switching to Cranelift for local development, less optimized but produces working executable binaries
- Switching to nightly compiler:
    - ``rustup target add wasm32-unknown-unknown --toolchain nightly``
    - or ``rustup toolchain install nightly --allow-downgrade`` ?
    -
  ``cargo +nightly build -Z build-std=std,panic_abort --target wasm32-unknown-unknown --release --target-dir target --frozen --bin kloenk_bin``
- Look at Bevy's optimizations:
    - https://github.com/bevyengine/bevy/blob/main/.cargo/config_fast_builds.toml

Setting up the server:

- To enter the server: ``ssh root@$ip_adress``
- Initialize using ``initialize_server.sh``

Now we can deploy

- Checking docker containers: ``docker ps -a``
- Logs: ``docker logs $container_name``
- Enter container: ``docker exec -it $container_name /bin/sh``

Local development(linux+standalone client):
cargo watch -x 'run'

Local development(windows+standalone client):
cargo run --target x86_64-pc-windows-msvc
(cargo watch -x 'run --target x86_64-pc-windows-msvc')
(cargo +nightly watch --delay 60 -x 'run --target x86_64-pc-windows-msvc')

Local development(windows+web):
Once:
Run nginx
cp .\resources\web\nginx.conf C:\Users\Jelmer\Downloads\nginx-1.27.2\nginx-1.27.2\conf
cp .\resources\web\common_headers.conf C:\Users\Jelmer\Downloads\nginx-1.27.2\nginx-1.27.2\conf
cp -r .\resources\ C:\Users\Jelmer\Downloads\nginx-1.27.2\nginx-1.27.2\html\
cp .\index.html C:\Users\Jelmer\Downloads\nginx-1.27.2\nginx-1.27.2\html

Every change:
cargo build --target wasm32-unknown-unknown --target-dir target --frozen --bin kloenk_bin
wasm-bindgen target/wasm32-unknown-unknown/debug/kloenk_bin.wasm --target web --out-dir bg_output --out-name kloenk
mv -Force .\bg_output\kloenk_bg.wasm .\bg_output\kloenk.wasm
cp ./bg_output/kloenk.wasm C:\Users\Jelmer\Downloads\nginx-1.27.2\nginx-1.27.2\html\
cp ./bg_output/kloenk.js C:\Users\Jelmer\Downloads\nginx-1.27.2\nginx-1.27.2\html\
