import {
    Box,
    Button,
    Card,
    CardActions,
    CardContent,
    CircularProgress,
    Stack,
    Typography,
} from "@mui/material";
import { Link, useNavigate } from "@tanstack/react-router";
import toast from "react-hot-toast";
import { useCreatePortalSession } from "#/api/billing/billing";
import { useAuth } from "#/hooks/useAuth";
import { CANCELLATION_COPY } from "./CancellationNote";
import { getCurrentTier, MEMBERSHIP_PERKS, TIERS, type Tier } from "./tiers";

export function MembershipPage() {
    const { user, isAuthenticated, isLoading } = useAuth();

    const createPortal = useCreatePortalSession({
        mutation: {
            onSuccess: (res) => {
                if (res.status === 200) {
                    window.location.href = res.data.url;
                }
            },
            onError: () => {
                toast.error("Couldn't open the billing portal.");
            },
        },
    });

    const currentTier = getCurrentTier(user?.roles);
    const isPaid = !!currentTier;

    if (isLoading) {
        return (
            <Box className="flex items-center justify-center min-h-[60vh]">
                <CircularProgress />
            </Box>
        );
    }

    return (
        // flex-1 (not min-h-full) so the white background fills the
        // remaining vertical space in the root flex-column layout —
        // otherwise short content lets the page bg show through to
        // the footer.
        <div className="flex-1 bg-white">
            <div className="max-w-5xl mx-auto px-6 md:px-8 py-12 md:py-20">
                <header className="mb-16 text-center">
                    <Typography
                        variant="h3"
                        component="h1"
                        className="!font-serif !mb-3"
                    >
                        Membership
                    </Typography>
                    <div className="max-w-2xl mx-auto">
                        <Typography variant="body1" className="!text-stone-600">
                            Scholia is a free, open reader for the classical
                            canon; philosophical, literary, and historical works
                            structured down to the sentence and aligned across
                            translations. Membership funds the platform
                            development and editorial work behind it.
                        </Typography>
                        <Typography
                            variant="body1"
                            className="!text-stone-600 !mt-3"
                        >
                            All tiers grant the same access. Choose the level of
                            support you can sustain.
                        </Typography>
                        <Typography
                            variant="body1"
                            className="!text-stone-600 !mt-3"
                        >
                            Whatever you choose, thank you. Your support enables
                            us to develop and maintain this work.
                        </Typography>
                    </div>
                </header>

                {isPaid && currentTier ? (
                    <CurrentSubscriptionCard
                        tier={currentTier}
                        onManage={() => createPortal.mutate(undefined as never)}
                        managePending={createPortal.isPending}
                    />
                ) : null}

                <Stack
                    direction={{ xs: "column", md: "row" }}
                    spacing={3}
                    className="mt-16"
                >
                    {TIERS.map((tier) => (
                        <TierCard
                            key={tier.slug}
                            tier={tier}
                            isCurrent={currentTier?.slug === tier.slug}
                            isAuthenticated={isAuthenticated}
                            isPaid={isPaid}
                        />
                    ))}
                </Stack>

                {!isAuthenticated ? (
                    <p className="mt-10 text-center text-stone-600">
                        <Link
                            to="/register"
                            className="text-stone-900 underline"
                        >
                            Create an account
                        </Link>{" "}
                        or{" "}
                        <Link to="/login" className="text-stone-900 underline">
                            sign in
                        </Link>{" "}
                        to become a member.
                    </p>
                ) : null}
            </div>
        </div>
    );
}

interface TierCardProps {
    tier: Tier;
    isCurrent: boolean;
    isAuthenticated: boolean;
    isPaid: boolean;
}

function TierCard({ tier, isCurrent, isAuthenticated, isPaid }: TierCardProps) {
    const navigate = useNavigate();
    return (
        <Card
            variant="outlined"
            className="flex-1"
            sx={
                isCurrent
                    ? { borderColor: "primary.main", borderWidth: 2 }
                    : undefined
            }
        >
            <CardContent>
                <Typography variant="overline" className="!text-stone-500">
                    {tier.label}
                </Typography>
                <Typography variant="h4" className="!font-serif !mt-1">
                    {tier.price}
                    <Typography
                        component="span"
                        variant="body2"
                        className="!text-stone-500 !ml-1"
                    >
                        / month
                    </Typography>
                </Typography>
                <Typography variant="body2" className="!text-stone-600 !mt-3">
                    {tier.blurb}
                </Typography>
                <ul className="mt-4 space-y-1.5">
                    {MEMBERSHIP_PERKS.map((perk) => (
                        <li
                            key={perk}
                            className="flex items-start gap-2 text-sm text-stone-600"
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
            </CardContent>
            <CardActions sx={{ px: 2, pb: 2 }}>
                {isCurrent ? (
                    <Button fullWidth disabled variant="outlined">
                        Current tier
                    </Button>
                ) : isPaid ? (
                    // Paid user on a different tier: tier switching
                    // routes through the Stripe Customer Portal (the
                    // "Manage subscription" button on the active card
                    // above), so no per-card action is needed here.
                    // Empty placeholder keeps card heights aligned.
                    <Box className="h-9 w-full" aria-hidden="true" />
                ) : isAuthenticated ? (
                    // useNavigate instead of <Link> here: MUI's
                    // component={Link} overload erases the route literal,
                    // so the typed `search` prop fails to validate.
                    // Button onClick stays fully typed via navigate().
                    <Button
                        fullWidth
                        variant="contained"
                        onClick={() =>
                            navigate({
                                to: "/membership/checkout",
                                search: { tier: tier.slug },
                            })
                        }
                    >
                        Become a {tier.shortLabel}
                    </Button>
                ) : (
                    <Button
                        fullWidth
                        variant="outlined"
                        component={Link}
                        to="/register"
                    >
                        Sign up to support
                    </Button>
                )}
            </CardActions>
        </Card>
    );
}

interface CurrentSubscriptionCardProps {
    tier: Tier;
    onManage: () => void;
    managePending: boolean;
}

function CurrentSubscriptionCard({
    tier,
    onManage,
    managePending,
}: CurrentSubscriptionCardProps) {
    return (
        <Card
            variant="outlined"
            sx={{ borderColor: "primary.main", borderWidth: 2 }}
        >
            <CardContent>
                <Stack
                    direction={{ xs: "column", sm: "row" }}
                    spacing={2}
                    alignItems={{ xs: "stretch", sm: "center" }}
                    justifyContent="space-between"
                >
                    <Box>
                        <Typography
                            variant="overline"
                            className="!text-stone-500"
                        >
                            Active membership
                        </Typography>
                        <Typography
                            variant="h5"
                            className="!font-serif"
                            component="div"
                        >
                            {tier.label} · {tier.price} / month
                        </Typography>
                    </Box>
                    <Button
                        variant="outlined"
                        onClick={onManage}
                        disabled={managePending}
                    >
                        Manage subscription
                    </Button>
                </Stack>
                <Typography
                    variant="body2"
                    className="!text-stone-500 !mt-4 !pt-4 !border-t !border-stone-200"
                >
                    {CANCELLATION_COPY}
                </Typography>
            </CardContent>
        </Card>
    );
}
