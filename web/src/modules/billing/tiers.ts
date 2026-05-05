/**
 * Patronage tiers — single source of truth for the frontend. The
 * `slug` is the API contract (sent to POST /api/billing/checkout, mapped
 * server-side to a Stripe Price ID via STRIPE_PRICE_BASE/_MID/_HIGH).
 * The `role` matches what `permissions.rs::Role::as_str()` returns
 * and what the webhook inserts into `user_roles`.
 */
export const TIERS = [
    {
        slug: "base",
        label: "Scholiast",
        shortLabel: "Scholiast",
        role: "scholiast",
        price: "€5",
        blurb: "Support the project and unlock the elevated note and quotation limits.",
    },
    {
        slug: "mid",
        label: "Scholiast Benefactor",
        shortLabel: "Benefactor",
        role: "scholiast_benefactor",
        price: "€15",
        blurb: "Sustain the project's growth and signal your support.",
    },
    {
        slug: "high",
        label: "Scholiast Patron",
        shortLabel: "Patron",
        role: "scholiast_patron",
        price: "€50",
        blurb: "Champion the project and help fund new editions.",
    },
] as const;

export type Tier = (typeof TIERS)[number];
export type TierSlug = Tier["slug"];

/** Find the tier metadata for a given role name, if any. */
export function getTierByRole(role: string): Tier | null {
    return TIERS.find((t) => t.role === role) ?? null;
}

/** Find the user's current paid tier from their role list, if any. */
export function getCurrentTier(
    roles: readonly string[] | undefined,
): Tier | null {
    if (!roles) return null;
    return TIERS.find((t) => roles.includes(t.role)) ?? null;
}

/**
 * What every paid tier grants. Identical across all three tiers (the
 * patronage model: tier == support level, not access level).
 */
export const MEMBERSHIP_PERKS = [
    "Unlimited notes and saved quotations",
    "Unlimited articles to draft and publish",
    "Advanced search (coming soon)",
];
