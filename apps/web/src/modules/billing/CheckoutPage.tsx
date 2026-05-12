import { Box, Button, CircularProgress, Typography } from "@mui/material";
import {
    EmbeddedCheckout,
    EmbeddedCheckoutProvider,
} from "@stripe/react-stripe-js";
import { Link, useNavigate } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import toast from "react-hot-toast";
import { useCreateCheckoutSession } from "#/api/billing/billing";
import { getStripe } from "./stripe";

export type CheckoutTier = "base" | "mid" | "high";

interface CheckoutPageProps {
    tier: CheckoutTier;
}

/**
 * Full-page wrapper around Stripe Embedded Checkout. Mounts the
 * provider exactly once with stable options so we never trigger
 * Stripe's "you cannot change the onComplete option after setting it"
 * warning. On completion, navigates back to /membership with an
 * `activating` flag so MembershipPage can poll /api/auth/me until the
 * webhook lands the new role.
 */
export function CheckoutPage({ tier }: CheckoutPageProps) {
    const navigate = useNavigate();

    const [clientSecret, setClientSecret] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);

    const createCheckout = useCreateCheckoutSession({
        mutation: {
            onSuccess: (res) => {
                if (res.status === 200) {
                    setClientSecret(res.data.client_secret);
                }
            },
            onError: () => {
                setError("Couldn't start checkout. Please try again.");
                toast.error("Couldn't start checkout.");
            },
        },
    });

    // Pull the stable `mutate` function (React Query memoizes it) so
    // the effect's dep array is stable across renders. Depending on
    // `createCheckout` itself would re-run the effect every render
    // because the result object is new each time.
    const { mutate } = createCheckout;

    // Fire the create-session call exactly once per tier. useRef guard
    // is for StrictMode dev double-invoke (and hot-reload remounts).
    const fired = useRef<string | null>(null);
    useEffect(() => {
        if (fired.current === tier) return;
        fired.current = tier;
        mutate({ data: { tier } });
    }, [tier, mutate]);

    // Stable onComplete — Stripe forbids changing it after the provider
    // mounts, so it must be a stable reference for the lifetime of the
    // page.
    const onComplete = useCallback(() => {
        navigate({ to: "/membership", search: { activating: true } });
    }, [navigate]);

    const options = useMemo(
        () => (clientSecret ? { clientSecret, onComplete } : null),
        [clientSecret, onComplete],
    );

    if (error) {
        return (
            <div className="flex-1 bg-white">
                <div className="max-w-2xl mx-auto px-6 py-16 text-center">
                    <Typography variant="h5" className="!mb-3">
                        Checkout couldn't start
                    </Typography>
                    <Typography
                        variant="body1"
                        className="!text-stone-600 !mb-6"
                    >
                        {error}
                    </Typography>
                    <Button
                        component={Link}
                        to="/membership"
                        variant="outlined"
                    >
                        Back to membership
                    </Button>
                </div>
            </div>
        );
    }

    return (
        <div className="flex-1 bg-white">
            {/* Full available width: Stripe Embedded Checkout renders a
             * responsive two-column layout (summary + form) at larger
             * widths, which feels much better than a narrow column. */}
            <div className="px-6 md:px-8 py-12">
                <Box className="mb-6 text-center">
                    <Link
                        to="/membership"
                        className="text-sm text-stone-500 hover:text-stone-900"
                    >
                        ← Back to membership
                    </Link>
                </Box>
                {options ? (
                    <EmbeddedCheckoutProvider
                        stripe={getStripe()}
                        options={options}
                    >
                        <EmbeddedCheckout />
                    </EmbeddedCheckoutProvider>
                ) : (
                    <Box className="flex items-center justify-center min-h-[40vh]">
                        <CircularProgress />
                    </Box>
                )}
            </div>
        </div>
    );
}
