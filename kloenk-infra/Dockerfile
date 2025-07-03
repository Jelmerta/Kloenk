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
# we got the cc/ld flags from https://github.com/google/ngx_brotli, we might want to change these as these might just be meant for brotli performance
# Do we not want keepalive?
# probably want real_ip with cloudflare to see where connections are coming from
RUN ./configure \
    --prefix=/usr/share/nginx \
    --sbin-path=/usr/sbin/nginx \
    --modules-path=/usr/lib/nginx/modules \
    --without-http_autoindex_module \
    --without-http_ssi_module \
    --without-http_access_module \
    --without-http_auth_basic_module \
    --without-http_charset_module \
    --without-http_mirror_module \
    --without-http_empty_gif_module \
    --without-http_upstream_hash_module \
    --without-http_upstream_ip_hash_module \
    --without-http_upstream_least_conn_module \
    --without-http_upstream_random_module \
    --without-http_upstream_keepalive_module \
    --without-http_upstream_zone_module \
    --without-http_grpc_module \
    --without-http_scgi_module \
    --without-http_uwsgi_module \
    --without-http_fastcgi_module \
    --without-http_proxy_module \
    --without-http_split_clients_module \
    --without-http_map_module \
    --without-http_geo_module \
    --without-http_userid_module \
    --without-select_module \
    --without-poll_module \
    --without-mail_pop3_module \
    --without-mail_imap_module \
    --without-mail_smtp_module \
    --without-stream_limit_conn_module \
    --without-stream_access_module \
    --without-stream_geo_module \
    --without-stream_map_module \
    --without-stream_split_clients_module \
    --without-stream_return_module \
    --without-stream_set_module \
    --without-stream_upstream_hash_module \
    --without-stream_upstream_least_conn_module \
    --without-stream_upstream_random_module \
    --without-stream_upstream_zone_module \
    --with-http_ssl_module \
    --with-http_v2_module \
    --with-http_v3_module \
    --with-ipv6 \
    --with-http_gzip_static_module \
    --with-http_gunzip_module \
    --with-threads \
    --with-cc-opt="-I/boringssl/include -m64 -march=native -mtune=native -Ofast -flto -funroll-loops -ffunction-sections -fdata-sections -Wl,--gc-sections" \
    --with-ld-opt="-L/boringssl/build -lssl -lcrypto -lstdc++ -m64 -Wl,-s -Wl,-Bsymbolic -Wl,--gc-sections" \
    --add-module=/ngx_devel_kit \
    --add-module=/set-misc-nginx-module \
    --add-module=/ngx_brotli && \
    make -j4 && \
    make install

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libpcre3

COPY --from=nginx-builder /usr/sbin/nginx /usr/sbin/nginx
COPY --from=nginx-builder /usr/share/nginx /usr/share/nginx

COPY nginx /usr/share/nginx/conf
RUN rm -rf /usr/share/nginx/html/

CMD ["nginx", "-g", "daemon off;"]