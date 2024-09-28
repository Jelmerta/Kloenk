# Add dependencies to dynamically generate nonces in nginx conf (we could manually add dependencies to latest nginx version, openresty could be behind latest)
FROM openresty/openresty:alpine

# Note: We may leave the older WASM/JS files this way... Probably gonna need to clean this up
ADD dist /usr/share/nginx/html
COPY dist/resources/web/nginx.conf /etc/nginx/conf.d/default.conf
COPY dist/resources/web/common_headers.conf /etc/nginx/conf.d/common_headers.conf

