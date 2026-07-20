# AWS provider pointed at Hetzner Object Storage. AWS_ACCESS_KEY_ID
# and AWS_SECRET_ACCESS_KEY in the env are the Hetzner-issued S3
# credentials (NOT the Hetzner Cloud API token).
provider "aws" {
  region = "fsn1"

  endpoints {
    s3 = "https://fsn1.your-objectstorage.com"
  }

  # Hetzner Object Storage speaks S3 but doesn't expose IAM / STS /
  # metadata APIs, so all the AWS-flavored validation must be off.
  skip_credentials_validation = true
  skip_metadata_api_check     = true
  skip_region_validation      = true
  skip_requesting_account_id  = true
  s3_use_path_style           = true
}

# Asset store for content ingested by the in-cluster Jobs (Bible
# translations, Kant struct JSONs, etc.). Local `assets/` is the
# canonical source; `just assets-sync` mirrors it here. See
# PLAN_DEVOPS.md § Ingest-as-Jobs.
resource "aws_s3_bucket" "assets" {
  bucket = "scholia-assets"
}

# CI-owned counterpart for auto-ingest. Kept separate
# from scholia-assets because `just assets-sync` mirrors with delete
# semantics and would wipe CI-only prefixes.
#
# Its 30-day expiry lifecycle rule is NOT managed here: the provider's
# post-PUT read-back poll expects fields Hetzner never echoes, so the
# resource times out on every apply — with filter {}, with legacy
# prefix, and with neither (tested on v5.100.0; see
# aws/aws-sdk-go-v2#3285). The rule is applied by
# `just assets-lifecycle` instead.
resource "aws_s3_bucket" "assets_auto" {
  bucket = "scholia-assets-auto"
}

# Daily pg_dump target (see PLAN_DEVOPS.md § 3 + backup-cronjob.yaml).
# Created out of band before this resource existed, so it must be
# imported before the next apply, or the create call collides:
#   terraform import aws_s3_bucket.backups scholia-backups
# The 60-day retention rule is NOT here (aws provider can't converge
# Hetzner lifecycle PUTs — see assets_auto's note); it lives in
# scripts/backups_lifecycle.sh.
resource "aws_s3_bucket" "backups" {
  bucket = "scholia-backups"
}

output "assets_bucket_name" {
  description = "Asset bucket name. Wire into rclone configs and Job manifests."
  value       = aws_s3_bucket.assets.id
}

output "assets_auto_bucket_name" {
  description = "Auto-ingest bucket name. Wire into build.yml and the ingest Job manifests."
  value       = aws_s3_bucket.assets_auto.id
}

output "backups_bucket_name" {
  description = "DB-backup bucket name. Wire into the backup CronJob + scripts/backups_lifecycle.sh."
  value       = aws_s3_bucket.backups.id
}

output "assets_endpoint" {
  description = "S3 endpoint for the assets bucket."
  value       = "https://fsn1.your-objectstorage.com"
}
