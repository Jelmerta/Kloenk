FROM rust:1.88 AS rust

RUN rustup target add wasm32-unknown-unknown \
#	&& rustup component add clippy rustfmt \
	&& cargo install wasm-bindgen-cli cargo-audit cargo-chef wasm-opt
WORKDIR /app

FROM rust AS planner
# src folder invalidates all next cache layers. We do not gain speed to remove resulting folders such as audits advisory db or clippy's target folder
# Clippy is being very annoying by running a different rust command and not producing the resulting binary, otherwise we would use this for building as well, related: https://github.com/rust-lang/cargo/issues/8716
# Consider running clippy only on desktop client target
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

#Disable for now to speed up iterations without complaints
#FROM rust AS checker
#COPY . .
#RUN cargo clippy --target wasm32-unknown-unknown --release --target-dir target --locked -- -W clippy::pedantic -W clippy::all -Dwarnings

FROM rust AS auditor
COPY . .
RUN cargo audit

#Temporarily disabled, pipeline has minor difference? Should be checked out
#FROM rust AS formatchecker
#COPY . .
#RUN cargo fmt --all -- --check

FROM rust AS builder
COPY --from=planner /app/recipe.json recipe.json
#https://sharnoff.io/blog/why-rust-compiler-slow, wondering if we even want this. yes it's slow once but then it's just cached anyway
#-Zshare-generics?
RUN RUSTFLAGS='-Cllvm-args=-inline-threshold=10 -Cllvm-args=-inlinedefault-threshold=10 -Cllvm-args=-inlinehint-threshold=10' \
    cargo chef cook --release --recipe-path recipe.json --target wasm32-unknown-unknown --target-dir target

# wasm-opt options: https://manpages.debian.org/testing/binaryen/wasm-opt.1.en.html#enable~4
COPY . .
RUN RUSTFLAGS='-Cllvm-args=-inline-threshold=10 -Cllvm-args=-inlinedefault-threshold=10 -Cllvm-args=-inlinehint-threshold=10' \
    cargo build --target wasm32-unknown-unknown --release --target-dir target --frozen --bin kloenk \
&& wasm-bindgen target/wasm32-unknown-unknown/release/kloenk.wasm --target web --out-dir bg_output --out-name kloenk \
#&& wasm-opt bg_output/kloenk_bg.wasm -o bg_output/kloenk.wasm -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --dce --strip-debug --strip-producers --inlining --coalesce-locals --simplify-locals \ TODO tmp disabled for profiling
#&& wasm-opt bg_output/kloenk_bg.wasm -o bg_output/kloenk.wasm -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --dce --strip-producers --inlining --coalesce-locals --simplify-locals \
&& wasm-opt bg_output/kloenk_bg.wasm -o bg_output/kloenk.wasm -Oz -g --enable-bulk-memory --enable-nontrapping-float-to-int \
&& mkdir output \
&& cp ./bg_output/kloenk.js output/kloenk.js \
&& cp ./bg_output/kloenk.wasm output/kloenk.wasm

FROM debian:bookworm-slim

# Force stages to be run
#COPY --from=checker /etc/hostname /dev/null
COPY --from=auditor /etc/hostname /dev/null
#COPY --from=formatchecker /etc/hostname /dev/null

COPY --from=builder /app/output/ /app/output/
#COPY assets /usr/share/nginx/html/assets


# TODO serve files
#no we do this just through infra?

