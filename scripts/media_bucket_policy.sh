#!/usr/bin/env bash
#
# Apply the anonymous-read bucket policy on the user-media buckets
# (scholia-media, scholia-media-dev). The proxy's /media/ location
# fetches objects from these buckets without credentials, so GetObject
# must be public; everything else (list, write) stays credentialed.
#
# This lives in a script, not Terraform: the aws provider can't converge
# bucket sub-resources against Hetzner (Ceph) — same story as the
# lifecycle rules, see scripts/assets_lifecycle.sh. The buckets
# themselves ARE Terraform-managed — infra/terraform/shared/main.tf.
#
# Idempotent: each PUT replaces the bucket's whole policy.
#
# Requires AWS_ACCESS_KEY_ID + AWS_SECRET_ACCESS_KEY in env (Hetzner S3
# credentials): source ~/.config/scholia-infra.env
set -euo pipefail

: "${AWS_ACCESS_KEY_ID:?source ~/.config/scholia-infra.env first}"
: "${AWS_SECRET_ACCESS_KEY:?source ~/.config/scholia-infra.env first}"

endpoint="https://fsn1.your-objectstorage.com"

for bucket in scholia-media scholia-media-dev; do
    policy=$(cat << EOF
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "PublicReadGetObject",
            "Effect": "Allow",
            "Principal": "*",
            "Action": "s3:GetObject",
            "Resource": "arn:aws:s3:::${bucket}/*"
        }
    ]
}
EOF
    )

    echo "→ PUT policy on ${bucket} ..."
    curl -fsS -X PUT \
        --aws-sigv4 "aws:amz:fsn1:s3" \
        --user "${AWS_ACCESS_KEY_ID}:${AWS_SECRET_ACCESS_KEY}" \
        -H "Content-Type: application/json" \
        --data-binary "$policy" \
        "${endpoint}/${bucket}/?policy"

    echo "→ Verifying (GET policy on ${bucket}):"
    curl -fsS \
        --aws-sigv4 "aws:amz:fsn1:s3" \
        --user "${AWS_ACCESS_KEY_ID}:${AWS_SECRET_ACCESS_KEY}" \
        "${endpoint}/${bucket}/?policy"
    echo
done
