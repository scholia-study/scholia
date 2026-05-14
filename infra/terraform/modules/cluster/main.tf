# One Hetzner VPS running k3s + Tailscale + the cluster firewall, plus
# the Porkbun DNS A record pointing at it.

locals {
  # `dev.scholia.study` → "dev"; "scholia.study" → "" (apex).
  subdomain = trimsuffix(trimsuffix(var.hostname, var.dns_zone), ".")
}

# ---------------------------------------------------------------------
# Firewall
#
# Per PLAN_DEVOPS § 1.2 the public-facing ports are 80, 443, and ICMP.
# SSH (22) and the k3s API (6443) are intentionally NOT opened at the
# Hetzner firewall — they're reachable only over the tailnet, on the
# 100.x.x.x interface Tailscale adds inside the VPS. That side isn't
# touched by Hetzner Cloud Firewall (it sees the tailscale0 interface
# as already inside the host), so dropping 22/6443 here costs us
# nothing while shrinking the public attack surface to two ports.
# ---------------------------------------------------------------------
resource "hcloud_firewall" "this" {
  name = "scholia-${var.environment}"

  rule {
    description = "HTTP (cert-manager HTTP-01 + 301 to HTTPS)"
    direction   = "in"
    protocol    = "tcp"
    port        = "80"
    source_ips  = ["0.0.0.0/0", "::/0"]
  }

  rule {
    description = "HTTPS"
    direction   = "in"
    protocol    = "tcp"
    port        = "443"
    source_ips  = ["0.0.0.0/0", "::/0"]
  }

  rule {
    description = "ICMP (debugging)"
    direction   = "in"
    protocol    = "icmp"
    source_ips  = ["0.0.0.0/0", "::/0"]
  }
}

# ---------------------------------------------------------------------
# VPS
# ---------------------------------------------------------------------
resource "hcloud_server" "this" {
  name        = "scholia-${var.environment}"
  server_type = var.vps_type
  image       = var.vps_image
  location    = var.vps_location

  # cloud-init script: installs Tailscale + k3s on first boot. See
  # ../../cloud-init/k3s.yaml.tpl.
  user_data = templatefile("${path.module}/../../cloud-init/k3s.yaml.tpl", {
    hostname           = var.hostname
    environment        = var.environment
    tailscale_auth_key = var.tailscale_auth_key
    ssh_public_keys    = var.ssh_public_keys
  })

  firewall_ids = [hcloud_firewall.this.id]

  labels = {
    project     = "scholia"
    environment = var.environment
    managed_by  = "terraform"
  }

  # Hetzner serves user_data over a metadata endpoint that the kernel
  # reads at first boot; rebuilding the server (rather than just
  # replacing user_data in-place, which would be a no-op) is the only
  # way to re-run cloud-init.
  lifecycle {
    ignore_changes = [user_data]
  }
}

# ---------------------------------------------------------------------
# DNS — Porkbun A record
#
# `cullenmcdermott/porkbun` resource name is `porkbun_dns_record`.
# Subdomain "" maps the apex (scholia.study); a non-empty subdomain
# becomes "<sub>.scholia.study".
# ---------------------------------------------------------------------
resource "porkbun_dns_record" "a" {
  domain  = var.dns_zone
  name    = local.subdomain
  type    = "A"
  content = hcloud_server.this.ipv4_address
  ttl     = 600
}

# ---------------------------------------------------------------------
# www → apex CNAME, only when this workspace IS the apex (prod). For
# dev we don't want a www.dev.scholia.study record.
# ---------------------------------------------------------------------
resource "porkbun_dns_record" "www_cname" {
  count = local.subdomain == "" ? 1 : 0

  domain  = var.dns_zone
  name    = "www"
  type    = "CNAME"
  content = var.dns_zone
  ttl     = 600
}
