import {
    Box,
    Button,
    CircularProgress,
    Stack,
    Typography,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { getMeQueryKey } from "#/api/auth/auth";
import { useAuth } from "#/hooks/useAuth";
import { CancellationNote } from "./CancellationNote";
import { getCurrentTier, MEMBERSHIP_PERKS } from "./tiers";

const POLL_INTERVAL_MS = 1500;
const POLL_TIMEOUT_MS = 25_000;

type Status = "pending" | "success" | "timeout";

/**
 * Post-checkout landing page. Stripe redirects here via `return_url`
 * after a successful subscription; we then poll /api/auth/me until the
 * webhook lands the new role (~few seconds typically). Three states:
 *   - pending: spinner while polling
 *   - success: thank-you + perks + onward links
 *   - timeout: payment succeeded but role hasn't propagated
 */
export function WelcomePage() {
    const { user } = useAuth();
    const queryClient = useQueryClient();
    const [status, setStatus] = useState<Status>("pending");

    const tier = getCurrentTier(user?.roles);

    // Promote pending -> success the moment the role arrives.
    useEffect(() => {
        if (status === "pending" && tier) {
            setStatus("success");
        }
    }, [status, tier]);

    // Poll while pending; stop on success or timeout.
    useEffect(() => {
        if (status !== "pending") return;
        const intervalId = window.setInterval(() => {
            queryClient.invalidateQueries({ queryKey: getMeQueryKey() });
        }, POLL_INTERVAL_MS);
        const timeoutId = window.setTimeout(() => {
            setStatus("timeout");
        }, POLL_TIMEOUT_MS);
        return () => {
            window.clearInterval(intervalId);
            window.clearTimeout(timeoutId);
        };
    }, [status, queryClient]);

    return (
        <div className="flex-1 bg-white">
            <div className="max-w-2xl mx-auto px-6 py-20 text-center">
                {status === "pending" ? (
                    <PendingState />
                ) : status === "success" && tier ? (
                    <SuccessState tierLabel={tier.label} />
                ) : (
                    <TimeoutState />
                )}
            </div>
        </div>
    );
}

function PendingState() {
    return (
        <Stack spacing={3} alignItems="center" className="py-10">
            <CircularProgress />
            <Typography variant="h5" component="p" className="!font-serif">
                Activating your membership…
            </Typography>
            <Typography variant="body2" className="!text-stone-500">
                This usually takes a few seconds.
            </Typography>
        </Stack>
    );
}

function SuccessState({ tierLabel }: { tierLabel: string }) {
    return (
        <>
            <Typography
                variant="h3"
                component="h1"
                className="!font-serif !mb-3"
            >
                Thank you.
            </Typography>
            <Typography variant="body1" className="!text-stone-600 !mb-2">
                You're now a {tierLabel}. Your support keeps Scholia open.
            </Typography>
            <Typography variant="body2" className="!text-stone-500 !mb-8">
                Membership unlocks:
            </Typography>

            <Box className="max-w-md mx-auto mb-10 text-left">
                <ul className="space-y-2">
                    {MEMBERSHIP_PERKS.map((perk) => (
                        <li
                            key={perk}
                            className="flex items-start gap-3 text-stone-700"
                        >
                            <span
                                aria-hidden="true"
                                className="text-stone-400 mt-0.5"
                            >
                                ✓
                            </span>
                            <span>{perk}</span>
                        </li>
                    ))}
                </ul>
            </Box>

            <Stack
                direction={{ xs: "column", sm: "row" }}
                spacing={2}
                justifyContent="center"
            >
                <Button component={Link} to="/" variant="contained">
                    Continue reading
                </Button>
                <Button component={Link} to="/membership" variant="outlined">
                    Manage membership
                </Button>
            </Stack>

            <CancellationNote className="mt-10 max-w-md mx-auto" />
        </>
    );
}

function TimeoutState() {
    return (
        <>
            <Typography
                variant="h4"
                component="h1"
                className="!font-serif !mb-3"
            >
                Payment received
            </Typography>
            <Typography variant="body1" className="!text-stone-600 !mb-8">
                Your payment went through, but the membership is still
                propagating. It should land within a minute — try refreshing the
                page, or come back shortly.
            </Typography>
            <Stack
                direction={{ xs: "column", sm: "row" }}
                spacing={2}
                justifyContent="center"
            >
                <Button
                    variant="contained"
                    onClick={() => window.location.reload()}
                >
                    Refresh
                </Button>
                <Button component={Link} to="/membership" variant="outlined">
                    Back to membership
                </Button>
            </Stack>
        </>
    );
}
