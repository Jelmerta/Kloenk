FROM nginx:alpine

# Make sure dependency for hiding server is installed
RUN apt-get update && apt-get install -y nginx-extras

# Note: We may leave the older WASM/JS files this way... Probably gonna need to clean this up
ADD dist /usr/share/nginx/html
COPY dist/resources/web/nginx.conf /etc/nginx/conf.d/default.conf
# COPY resources/web/nginx.conf /etc/nginx/conf.d/default.conf
# COPY usr/share/nginx/html/resources

