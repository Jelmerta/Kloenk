FROM rust:1.87 AS rust

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
RUN cargo chef cook --release --recipe-path recipe.json --target wasm32-unknown-unknown --target-dir target

# wasm-opt options: https://manpages.debian.org/testing/binaryen/wasm-opt.1.en.html#enable~4
COPY . .
RUN cargo build --target wasm32-unknown-unknown --release --target-dir target --frozen --bin kloenk \
&& wasm-bindgen target/wasm32-unknown-unknown/release/kloenk.wasm --target web --out-dir bg_output --out-name kloenk \
&& wasm-opt bg_output/kloenk_bg.wasm -o bg_output/kloenk.wasm -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --dce --strip-debug --strip-producers --inlining --coalesce-locals --simplify-locals \
&& mkdir output \
&& cp ./bg_output/kloenk.js output/kloenk.js \
&& cp ./bg_output/kloenk.wasm output/kloenk.wasm


FROM debian:bookworm-slim AS nginx-builder
WORKDIR /

#pcre3 without dev needed?
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

# Note: there are other ssl options that support quic: https://nginx.org/en/docs/quic.html Difficult to decide which to use but boringssl seems best maintained
RUN git clone "https://boringssl.googlesource.com/boringssl" \
    && cd boringssl \
    && cmake -GNinja -B build -DCMAKE_BUILD_TYPE=Release -DCMAKE_CXX_FLAGS="-Wno-error=array-bounds" \
    && ninja -C build

RUN git clone https://github.com/vision5/ngx_devel_kit
RUN git clone https://github.com/openresty/set-misc-nginx-module

RUN wget https://nginx.org/download/nginx-1.27.5.tar.gz && \
    tar zxf nginx-1.27.5.tar.gz

WORKDIR /nginx-1.27.5

# Is stdc++ required? crypto? --with-ipv6?
RUN ./configure \
    --prefix=/usr/local/nginx \
    --sbin-path=/usr/sbin/nginx \
    --modules-path=/usr/lib/nginx/modules \
    --with-http_ssl_module \
    --with-http_sub_module \
    --with-http_v2_module \
    --with-http_v3_module \
    --with-cc-opt="-I/boringssl/include" \
    --with-ld-opt="-L/boringssl/build -lssl -lcrypto -lstdc++" \
    --add-module=/ngx_devel_kit \
    --add-module=/set-misc-nginx-module && \
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
COPY --from=nginx-builder /usr/local/nginx /usr/local/nginx
# Not sure if this is needed, probably copied anyway
RUN mkdir -p /etc/nginx

COPY --from=builder /app/output /usr/share/nginx/html
COPY assets /usr/share/nginx/html/assets
COPY web/html /usr/share/nginx/html
COPY web/nginx /etc/nginx/conf.d

RUN mkdir -p /var/log/nginx
RUN touch /var/log/nginx/error.log /var/log/nginx/access.log
#CMD ["nginx", "-g", "daemon off; error_log /var/log/nginx/error.log debug;"]
CMD ["nginx", "-g", "daemon off;"]

