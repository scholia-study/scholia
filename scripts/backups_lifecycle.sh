#!/usr/bin/env bash
#
# Apply the retention lifecycle rule on every DB-backup bucket: objects
# under db/daily/ expire after 60 days. The daily pg_dump CronJob
# (infra/k8s/base/postgres/backup-cronjob.yaml) writes each dump to
# db/daily/<env>/<ts>.dump.gz and mirrors it to all three regional
# buckets, so this one prefix rule caps every environment at "last 60
# daily dumps" — no edit needed as environments are added.
#
# Same rationale as scripts/assets_lifecycle.sh for living in a script,
# not Terraform: aws provider >= 5.70 verifies lifecycle PUTs by polling
# for transition_default_minimum_object_size in the read-back, which
# Hetzner (Ceph) never echoes, so the resource times out on every apply
# (aws/aws-sdk-go-v2#3285). The buckets themselves are Terraform-managed —
# see infra/terraform/shared/main.tf.
#
# Idempotent: each PUT replaces its bucket's whole lifecycle config.
#
# Requires AWS_ACCESS_KEY_ID + AWS_SECRET_ACCESS_KEY in env (Hetzner S3
# credentials, project-scoped — one keypair works across all regions):
# source ~/.config/scholia-infra.env
set -euo pipefail

: "${AWS_ACCESS_KEY_ID:?source ~/.config/scholia-infra.env first}"
: "${AWS_SECRET_ACCESS_KEY:?source ~/.config/scholia-infra.env first}"

# region  bucket
targets=(
    "fsn1 scholia-backups"
    "hel1 scholia-backups-sigma"
    "nbg1 scholia-backups-tau"
)

config='<?xml version="1.0" encoding="UTF-8"?>
<LifecycleConfiguration>
    <Rule>
        <ID>expire-daily-dumps</ID>
        <Prefix>db/daily/</Prefix>
        <Status>Enabled</Status>
        <Expiration><Days>60</Days></Expiration>
    </Rule>
</LifecycleConfiguration>'

# PutBucketLifecycleConfiguration requires Content-MD5.
md5=$(printf '%s' "$config" | openssl dgst -md5 -binary | base64)

for entry in "${targets[@]}"; do
    read -r region bucket <<<"$entry"
    endpoint="https://${region}.your-objectstorage.com"

    echo "→ PUT lifecycle on ${bucket} (${region}) ..."
    curl -fsS -X PUT \
        --aws-sigv4 "aws:amz:${region}:s3" \
        --user "${AWS_ACCESS_KEY_ID}:${AWS_SECRET_ACCESS_KEY}" \
        -H "Content-MD5: ${md5}" \
        -H "Content-Type: application/xml" \
        --data-binary "$config" \
        "${endpoint}/${bucket}/?lifecycle"

    echo "→ Verifying (GET lifecycle):"
    curl -fsS \
        --aws-sigv4 "aws:amz:${region}:s3" \
        --user "${AWS_ACCESS_KEY_ID}:${AWS_SECRET_ACCESS_KEY}" \
        "${endpoint}/${bucket}/?lifecycle"
    echo
done
