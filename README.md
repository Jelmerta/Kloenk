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
For local development, we want to use bacon with clippy instead of building every time. Clippy does not easily allow building of wasm though...

Notes for host deployment: Make sure firewall allows access on 80/443 for http/https
sudo ufw allow 80
sudo ufw allow 443

sudo ufw allow http
sudo ufw allow https


Useful rust tools to improve project:
- ``cargo build --timings`` produces a report showing crate compile times
- ``cargo update`` to update latest compatible semantic version
- ``cargo fmt`` to format the project
- ``cargo tree`` to see dependency graph. ``cargo tree --duplicate`` can be used to find dependencies with multiple versions. 
- ``cargo features prune`` to show only the features used by our project. Other features can be disabled
- ``cargo audit`` to scan CVEs 
- ``cargo +nightly udeps --all-targets`` to find unused/duplicate dependencies 
- ``cargo clippy`` for linting tips, small code improvements 
- ``cargo outdated`` or ``cargo outdated --depth 1`` to find new versions of dependencies. Alternatively, cargo-machete can be used. 
- ``cargo bloat --release -n 100`` optionally with ``--crates`` to figure out functions and dependencies that contribute most to binary size
Other things to consider:
- Switching to other linker may be faster (mold is optimized for linux, lld is an option)
    - ``cargo +nightly rustc --bin kloenk_bin -- -Z time-passes`` to figure out time taken for linker
- Switching to Cranelift for local development, less optimized but produces working executable binaries
- Switching to nightly compiler:
    - ``rustup target add wasm32-unknown-unknown --toolchain nightly``
    - ``cargo +nightly build -Z build-std=std,panic_abort --target wasm32-unknown-unknown --release --target-dir target --frozen --bin kloenk_bin``

Setting up the server: 
- To enter the server: ``ssh root@$ip_adress``
- Initialize using ``initialize_server.sh``

Now we can deploy

- Checking docker containers: ``docker ps -a``
- Logs: ``docker logs $container_name``
- Enter container: ``docker exec -it $container_name /bin/sh``
