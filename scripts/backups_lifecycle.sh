#!/usr/bin/env bash
#
# Apply the retention lifecycle rule on scholia-backups: everything under
# the daily/ prefix expires after 60 days. The daily pg_dump CronJob
# (infra/k8s/base/postgres/backup-cronjob.yaml) writes there, so this
# rule is what caps storage at "last 60 daily dumps".
#
# Same rationale as scripts/assets_lifecycle.sh for living in a script,
# not Terraform: aws provider >= 5.70 verifies lifecycle PUTs by polling
# for transition_default_minimum_object_size in the read-back, which
# Hetzner (Ceph) never echoes, so the resource times out on every apply
# (aws/aws-sdk-go-v2#3285). The bucket itself is created out of band /
# in infra/terraform/shared/main.tf; only this rule lives here.
#
# Idempotent: the PUT replaces the bucket's whole lifecycle config.
#
# Requires AWS_ACCESS_KEY_ID + AWS_SECRET_ACCESS_KEY in env (Hetzner S3
# credentials): source ~/.config/scholia-infra.env
set -euo pipefail

: "${AWS_ACCESS_KEY_ID:?source ~/.config/scholia-infra.env first}"
: "${AWS_SECRET_ACCESS_KEY:?source ~/.config/scholia-infra.env first}"

endpoint="https://fsn1.your-objectstorage.com"
bucket="scholia-backups"

config='<?xml version="1.0" encoding="UTF-8"?>
<LifecycleConfiguration>
    <Rule>
        <ID>expire-daily-dumps</ID>
        <Prefix>daily/</Prefix>
        <Status>Enabled</Status>
        <Expiration><Days>60</Days></Expiration>
    </Rule>
</LifecycleConfiguration>'

# PutBucketLifecycleConfiguration requires Content-MD5.
md5=$(printf '%s' "$config" | openssl dgst -md5 -binary | base64)

echo "→ PUT lifecycle on ${bucket} ..."
curl -fsS -X PUT \
    --aws-sigv4 "aws:amz:fsn1:s3" \
    --user "${AWS_ACCESS_KEY_ID}:${AWS_SECRET_ACCESS_KEY}" \
    -H "Content-MD5: ${md5}" \
    -H "Content-Type: application/xml" \
    --data-binary "$config" \
    "${endpoint}/${bucket}/?lifecycle"

echo "→ Verifying (GET lifecycle):"
curl -fsS \
    --aws-sigv4 "aws:amz:fsn1:s3" \
    --user "${AWS_ACCESS_KEY_ID}:${AWS_SECRET_ACCESS_KEY}" \
    "${endpoint}/${bucket}/?lifecycle"
echo
