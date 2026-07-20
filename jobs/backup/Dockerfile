# Daily DB-backup image: the postgres client (pg_dump, matched to the
# server's major version) plus rclone to push the dump to Hetzner Object
# Storage. Deliberately FROM postgres:18 so pg_dump can never lag the
# server — bump both together. Build context is the repo root (needs
# scripts/lib.sh for the shared rclone config).
FROM postgres:18

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      ca-certificates \
      rclone \
      curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY scripts/lib.sh /app/scripts/lib.sh
COPY jobs/backup/backup_db.sh /app/backup_db.sh
RUN chmod +x /app/backup_db.sh

ENTRYPOINT ["/app/backup_db.sh"]
