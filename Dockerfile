FROM nginx:alpine

# Note: We may leave the older WASM/JS files this way... Probably gonna need to clean this up
ADD dist /usr/share/nginx/html

# Might need this when setting up new server?
#COPY build/resources/web/nginx.conf /etc/nginx/conf.d/default.conf
