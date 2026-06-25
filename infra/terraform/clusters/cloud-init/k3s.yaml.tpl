#cloud-config
# First-boot setup for a Scholia cluster node.
#
# Ordering matters:
#   1. SSH keys go in early so we have a foothold even if later steps fail.
#   2. Tailscale comes up next — this is the durable management plane;
#      once it's registered, the box is reachable from your laptop over
#      the tailnet even without public SSH.
#   3. k3s installs last; its `INSTALL_K3S_EXEC` flags lock down a few
#      things for the single-node single-tenant case.
#
# Interpolated values are supplied by Terraform's templatefile() call
# in modules/cluster/main.tf — they're not shell substitutions.

hostname: ${environment}-scholia
manage_etc_hosts: true

users:
  - name: root
    ssh_authorized_keys:
%{ for k in ssh_public_keys ~}
      - ${k}
%{ endfor ~}

package_update: true
package_upgrade: false # security updates only — full upgrades on first boot are slow and rarely worth it
packages:
  - curl
  - ca-certificates
  - gnupg
  - unattended-upgrades

write_files:
  - path: /etc/sysctl.d/99-scholia.conf
    content: |
      # k3s + small node tunings.
      net.ipv4.ip_forward = 1
      net.bridge.bridge-nf-call-iptables = 1
    permissions: "0644"

  # unattended-upgrades: apply Ubuntu security updates daily without
  # human intervention.
  - path: /etc/apt/apt.conf.d/20auto-upgrades
    content: |
      APT::Periodic::Update-Package-Lists "1";
      APT::Periodic::Unattended-Upgrade "1";
      APT::Periodic::AutocleanInterval "7";
    permissions: "0644"

  # Scheduled reboot.
  - path: /usr/local/sbin/reboot-if-required.sh
    permissions: "0755"
    content: |
      #!/bin/sh
      set -eu
      MIN_UPTIME_DAYS=%{ if environment == "prod" }20%{ else }6%{ endif }
      [ -f /var/run/reboot-required ] || exit 0
      up_days=$(awk '{print int($1/86400)}' /proc/uptime)
      [ "$up_days" -ge "$MIN_UPTIME_DAYS" ] || {
        logger -t scheduled-reboot "reboot pending but uptime $${up_days}d < $${MIN_UPTIME_DAYS}d; deferring"
        exit 0
      }
      logger -t scheduled-reboot "reboot pending, uptime $${up_days}d; rebooting now"
      systemctl reboot

  - path: /etc/systemd/system/scheduled-reboot.service
    permissions: "0644"
    content: |
      [Unit]
      Description=Apply a pending OS reboot if one is required
      ConditionPathExists=/var/run/reboot-required
      [Service]
      Type=oneshot
      ExecStart=/usr/local/sbin/reboot-if-required.sh

  - path: /etc/systemd/system/scheduled-reboot.timer
    permissions: "0644"
    content: |
      [Unit]
      Description=Weekly maintenance-window check for a pending reboot
      [Timer]
      OnCalendar=Sun *-*-* 04:00:00
      RandomizedDelaySec=600
      Persistent=true
      [Install]
      WantedBy=timers.target

runcmd:
  # --- Tailscale ----------------------------------------------------
  # Install via the official one-liner. The auth key joins the node to
  # the tailnet non-interactively. `--ssh` flips on Tailscale SSH so
  # we can reach the box without Hetzner-firewall port 22 being open.
  # `--accept-routes=false` keeps the node from advertising other
  # tailnet subnets out of caution.
  - |
    n=0
    until [ "$n" -ge 5 ]; do
      curl -fsSL https://tailscale.com/install.sh | sh && break
      n=$((n+1)); echo "tailscale install attempt $n/5 failed; retrying in 10s"; sleep 10
    done
  - tailscale up
      --auth-key=${tailscale_auth_key}
      --hostname=${environment}-scholia
      --ssh
      --accept-routes=false

  # --- k3s ----------------------------------------------------------
  # Stock k3s with Traefik (default) and local-path storage (default).
  # We pin the channel to `stable` so dev and prod track the same
  # train without surprise jumps. `--write-kubeconfig-mode=644` makes
  # the kubeconfig readable to a normal user once we add one; for now
  # everything runs as root so it doesn't matter much.
  - |
    n=0
    until [ "$n" -ge 5 ]; do
      curl -sfL https://get.k3s.io | INSTALL_K3S_CHANNEL=stable sh -s - server --write-kubeconfig-mode=644 --disable-cloud-controller --node-name=${environment}-scholia && break
      n=$((n+1)); echo "k3s install attempt $n/5 failed; retrying in 10s"; sleep 10
    done

  # Apply sysctl tweaks now (they're persisted via the file above for
  # future boots).
  - sysctl --system

  # Enable the maintenance-window reboot timer (units written above).
  - systemctl daemon-reload
  - systemctl enable --now scheduled-reboot.timer

# Once cloud-init has run, a marker file exists at
# /var/lib/cloud/instance/boot-finished. To check the node from your
# laptop after `terraform apply`:
#
#   ssh root@${environment}-scholia ls -la /var/lib/cloud/instance/boot-finished
#
# If that's there and `kubectl get nodes` (via the kubeconfig you'll
# scp over) shows Ready, you're done.
