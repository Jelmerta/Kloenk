FROM rust:1.81 AS rust

RUN rustup target add wasm32-unknown-unknown \
	&& rustup component add clippy rustfmt \
	&& cargo install cargo-audit cargo-chef wasm-bindgen-cli wasm-opt
WORKDIR /app

FROM rust AS planner
# src folder invalidates all next cache layers. We do not gain speed to remove resulting folders such as audits advisory db or clippy's target folder
# Clippy is being very annoying by running a different rust command and not producing the resulting binary, otherwise we would use this for building as well, related: https://github.com/rust-lang/cargo/issues/8716
# Consider running clippy only on desktop client target
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust AS checker
COPY . .
RUN cargo clippy --target wasm32-unknown-unknown --release --target-dir target --locked -- -Dwarnings

FROM rust AS auditor
COPY . .
RUN cargo audit

FROM rust AS formatchecker
COPY . .
RUN cargo fmt --all -- --check

FROM rust AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --target wasm32-unknown-unknown --target-dir target

COPY . .
RUN cargo build --target wasm32-unknown-unknown --release --target-dir target --frozen --bin kloenk_bin \
&& wasm-bindgen target/wasm32-unknown-unknown/release/kloenk_bin.wasm --target web --out-dir bg_output --out-name kloenk \
&& wasm-opt bg_output/kloenk_bg.wasm -o bg_output/kloenk.wasm -Oz --dce --strip-debug --strip-producers --inlining --coalesce-locals --simplify-locals \
&& mkdir output \
&& cp ./bg_output/kloenk.js output/kloenk.js \
&& cp ./bg_output/kloenk.wasm output/kloenk.wasm

#nginx:alpine does not include required nginx sub_filter dependencies
FROM openresty/openresty:alpine
# Force stages to be run
COPY --from=checker /etc/hostname /dev/null
COPY --from=auditor /etc/hostname /dev/null
COPY --from=formatchecker /etc/hostname /dev/null

COPY ./index.html /usr/share/nginx/html/index.html
COPY --from=builder /app/output /usr/share/nginx/html
COPY ./resources /usr/share/nginx/html/resources
COPY ./resources/web/nginx.conf /etc/nginx/conf.d/default.conf
COPY ./resources/web/common_headers.conf /etc/nginx/conf.d/common_headers.conf
