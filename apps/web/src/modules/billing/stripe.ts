import { loadStripe, type Stripe } from "@stripe/stripe-js";
import config from "../../config";

/**
 * Cache the loadStripe() promise so it runs once per page load.
 * loadStripe handles its own deduplication internally too, but
 * caching the promise keeps our usage explicit.
 */
let stripePromise: Promise<Stripe | null> | null = null;

export function getStripe(): Promise<Stripe | null> {
    if (!stripePromise) {
        stripePromise = loadStripe(config.STRIPE_PUBLISHABLE_KEY);
    }
    return stripePromise;
}
