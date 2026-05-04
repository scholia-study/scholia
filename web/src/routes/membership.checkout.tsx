import { createFileRoute } from "@tanstack/react-router";
import { CheckoutPage, type CheckoutTier } from "../modules/billing";

interface CheckoutSearch {
    tier: CheckoutTier;
}

const VALID_TIERS = new Set<CheckoutTier>(["base", "mid", "high"]);

export const Route = createFileRoute("/membership/checkout")({
    validateSearch: (search: Record<string, unknown>): CheckoutSearch => {
        const t = search.tier;
        if (typeof t === "string" && VALID_TIERS.has(t as CheckoutTier)) {
            return { tier: t as CheckoutTier };
        }
        // Default to entry tier if the URL is missing/invalid; the page
        // will still render and the user can navigate back.
        return { tier: "base" };
    },
    component: CheckoutRoute,
});

function CheckoutRoute() {
    const { tier } = Route.useSearch();
    return <CheckoutPage tier={tier} />;
}
