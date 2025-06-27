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
&& wasm-opt bg_output/kloenk_bg.wasm -o bg_output/kloenk.wasm -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --dce --strip-debug --strip-producers --inlining --coalesce-locals --simplify-locals \
&& mkdir output \
&& cp ./bg_output/kloenk.js output/kloenk.js \
&& cp ./bg_output/kloenk.wasm output/kloenk.wasm

FROM debian:bookworm-slim AS nginx-builder
WORKDIR /

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    git \
    wget \
    ca-certificates \
    cmake \
    ninja-build \
    libpcre3-dev \
    zlib1g-dev

RUN git clone "https://boringssl.googlesource.com/boringssl" \
    && cd boringssl \
    && cmake -GNinja -B build -DCMAKE_BUILD_TYPE=Release -DCMAKE_CXX_FLAGS="-Wno-error=array-bounds" \
    && ninja -C build

RUN git clone https://github.com/vision5/ngx_devel_kit
RUN git clone https://github.com/openresty/set-misc-nginx-module

RUN git clone --recurse-submodules -j8 https://github.com/google/ngx_brotli && \
    cd ngx_brotli/deps/brotli && \
    mkdir out && cd out && \
    cmake -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=OFF -DCMAKE_C_FLAGS="-Ofast -m64 -march=native -mtune=native -flto -funroll-loops -ffunction-sections -fdata-sections -Wl,--gc-sections" -DCMAKE_CXX_FLAGS="-Ofast -m64 -march=native -mtune=native -flto -funroll-loops -ffunction-sections -fdata-sections -Wl,--gc-sections" -DCMAKE_INSTALL_PREFIX=./installed .. && \
    cmake --build . --config Release --target brotlienc

RUN wget https://nginx.org/download/nginx-1.29.0.tar.gz && \
    tar zxf nginx-1.29.0.tar.gz

WORKDIR /nginx-1.29.0

# Is stdc++ required? crypto? --with-ipv6?
RUN export CFLAGS="-m64 -march=native -mtune=native -Ofast -flto -funroll-loops -ffunction-sections -fdata-sections -Wl,--gc-sections" && \
    export LDFLAGS="-m64 -Wl,-s -Wl,-Bsymbolic -Wl,--gc-sections" && \
    ./configure \
    --prefix=/usr/share/nginx \
    --sbin-path=/usr/sbin/nginx \
    --modules-path=/usr/lib/nginx/modules \
    --without-mail_pop3_module \
    --without-mail_imap_module \
    --without-mail_smtp_module \
    --with-http_ssl_module \
    --with-http_sub_module \
    --with-http_v2_module \
    --with-http_v3_module \
    --with-ipv6 \
    --with-http_gzip_static_module \
    --with-http_gunzip_module \
    --with-cc-opt="-I/boringssl/include" \
    --with-ld-opt="-L/boringssl/build -lssl -lcrypto -lstdc++" \
    --add-module=/ngx_devel_kit \
    --add-module=/set-misc-nginx-module \
    --add-module=/ngx_brotli && \
    make -j4 && \
    make install

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libpcre3

# Force stages to be run
#COPY --from=checker /etc/hostname /dev/null
COPY --from=auditor /etc/hostname /dev/null
#COPY --from=formatchecker /etc/hostname /dev/null

COPY --from=nginx-builder /usr/sbin/nginx /usr/sbin/nginx
COPY --from=nginx-builder /usr/share/nginx /usr/share/nginx

COPY --from=builder /app/output /usr/share/nginx/html
COPY assets /usr/share/nginx/html/assets
COPY web/nginx /usr/share/nginx/conf
COPY web/html /usr/share/nginx/html

CMD ["nginx", "-g", "daemon off;"]

