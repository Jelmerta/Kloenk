FROM nginx:alpine

RUN apk add nginx-plus-module-set-misc # Add dependencies to dynamically generate nonces in nginx conf

# Note: We may leave the older WASM/JS files this way... Probably gonna need to clean this up
ADD dist /usr/share/nginx/html
COPY dist/resources/web/nginx.conf /etc/nginx/conf.d/default.conf

