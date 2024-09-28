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


Useful rust tools to improve project:
- ``cargo tree`` to see dependency graph 
- ``cargo audit`` to scan CVEs 
- ``cargo +nightly udeps --all-targets`` to find unused/duplicate dependencies 
- ``cargo clippy`` for linting tips, small code improvements 
- ``cargo outdated`` or ``cargo outdated --depth 1`` to find new versions of dependencies 
- ``cargo bloat --release -n 100`` optionally with ``--crates`` to figure out functions and dependencies that contribute most to binary size


Setting up the server: 
- To enter the server: ``ssh root@$ip_adress``
- Initialize using ``initialize_server.sh``

Now we can deploy

- Checking docker containers: ``docker ps -a``
- Logs: ``docker logs $container_name``
- Enter container: ``docker exec -it $container_name /bin/sh``
