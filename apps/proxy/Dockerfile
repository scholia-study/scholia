# Scholia edge proxy / HTTP cache.
#
# Sits in front of Node SSR (apps/web) and Rust API (apps/api). Caches
# HTML and anonymous API responses; routes authenticated requests through
# to upstreams; exposes a cluster-internal :8080 listener for PURGE
# requests issued by Rust after content writes.

# --- Stage 1: compile ngx_cache_purge against the runtime nginx version ---
#
# Stock nginx:alpine doesn't include ngx_cache_purge. We build it as a
# dynamic module here so the runtime layer can load it via `load_module`.
# Using the same nginx image as the base ensures the module ABI matches.
FROM nginx:1.27-alpine AS modules

RUN apk add --no-cache \
    build-base linux-headers \
    pcre-dev zlib-dev openssl-dev \
    gd-dev geoip-dev libxslt-dev perl-dev \
    git wget

WORKDIR /tmp/build

# `nginx -V` reports both the version and the full configure args used to
# build the binary in this image. We re-use both: the source must match
# the binary version, and the configure args must match so the dynamic
# module's signature is compatible.
RUN NGX_VERSION=$(nginx -v 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+') \
    && NGX_CONFIG_ARGS=$(nginx -V 2>&1 | grep -F 'configure arguments:' | sed -e 's/^.*configure arguments: //') \
    && wget -q "https://nginx.org/download/nginx-${NGX_VERSION}.tar.gz" -O nginx.tar.gz \
    && tar -xzf nginx.tar.gz \
    && git clone --depth 1 https://github.com/nginx-modules/ngx_cache_purge.git \
    && cd "nginx-${NGX_VERSION}" \
    && eval ./configure ${NGX_CONFIG_ARGS} --add-dynamic-module=../ngx_cache_purge \
    && make -j"$(nproc)" modules \
    && mkdir -p /modules \
    && cp objs/ngx_http_cache_purge_module.so /modules/

# --- Stage 2: runtime ---
FROM nginx:1.27-alpine

COPY --from=modules /modules/ngx_http_cache_purge_module.so /usr/lib/nginx/modules/

# Cache directories — back this with a PersistentVolume in k3s so the
# cache survives pod restarts.
RUN mkdir -p /var/cache/nginx/html /var/cache/nginx/api

# Drop-in directory for per-environment server-scope overrides
# (auth_basic, X-Robots-Tag, location carve-outs). The default.conf
# template `include`s this dir at server scope; overlays mount a
# ConfigMap with their fragment files here. Empty by default so the
# base image is prod-ready (public, no auth).
RUN mkdir -p /etc/nginx/conf.d/security

# Main nginx.conf override — needed to issue `load_module` before the
# http {} block.
COPY nginx.conf /etc/nginx/nginx.conf

COPY conf.d/ /etc/nginx/conf.d/

# Templates are rendered by the official entrypoint's envsubst step at
# container start: /etc/nginx/templates/*.template → /etc/nginx/conf.d/*.
COPY templates/ /etc/nginx/templates/

# Restrict envsubst to the variables we actually template. Without this
# nginx's own `$request_uri`, `$host`, etc. would be substituted to empty
# strings and silently break the config.
ENV NGINX_ENVSUBST_FILTER="^UPSTREAM_(WEB|API)$"

EXPOSE 80 8080
