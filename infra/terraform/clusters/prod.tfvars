# Prod cluster inputs — STUB ONLY, not for application yet.
#
# Bring up dev first, validate end-to-end, then come back and complete
# this file. Same shape as dev.tfvars.
#
# When you're ready:
#   1. Fill in `ssh_public_keys` (typically the same key as dev).
#   2. Generate a separate Tailscale pre-auth key tagged for prod;
#      export it as TF_VAR_tailscale_auth_key before applying.
#   3. terraform workspace new prod
#   4. terraform apply -var-file=prod.tfvars

hostname = "scholia.study"

ssh_public_keys = [
  # TODO: fill in before applying prod.
  # file("~/.ssh/id_ed25519.pub")
]
