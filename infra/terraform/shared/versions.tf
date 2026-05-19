# Terraform + provider pins for shared, non-cluster-specific resources.
# Currently houses the assets Object Storage bucket; future additions
# might be a DB-backup bucket, DNS apex records that aren't per-cluster,
# etc.
#
# Applied once (no workspaces), independently of infra/terraform/clusters/.
# State lives in the same Hetzner Object Storage backend bucket as the
# cluster state, but under a different key — they're two separate
# Terraform configurations that just happen to share storage.

terraform {
  required_version = ">= 1.6"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }

  backend "s3" {
    bucket = "scholia-tf-state"
    key    = "scholia/shared.tfstate"
    region = "fsn1" # Hetzner ignores it; the s3 backend insists on something

    endpoints = {
      s3 = "https://fsn1.your-objectstorage.com"
    }

    # All the "this isn't actually AWS" knobs. Same set as
    # infra/terraform/clusters/versions.tf — without them the backend
    # tries to talk to AWS STS / IAM and fails.
    skip_credentials_validation = true
    skip_metadata_api_check     = true
    skip_region_validation      = true
    skip_requesting_account_id  = true
    use_path_style              = true
  }
}
