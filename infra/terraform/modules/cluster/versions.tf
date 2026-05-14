# Each module needs its own required_providers block — without one,
# Terraform assumes hashicorp/<name> for every resource's provider
# source, which doesn't resolve for community providers. Sources here
# MUST match the root module's required_providers in
# ../../versions.tf; version constraints can be looser (the root's
# narrower constraint wins).

terraform {
  required_providers {
    hcloud = {
      source = "hetznercloud/hcloud"
    }
    porkbun = {
      source = "cullenmcdermott/porkbun"
    }
  }
}
