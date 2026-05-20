FROM lukemathwalker/cargo-chef:latest-rust-1.91-bookworm AS chef
WORKDIR /build

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json -p bible_to_db --bin bible_to_db

COPY . .

RUN cargo build --release --bin bible_to_db -p bible_to_db

FROM debian:bookworm-slim AS runtime

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      ca-certificates \
      rclone \
      bash && \
    rm -rf /var/lib/apt/lists/*

RUN useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin scholia

WORKDIR /app
COPY --from=builder /build/target/release/bible_to_db /usr/local/bin/bible_to_db
COPY jobs/ingest-bible/ingest_bible.sh /app/ingest_bible.sh

RUN chmod +x /app/ingest_bible.sh
# Ensure the assets dir exists
RUN mkdir -p /app/assets && chown -R scholia:scholia /app

USER scholia
ENTRYPOINT ["/app/ingest_bible.sh"]
