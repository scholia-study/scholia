# Scholia operational tasks — `just --list` to browse.

set shell := ["bash", "-uc"]

# list available recipes
default:
    @just --list --unsorted

# regenerate a corpus's struct JSON(s) from curated MD (kant1|kant3|shakespeare1|milton1|ibsen1)
[group("corpus")]
struct corpus:
    bash scripts/struct.sh {{ corpus }}

# import a corpus's struct JSON(s) into Postgres; flags reach the importer (e.g. --dry-run, --database-url …)
[group("corpus")]
db corpus *flags:
    bash scripts/ingest.sh {{ corpus }} {{ flags }}

# import all five Bible translations (KJV first — canonical)
[group("corpus")]
db-bible:
    bash scripts/db_bible.sh

# wipe the local DB, then re-import every corpus the manifest knows + bible
[group("corpus")]
db-reload:
    bash scripts/db_reset.sh
    for c in $(bash scripts/ingest.sh --list); do just db "$c"; done
    just db-bible

# mirror ./assets/ to the scholia-assets bucket; flags reach rclone (e.g. --dry-run)
[group("assets")]
assets-sync *flags:
    bash scripts/assets_sync.sh {{ flags }}

# (re)apply bucket lifecycle rules (idempotent; currently scholia-assets-auto)
[group("assets")]
assets-lifecycle:
    bash scripts/assets_lifecycle.sh

# (re)apply the scholia-backups retention rule (daily/ dumps expire at 60 days)
[group("assets")]
backups-lifecycle:
    bash scripts/backups_lifecycle.sh

# kant1 OCR pre-curation stages (raw → lines → elements)
[group("assets")]
elem-kant1:
    cargo run -p kant1_ocr_to_lines
    cargo run -p kant1_lines_to_elements

# port-forward the dev cluster's Postgres to localhost:55432 (leave running)
[group("dev-cluster")]
dev-forward:
    KUBECONFIG=~/.kube/scholia-dev.yaml kubectl port-forward -n scholia svc/postgres 55432:5432

# run a command against the dev cluster DB (needs `just dev-forward` in another terminal)
[group("dev-cluster")]
dev-run *cmd:
    bash scripts/db_dev_run.sh {{ cmd }}

# confirmation-gated dev-cluster schema reset
[group("dev-cluster")]
dev-reset:
    bash scripts/db_dev_reset.sh

# dev-cluster reset, then re-import every corpus + bible through the forward
[group("dev-cluster")]
dev-reload:
    bash scripts/db_dev_reset.sh
    for c in $(bash scripts/ingest.sh --list); do bash scripts/db_dev_run.sh just db "$c"; done
    bash scripts/db_dev_run.sh just db-bible

# push a phone notification via ntfy (needs NTFY_URL in the env)
[group("dev-cluster")]
notify *msg:
    bash scripts/notify.sh {{ msg }}

# restore the DB from a daily backup — acts on your CURRENT kubectl context
# (set KUBECONFIG for dev/prod); flags: --dump <key|latest> --region --dry-run
[group("dev-cluster")]
db-restore *flags:
    bash scripts/db_restore.sh {{ flags }}
