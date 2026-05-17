FROM lukemathwalker/cargo-chef:latest-rust-1.91-bookworm AS chef
WORKDIR /build

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Cooks deps from recipe.json into a cached layer, then copies the
# real source on top. -p api --bin api scopes the cook to the api
# crate's transitive deps so kant1_*/bible_to_db crates (developer-only
# ingest CLIs) don't pull their deps into this image.
FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json -p api --bin api

COPY . .

# sqlx::query! macros validate against .sqlx/query-*.json instead of
# opening a DB connection. Build fails loudly if .sqlx/ is missing or
# stale relative to source — run `pnpm db:prepare` and commit the diff.
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin api -p api

FROM debian:bookworm-slim AS runtime

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      ca-certificates \
      libssl3 && \
    rm -rf /var/lib/apt/lists/*

RUN useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin scholia

WORKDIR /app
COPY --from=builder /build/target/release/api /usr/local/bin/api

USER scholia
EXPOSE 4000
ENTRYPOINT ["/usr/local/bin/api"]
