FROM rust:1.81

# Add empty project such that dependencies can be built without requiring src code
RUN cargo new --bin app
WORKDIR /app

# Setup # only if web , for we docker layers we might wanna split up as late as possible? though adding this target has no real downsides anyway
RUN rustup target add wasm32-unknown-unknown \
	&& rustup component add clippy rustfmt \
	&& cargo install cargo-audit wasm-bindgen-cli wasm-opt --locked

# Check dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch \
&& cargo audit 

# Build just the dependencies
RUN cargo build --release || true 

# Verify source & build binaries
COPY src src
RUN cargo fmt --all -- --check \
&& cargo clippy --all-targets --all-features -- -Dwarnings \
&& cargo build --target wasm32-unknown-unknown --release --locked --target-dir target \
&& wasm-bindgen target/wasm32-unknown-unknown/release/kloenk_bin.wasm --target web --out-dir bg_output --out-name kloenk \
&& wasm-opt bg_output/kloenk_bg.wasm -o bg_output/kloenk.wasm -Oz --dce --strip-debug --strip-producers --inlining --coalesce-locals --simplify-locals \
&& mkdir output \
&& cp ./bg_output/kloenk.js output/kloenk.js \
&& cp ./bg_output/kloenk.wasm output/kloenk.wasm

# does html need to be changed or directly copied to deploy server?
# COPY ./index.html output/index.html 

# ENTRYPOINT ["/bin/bash", "-l", "-c"]

# FROM alpine:3.20
FROM openresty/openresty:alpine
COPY ./index.html /usr/share/nginx/html/index.html
COPY --from=0 /usr/src/app/output /usr/share/nginx/html
COPY ./resources /usr/share/nginx/html/resources
COPY ./resources/web/nginx.conf /etc/nginx/conf.d/default.conf
COPY ./resources/web/common_headers.conf /etc/nginx/conf.d/common_headers.conf



# Build deployable container
# RUN docker-compose up -d

# Deploy application
# Should contain details on how:
# To copy over required files
# to run the server
# Server configuratoin such that it is exposed
# Consider bacon instead of cargo watch?
