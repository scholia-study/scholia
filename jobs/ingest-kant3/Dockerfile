FROM lukemathwalker/cargo-chef:latest-rust-1.91-bookworm AS chef
WORKDIR /build

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json -p kant3_struct_to_db --bin kant3_struct_to_db

COPY . .

RUN cargo build --release --bin kant3_struct_to_db -p kant3_struct_to_db

FROM debian:bookworm-slim AS runtime

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      ca-certificates \
      rclone \
      bash && \
    rm -rf /var/lib/apt/lists/*

RUN useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin scholia

WORKDIR /app
COPY --from=builder /build/target/release/kant3_struct_to_db /usr/local/bin/kant3_struct_to_db
COPY jobs/ingest-kant3/ingest_kant3.sh /app/ingest_kant3.sh

RUN chmod +x /app/ingest_kant3.sh
RUN mkdir -p /app/assets && chown -R scholia:scholia /app

USER scholia
ENTRYPOINT ["/app/ingest_kant3.sh"]
