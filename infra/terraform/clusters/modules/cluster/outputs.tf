output "vps_public_ipv4" {
  description = "The cluster node's public IPv4. The Porkbun A record points at this."
  value       = hcloud_server.this.ipv4_address
}

output "vps_name" {
  description = "Hetzner server name (also doubles as the Tailscale machine name once registered)."
  value       = hcloud_server.this.name
}

output "kubeconfig_fetch_command" {
  description = <<-EOT
    One-liner to pull k3s's kubeconfig from the node over the tailnet
    once Tailscale has registered the host. The k3s default kubeconfig
    has server=127.0.0.1, which is rewritten to the tailnet hostname so
    your laptop can reach the API server.

    Note the SSH/MagicDNS hostname is `<env>-scholia` (set by
    cloud-init's `tailscale up --hostname=...`), distinct from the
    Hetzner server name `scholia-<env>` which only labels the box in
    the Hetzner console.

    Run after `terraform apply` finishes and the node has had ~60s to
    boot, install Tailscale, and install k3s.
  EOT
  value = "ssh root@${var.environment}-scholia 'sed s,127.0.0.1,${var.environment}-scholia,g /etc/rancher/k3s/k3s.yaml' > ~/.kube/scholia-${var.environment}.yaml"
}
