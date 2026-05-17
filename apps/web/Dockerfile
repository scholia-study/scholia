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
RUN pnpm --filter @apps/web build

FROM base AS prod-deps
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY apps/web/package.json apps/web/
COPY apps/api/package.json apps/api/
COPY apps/proxy/package.json apps/proxy/
RUN pnpm install --frozen-lockfile --filter @apps/web... --ignore-scripts --prod

FROM node:22-alpine AS runtime
ENV NODE_ENV=production PORT=3000

WORKDIR /app

COPY --from=prod-deps /build/node_modules ./node_modules
COPY --from=prod-deps /build/apps/web/node_modules ./apps/web/node_modules
COPY --from=builder /build/apps/web/dist ./apps/web/dist
COPY --from=builder /build/apps/web/package.json ./apps/web/package.json

USER node
WORKDIR /app/apps/web
EXPOSE 3000

CMD ["./node_modules/.bin/srvx", "serve", "--prod", "--entry", "./dist/server/server.js", "--static", "../client", "--port", "3000"]
