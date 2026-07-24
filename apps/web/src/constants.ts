// Site-wide constants. Import from here rather than hardcoding values.

export const CONTACT_EMAIL = "contact@example.com";
export const SITE_NAME = "Scholia";

// Roles an admin may grant or revoke in the user dashboard. Mirrors the
// backend `MANAGEABLE_ROLES` in `identity::admin_users`.
export const MANAGEABLE_ROLES = [
    "admin",
    "editor",
    "scholiast",
    "honorary",
] as const;

// Paid tiers, owned by the Stripe webhook role-sync. Shown read-only in the
// admin UI — an admin cannot grant or revoke them directly.
export const PAID_ROLES = ["scholiast_benefactor", "scholiast_patron"] as const;
