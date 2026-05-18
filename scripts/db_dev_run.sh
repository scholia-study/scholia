#!/usr/bin/env bash

# Requires, in another terminal:
#   pnpm db:dev:forward
# This shell:
#   source ~/.config/scholia-infra.env

set -euo pipefail

if [[ $# -eq 0 ]]; then
    echo "usage: $0 <command> [args...]" >&2
    exit 64
fi

cd "$(dirname "$0")/.."

PASS=$(sops -d infra/k8s/overlays/dev/secrets/postgres.yaml | sed -n 's/^[[:space:]]*password:[[:space:]]*//p')
ENC=$(python3 -c 'import urllib.parse,sys;print(urllib.parse.quote(sys.argv[1],safe=""))' "$PASS")
export DATABASE_URL="postgres://prospero:$ENC@localhost:55432/prospero"

exec "$@"
