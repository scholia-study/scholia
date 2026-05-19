# Terraform + provider version pins.
#
# We require >= 1.6 for the s3 backend's `endpoints = {…}` block, which
# is how we point at Hetzner Object Storage instead of real AWS.
#
# The Porkbun provider is community-maintained. There are several on
# the Terraform Registry — verify the chosen one is still active at
# https://registry.terraform.io/search/providers?q=porkbun and bump
# the version if a more current fork has emerged. `terraform init`
# will fail loudly if the source string is wrong.

terraform {
  required_version = ">= 1.6"

  required_providers {
    hcloud = {
      source  = "hetznercloud/hcloud"
      version = "~> 1.48"
    }
    porkbun = {
      source  = "cullenmcdermott/porkbun"
      version = "~> 0.1"
    }
  }

  # State lives in the Hetzner Object Storage bucket created out-of-band
  # (web console). One bucket, two workspaces (dev, prod) — the s3
  # backend places per-workspace state at
  #   env:/<workspace>/<key>
  # so we end up with:
  #   scholia-tf-state/env:/dev/scholia/terraform.tfstate
  #   scholia-tf-state/env:/prod/scholia/terraform.tfstate
  #
  # AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY in the env supply the
  # bucket credentials (Hetzner-issued S3 credential pair — not the
  # Hetzner Cloud API token).
  backend "s3" {
    bucket = "scholia-tf-state"
    key    = "scholia/terraform.tfstate"
    region = "fsn1" # arbitrary string; Hetzner ignores it but the s3 backend insists on something

    endpoints = {
      s3 = "https://fsn1.your-objectstorage.com"
    }

    # All the "this isn't actually AWS" knobs. Without these the
    # backend tries to talk to AWS STS / IAM and fails.
    skip_credentials_validation = true
    skip_metadata_api_check     = true
    skip_region_validation      = true
    skip_requesting_account_id  = true
    use_path_style              = true
  }
}
