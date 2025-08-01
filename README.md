# Kloenk

Web Game/Engine written in Rust

Application is available online at https://hatsu.tech (probably only on chrome at the moment though, requires webgpu
support)

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
- ``cargo update --verbose`` can also be used to find older dependencies
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
cargo +nightly run -Z build-std=std,panic_abort --target-dir target --bin kloenk_bin --target x86_64-unknown-linux-gnu

Local development(windows+standalone client):
cargo run --target x86_64-pc-windows-msvc
(cargo watch -x 'run --target x86_64-pc-windows-msvc')
(cargo +nightly watch --delay 60 -x 'run --target x86_64-pc-windows-msvc')

Local development(windows+web):
Once:
Run nginx
In top level:
cp -Force .\hatsu-infra\nginx\nginx.conf C:\Users\Jelmer\Downloads\nginx-1.27.2\nginx-1.27.2\conf
cp -Force .\hatsu-infra\nginx\common_headers.conf C:\Users\Jelmer\Downloads\nginx-1.27.2\nginx-1.27.2\conf
cp -r -Force .\games\kloenk\kloenk-client\assets\ C:\Users\Jelmer\Downloads\nginx-1.27.2\nginx-1.27.2\html\assets\
cp .\games\kloenk\kloenk-web\html\index.html C:\Users\Jelmer\Downloads\nginx-1.27.2\nginx-1.27.2\html

Every change:
In client:
cargo build --target wasm32-unknown-unknown --target-dir target --frozen --bin kloenk
wasm-bindgen target/wasm32-unknown-unknown/debug/kloenk.wasm --target web --out-dir bg_output --out-name kloenk
mv -Force .\bg_output\kloenk_bg.wasm .\bg_output\kloenk.wasm
mv ./bg_output/kloenk.wasm C:\Users\Jelmer\Downloads\nginx-1.27.2\nginx-1.27.2\html\
mv ./bg_output/kloenk.js C:\Users\Jelmer\Downloads\nginx-1.27.2\nginx-1.27.2\html\

Local development(linux+web):
sudo systemctl start nginx
cp ./web/nginx/nginx.conf /etc/nginx/nginx.conf
cp ./web/nginx/common_headers.conf /etc/nginx/common_headers.conf
sudo cp -r ./resources /usr/share/nginx/html
sudo cp ./index.html /usr/share/nginx/html

Every change:
``cargo build --target wasm32-unknown-unknown --target-dir target --bin kloenk_bin``
``wasm-bindgen target/wasm32-unknown-unknown/debug/kloenk_bin.wasm --target web --out-dir bg_output --out-name kloenk``
``mv ./bg_output/kloenk_bg.wasm ./bg_output/kloenk.wasm``
``sudo cp bg_output/kloenk.wasm /usr/share/nginx/html``
sudo cp bg_output/kloenk.js /usr/share/nginx/html

Build for product owner (on windows):
```cargo build --target x86_64-pc-windows-msvc --release```
```Compress-Archive -Path .\target\x86_64-pc-windows-msvc\release\assets\,.\target\x86_64-pc-windows-msvc\release\kloenk.exe -DestinationPath .\kloenk.zip```