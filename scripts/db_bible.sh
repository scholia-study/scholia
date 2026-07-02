#!/usr/bin/env bash
# Import all five public-domain English Bibles from a local laptop
# checkout. The run logic (KJV-first canonical, four parallel) lives in
# scripts/bible_import.sh, shared with the ingest-bible job image.
#
# Local only
set -euo pipefail

exec bash "$(dirname "$0")/bible_import.sh"
