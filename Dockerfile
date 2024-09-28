# Add dependencies to dynamically generate nonces in nginx conf (we could manually add dependencies to latest nginx version, openresty could be behind latest)
FROM openresty/openresty:alpine

ARG CACHEBUST=1
RUN touch /etc/nginx/conf.d/default.conf
RUN touch /etc/nginx/conf.d/common_headers.conf

RUN pwd
# Note: We may leave the older WASM/JS files this way... Probably gonna need to clean this up
RUN ls -R /usr/share/nginx/html/resources/web/

RUN pwd

COPY /usr/share/nginx/html/resources/web/nginx.conf /etc/nginx/conf.d/default.conf
COPY /usr/share/nginx/html/resources/web/common_headers.conf /etc/nginx/conf.d/common_headers.conf

RUN ls -R /etc/nginx/conf.d/
