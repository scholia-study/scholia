FROM node:22-alpine AS base
RUN corepack enable
WORKDIR /build

FROM base AS deps
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY apps/web/package.json apps/web/
COPY apps/api/package.json apps/api/
COPY apps/proxy/package.json apps/proxy/
RUN pnpm install --frozen-lockfile --filter @apps/web... --ignore-scripts

FROM deps AS builder
COPY apps/web apps/web
# SENTRY_RELEASE (main-<sha7>, from CI) stamps the bundles so runtime
# events match the uploaded source maps; the upload token arrives as a
# BuildKit secret (never a layer). Both absent locally: the sentry vite
# plugin then skips uploading and the build stays green.
ARG SENTRY_RELEASE
RUN --mount=type=secret,id=sentry_auth_token \
    SENTRY_AUTH_TOKEN="$(cat /run/secrets/sentry_auth_token 2>/dev/null || true)" \
    pnpm --filter @apps/web build

FROM base AS prod-deps
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY apps/web/package.json apps/web/
COPY apps/api/package.json apps/api/
COPY apps/proxy/package.json apps/proxy/
RUN pnpm install --frozen-lockfile --filter @apps/web... --ignore-scripts --prod

FROM node:22-alpine AS runtime
ENV NODE_ENV=production PORT=3000
# Sentry-SDK SSR instrumentation: --import loads it before the server
# entry (the build script copied it into dist/server). Inert until the
# Deployment provides SENTRY_DSN. SENTRY_RELEASE identifies this build
# in events from any environment (one image, dev→prod promotion).
ARG SENTRY_RELEASE
ENV SENTRY_RELEASE=$SENTRY_RELEASE \
    NODE_OPTIONS="--import /app/apps/web/dist/server/instrument.server.mjs"

WORKDIR /app

COPY --from=prod-deps /build/node_modules ./node_modules
COPY --from=prod-deps /build/apps/web/node_modules ./apps/web/node_modules
COPY --from=builder /build/apps/web/dist ./apps/web/dist
COPY --from=builder /build/apps/web/package.json ./apps/web/package.json

USER node
WORKDIR /app/apps/web
EXPOSE 3000

CMD ["./node_modules/.bin/srvx", "serve", "--prod", "--entry", "./dist/server/server.js", "--static", "../client", "--port", "3000"]
