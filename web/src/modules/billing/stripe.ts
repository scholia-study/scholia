import { loadStripe, type Stripe } from "@stripe/stripe-js";

/**
 * Cache the loadStripe() promise so it runs once per page load.
 * loadStripe handles its own deduplication internally too, but
 * caching the promise keeps our usage explicit.
 */
let stripePromise: Promise<Stripe | null> | null = null;

export function getStripe(): Promise<Stripe | null> {
    if (!stripePromise) {
        const key = import.meta.env.VITE_STRIPE_PUBLISHABLE_KEY as
            | string
            | undefined;
        if (!key) {
            throw new Error(
                "VITE_STRIPE_PUBLISHABLE_KEY is missing — set it in web/.env",
            );
        }
        stripePromise = loadStripe(key);
    }
    return stripePromise;
}
