# Dev cluster inputs.
#
# Sensitive values (tailscale_auth_key) are NOT in this file — they
# come from TF_VAR_tailscale_auth_key in the shell env
#
# Apply with:
#   source ~/.config/scholia-infra.env
#   terraform workspace select dev   # or `new dev` first time
#   terraform apply -var-file=dev.tfvars

hostname = "dev.scholia.study"

# Path to the laptop SSH public key. main.tf reads the file contents
# at plan time via file(pathexpand(...)) and passes them through to
# cloud-init. Default in variables.tf is ~/.ssh/id_ed25519.pub —
# uncomment + edit only if yours lives elsewhere.
# ssh_public_key_path = "~/.ssh/id_rsa.pub"

# Defaults from variables.tf:
#   vps_type     = "cx22"
#   vps_image    = "ubuntu-24.04"
#   vps_location = "fsn1"
#   dns_zone     = "scholia.study"
# Override here if dev should diverge from those.
