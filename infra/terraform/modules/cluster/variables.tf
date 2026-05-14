variable "hostname" {
  description = "Public FQDN this cluster serves (e.g. dev.scholia.study)."
  type        = string
}

variable "dns_zone" {
  description = "Porkbun-registered apex domain."
  type        = string
}

variable "vps_type" {
  description = "Hetzner server type slug."
  type        = string
}

variable "vps_image" {
  description = "Hetzner image slug."
  type        = string
}

variable "vps_location" {
  description = "Hetzner location slug (fsn1, nbg1, hel1)."
  type        = string
}

variable "ssh_public_keys" {
  description = "Public SSH keys placed in root@<host>'s authorized_keys."
  type        = list(string)
}

variable "tailscale_auth_key" {
  description = "Pre-auth key used by cloud-init to register the VPS with the tailnet on first boot."
  type        = string
  sensitive   = true
}

variable "environment" {
  description = "Workspace name (dev|prod) — used for tagging and the Hetzner server name."
  type        = string
}
