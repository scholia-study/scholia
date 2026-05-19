# Root module: wire providers, then call modules/cluster once per
# environment via workspaces (one workspace = one cluster).
#
# Use:
#   terraform workspace new dev    # first time only
#   terraform workspace select dev
#   terraform apply -var-file=dev.tfvars
#
# (And the same with `prod`.) Workspaces keep per-environment state
# isolated in the s3 backend under env:/<workspace>/…

provider "hcloud" {
  # Token from env: HCLOUD_TOKEN.
}

provider "porkbun" {
  api_key    = var.porkbun_api_key
  secret_key = var.porkbun_secret_key
}

module "cluster" {
  source = "./modules/cluster"

  hostname     = var.hostname
  dns_zone     = var.dns_zone
  vps_type     = var.vps_type
  vps_image    = var.vps_image
  vps_location = var.vps_location
  # file() and pathexpand() resolve at plan time. ~ doesn't expand on
  # its own in Terraform paths, so pathexpand() is doing real work
  # here — without it, file() looks for a literal "~" directory.
  ssh_public_keys    = [file(pathexpand(var.ssh_public_key_path))]
  tailscale_auth_key = var.tailscale_auth_key
  environment        = terraform.workspace
}

output "vps_public_ipv4" {
  description = "Public IPv4 of the cluster node. Use this for the A record sanity check (`dig <hostname>`)."
  value       = module.cluster.vps_public_ipv4
}

output "kubeconfig_fetch_command" {
  description = "One-liner to pull the kubeconfig over the tailnet once the node is up."
  value       = module.cluster.kubeconfig_fetch_command
}
