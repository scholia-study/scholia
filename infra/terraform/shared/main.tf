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

output "assets_bucket_name" {
  description = "Asset bucket name. Wire into rclone configs and Job manifests."
  value       = aws_s3_bucket.assets.id
}

output "assets_endpoint" {
  description = "S3 endpoint for the assets bucket."
  value       = "https://fsn1.your-objectstorage.com"
}
