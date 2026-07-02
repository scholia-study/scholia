# Scholia operational tasks — `just --list` to browse.

set shell := ["bash", "-uc"]

# list available recipes
default:
    @just --list --unsorted

# --- corpus: curated MD → struct JSON → Postgres -----------------------------

# regenerate a corpus's struct JSON(s) from curated MD (kant1|kant3|shakespeare1|milton1|ibsen1)
struct corpus:
    bash scripts/struct.sh {{ corpus }}

# import a corpus's struct JSON(s) into Postgres; flags reach the importer (e.g. --dry-run, --database-url …)
db corpus *flags:
    bash scripts/ingest.sh {{ corpus }} {{ flags }}

# import all five Bible translations (KJV first — canonical)
db-bible:
    bash scripts/db_bible.sh

# wipe the local DB, then re-import every corpus the manifest knows + bible
db-reload:
    bash scripts/db_reset.sh
    for c in $(bash scripts/ingest.sh --list); do just db "$c"; done
    just db-bible

# --- assets ------------------------------------------------------------------

# mirror ./assets/ to the scholia-assets bucket; flags reach rclone (e.g. --dry-run)
assets-sync *flags:
    bash scripts/assets_sync.sh {{ flags }}

# kant1 OCR pre-curation stages (raw → lines → elements)
elem-kant1:
    cargo run -p kant1_ocr_to_lines
    cargo run -p kant1_lines_to_elements

# --- dev cluster (scholia-dev) -------------------------------------------------

# port-forward the dev cluster's Postgres to localhost:55432 (leave running)
dev-forward:
    KUBECONFIG=~/.kube/scholia-dev.yaml kubectl port-forward -n scholia svc/postgres 55432:5432

# run a command against the dev cluster DB (needs `just dev-forward` in another terminal)
dev-run *cmd:
    bash scripts/db_dev_run.sh {{ cmd }}

# confirmation-gated dev-cluster schema reset
dev-reset:
    bash scripts/db_dev_reset.sh

# dev-cluster reset, then re-import every corpus + bible through the forward
dev-reload:
    bash scripts/db_dev_reset.sh
    for c in $(bash scripts/ingest.sh --list); do bash scripts/db_dev_run.sh just db "$c"; done
    bash scripts/db_dev_run.sh just db-bible
