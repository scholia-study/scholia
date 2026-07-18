#!/usr/bin/env bash
#
# Publish a push notification to the scholia ntfy topic — the shell-side
# sibling of Argo's notification hook (infra/argo/values.yaml). For
# scripts, CronJobs, and ad-hoc use; the topic URL is a credential and
# comes from the environment, never from git.
#
#   NTFY_URL=https://ntfy.sh/<secret-topic>   (source ~/.config/scholia-infra.env)
#
# Usage: notify.sh [-t title] [-p default|high|urgent] <message...>
#   e.g. notify.sh -t backup -p urgent "pg_dump failed"
set -euo pipefail

title="scholia"
priority="default"
while getopts "t:p:" opt; do
    case "$opt" in
        t) title="$OPTARG" ;;
        p) priority="$OPTARG" ;;
        *) exit 2 ;;
    esac
done
shift $((OPTIND - 1))

[ $# -ge 1 ] || { echo "usage: notify.sh [-t title] [-p priority] <message...>" >&2; exit 2; }
: "${NTFY_URL:?Set NTFY_URL (source ~/.config/scholia-infra.env)}"

curl -fsS -o /dev/null \
    -H "Title: $title" \
    -H "Priority: $priority" \
    -d "$*" \
    "$NTFY_URL"
