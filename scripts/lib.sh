# Shared helpers for the scripts/ shell family. Source this; don't execute it.
#
#   source "$(dirname "$0")/lib.sh"

# Default local Postgres (dev cluster overrides via DATABASE_URL).
SCHOLIA_DB_URL_DEFAULT="postgres://prospero:prospero@localhost:5433/prospero"

# THE corpus list — every struct-importer corpus, in import order. Single
# source of truth for `ingest.sh`/`struct.sh` validation + `--list` (which
# `just db-reload` iterates), so a new corpus can never fall out of the reload
# path. Bible is NOT here: it has its own importer, image, and fetch flow.
# Adding a corpus: add it here + a case arm in ingest.sh and struct.sh.
SCHOLIA_CORPORA=(kant1 kant3 shakespeare1 milton1 ibsen1)

scholia_corpus_known() {
    local c
    for c in "${SCHOLIA_CORPORA[@]}"; do
        [ "$c" = "$1" ] && return 0
    done
    return 1
}

# Guard: sqlx-cli present (migrations).
scholia_require_sqlx() {
    if ! command -v sqlx >/dev/null 2>&1; then
        echo "error: sqlx-cli not on PATH." >&2
        echo "install with: cargo install sqlx-cli --no-default-features --features postgres,rustls" >&2
        exit 1
    fi
}

# Configure the `scholia` rclone remote inline via env vars — no rclone.conf
# file involved. Lets any machine with rclone + the Hetzner S3 creds talk to
# the scholia-assets bucket, no first-time `rclone config` ritual.
scholia_rclone_config() {
    : "${AWS_ACCESS_KEY_ID:?Set AWS_ACCESS_KEY_ID (source ~/.config/scholia-infra.env)}"
    : "${AWS_SECRET_ACCESS_KEY:?Set AWS_SECRET_ACCESS_KEY (source ~/.config/scholia-infra.env)}"
    export RCLONE_CONFIG_SCHOLIA_TYPE=s3
    export RCLONE_CONFIG_SCHOLIA_PROVIDER=Other
    export RCLONE_CONFIG_SCHOLIA_ENDPOINT=https://fsn1.your-objectstorage.com
    export RCLONE_CONFIG_SCHOLIA_REGION=fsn1
    export RCLONE_CONFIG_SCHOLIA_ACCESS_KEY_ID="$AWS_ACCESS_KEY_ID"
    export RCLONE_CONFIG_SCHOLIA_SECRET_ACCESS_KEY="$AWS_SECRET_ACCESS_KEY"
    export RCLONE_CONFIG_SCHOLIA_FORCE_PATH_STYLE=true
}
