# Scholia edge proxy / HTTP cache.
#
# Sits in front of Node SSR (apps/web) and Rust API (apps/api). Caches
# HTML and anonymous API responses; routes authenticated requests through
# to upstreams; exposes a cluster-internal :8080 listener for PURGE.
#
# NOTE: stock nginx does NOT ship the ngx_cache_purge module. When we wire
# up PURGE in step 4 of PLAN_3_TIER, switch to an image that has it
# (custom build / openresty / nginx-extras) or implement file-based
# invalidation in the Rust cache helper.

FROM nginx:1.27-alpine

# Cache directories — back this with a PersistentVolume in k3s so the
# cache survives pod restarts.
RUN mkdir -p /var/cache/nginx/html /var/cache/nginx/api

COPY conf.d/ /etc/nginx/conf.d/

# Templates are rendered by the official entrypoint's envsubst step at
# container start: /etc/nginx/templates/*.template → /etc/nginx/conf.d/*.
COPY templates/ /etc/nginx/templates/

# Restrict envsubst to the variables we actually template. Without this
# nginx's own `$request_uri`, `$host`, etc. would be substituted to empty
# strings and silently break the config.
ENV NGINX_ENVSUBST_FILTER="^(UPSTREAM_(WEB|API)|APP_PROFILE)$"

EXPOSE 80 8080
