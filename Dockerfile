FROM rust:1.81 AS rust

# Add empty project such that dependencies can be built without requiring src code
# RUN cargo new --bin app
RUN rustup target add wasm32-unknown-unknown \
	&& rustup component add clippy rustfmt \
	&& cargo install cargo-audit cargo-chef wasm-bindgen-cli wasm-opt
WORKDIR /app

FROM rust AS planner
# COPY Cargo.toml Cargo.lock ./
# COPY src src
COPY . .
RUN cargo audit \
&& cargo fmt --all -- --check \
&& cargo chef prepare --recipe-path recipe.json \
&& rm -rf /usr/local/cargo/advisory-db*
# ^ We remove the db containing audit advice as otherwise a big cache layer is introduced.

FROM rust AS builder
COPY --from=planner /app/recipe.json recipe.json
# clippy option?
RUN cargo chef cook --release --recipe-path recipe.json --target wasm32-unknown-unknown --target-dir target


# Setup # only if web , for we docker layers we might wanna split up as late as possible? though adding this target has no real downsides anyway

# Check dependencies
# COPY Cargo.toml Cargo.lock ./
# && cargo fetch --locked surely this is already done in chef

# Build just the dependencies
# RUN cargo build --target wasm32-unknown-unknown --release --target-dir target --frozen || true 

# Verify source & build binaries
# COPY src src
COPY . .
# && cargo clippy --release --all-targets --all-features --frozen -- -Dwarnings \
RUN cargo build --target wasm32-unknown-unknown --release --target-dir target --frozen --bin kloenk_bin \
&& wasm-bindgen target/wasm32-unknown-unknown/release/kloenk_bin.wasm --target web --out-dir bg_output --out-name kloenk \
&& wasm-opt bg_output/kloenk_bg.wasm -o bg_output/kloenk.wasm -Oz --dce --strip-debug --strip-producers --inlining --coalesce-locals --simplify-locals \
&& mkdir output \
&& cp ./bg_output/kloenk.js output/kloenk.js \
&& cp ./bg_output/kloenk.wasm output/kloenk.wasm

# does html need to be changed or directly copied to deploy server?
# COPY ./index.html output/index.html 

# ENTRYPOINT ["/bin/bash", "-l", "-c"]

# Deploy
# FROM alpine:3.20
FROM openresty/openresty:alpine
COPY ./index.html /usr/share/nginx/html/index.html
COPY --from=builder /app/output /usr/share/nginx/html
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
