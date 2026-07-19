FROM lukemathwalker/cargo-chef:latest-rust-1.91-bookworm AS chef
WORKDIR /build

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json -p struct_to_db --bin struct_to_db

COPY . .

RUN cargo build --release --bin struct_to_db -p struct_to_db

FROM debian:bookworm-slim AS runtime

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      ca-certificates \
      rclone \
      curl \
      bash && \
    rm -rf /var/lib/apt/lists/*

RUN useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin scholia

WORKDIR /app
COPY --from=builder /build/target/release/struct_to_db /usr/local/bin/struct_to_db
# The per-corpus import manifest is shared verbatim with local dev
# (`just db <corpus>`), so the job can never drift from the local flow.
COPY scripts/ingest.sh scripts/lib.sh /app/scripts/
COPY jobs/ingest/entrypoint.sh /app/entrypoint.sh

RUN chmod +x /app/entrypoint.sh /app/scripts/ingest.sh
RUN mkdir -p /app/assets && chown -R scholia:scholia /app

USER scholia
ENTRYPOINT ["/app/entrypoint.sh"]
