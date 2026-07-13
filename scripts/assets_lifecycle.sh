#!/usr/bin/env bash
#
# Apply lifecycle rules on the Scholia buckets. Currently one rule:
# scholia-assets-auto expires every object after 30 days — it holds
# only CI-built derived structs under hash-keyed prefixes
# (<corpus>/derived@<treehash>/), all re-buildable from git, so aging
# them out is safe.
#
# This lives in a script, not Terraform: aws provider ≥ 5.70 verifies
# lifecycle PUTs by polling for transition_default_minimum_object_size
# in the read-back, which Hetzner (Ceph) never echoes, so the resource
# times out on every apply (aws/aws-sdk-go-v2#3285). The buckets
# themselves ARE Terraform-managed — see infra/terraform/shared/main.tf.
#
# Idempotent: each PUT replaces its bucket's whole lifecycle config.
#
# Requires AWS_ACCESS_KEY_ID + AWS_SECRET_ACCESS_KEY in env (Hetzner S3
# credentials): source ~/.config/scholia-infra.env
set -euo pipefail

: "${AWS_ACCESS_KEY_ID:?source ~/.config/scholia-infra.env first}"
: "${AWS_SECRET_ACCESS_KEY:?source ~/.config/scholia-infra.env first}"

endpoint="https://fsn1.your-objectstorage.com"
bucket="scholia-assets-auto"

config='<?xml version="1.0" encoding="UTF-8"?>
<LifecycleConfiguration>
    <Rule>
        <ID>expire-stale-derived-structs</ID>
        <Prefix></Prefix>
        <Status>Enabled</Status>
        <Expiration><Days>30</Days></Expiration>
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
