# Inputs to the root module.
#
# Sensitive values (HCLOUD_TOKEN, Porkbun keys, Tailscale pre-auth key)
# come from the environment, not from tfvars files. Env-var contract:
#   HCLOUD_TOKEN               -> hcloud provider (auto-read)
#   TF_VAR_porkbun_api_key     -> this var
#   TF_VAR_porkbun_secret_key  -> this var
#   TF_VAR_tailscale_auth_key  -> this var
#   AWS_ACCESS_KEY_ID,         -> s3 backend (Hetzner Object Storage)
#     AWS_SECRET_ACCESS_KEY

variable "hostname" {
  description = "Public hostname for this environment (e.g. dev.scholia.study, scholia.study)."
  type        = string
}

variable "dns_zone" {
  description = "Apex domain registered at Porkbun. Used as Porkbun's `domain` argument; the hostname's leftmost label becomes the record subdomain."
  type        = string
  default     = "scholia.study"
}

variable "vps_type" {
  description = "Hetzner Cloud server type. cx23 = 2 vCPU / 4 GB / 40 GB SSD, x86, €4.99/mo in fsn1. (Renamed from the older cx22 slug.)"
  type        = string
  default     = "cx23"
}

variable "vps_image" {
  description = "Hetzner Cloud image slug."
  type        = string
  default     = "ubuntu-24.04"
}

variable "vps_location" {
  description = "Hetzner data centre location. fsn1 = Falkenstein; matches the Object Storage bucket region for same-region backups later."
  type        = string
  default     = "fsn1"
}

variable "ssh_public_key_path" {
  description = "Path to the laptop's public SSH key file. Cloud-init drops its contents into root@<host>'s authorized_keys. SSH is closed at the Hetzner firewall — only reachable over the tailnet."
  type        = string
  default     = "~/.ssh/id_ed25519.pub"
}

variable "tailscale_auth_key" {
  description = "Tailscale pre-auth key. Used by cloud-init to join the new VPS to the tailnet on first boot. Generate at https://login.tailscale.com/admin/settings/keys."
  type        = string
  sensitive   = true
}

variable "porkbun_api_key" {
  description = "Porkbun API key (public half of the pair). The cullenmcdermott/porkbun provider doesn't reliably pick up PORKBUN_API_KEY from the env, so we pass it explicitly via TF_VAR_porkbun_api_key."
  type        = string
  sensitive   = true
}

variable "porkbun_secret_key" {
  description = "Porkbun secret API key. Passed via TF_VAR_porkbun_secret_key in the shell env."
  type        = string
  sensitive   = true
}
