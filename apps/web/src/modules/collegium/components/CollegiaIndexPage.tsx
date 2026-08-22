import AddOutlined from "@mui/icons-material/AddOutlined";
import LockOutlined from "@mui/icons-material/LockOutlined";
import {
    Button,
    Checkbox,
    Chip,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    FormControlLabel,
    Paper,
    TextField,
    Tooltip,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { Link, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import toast from "react-hot-toast";
import {
    getDiscoverCollegiaQueryKey,
    getListMyCollegiaQueryKey,
    useCreateCollegium,
    useCreateJoinRequest,
    useDiscoverCollegia,
    useListMyCollegia,
} from "#/api/collegia/collegia";
import { FetchError } from "#/api/fetcher";
import type { CollegiumResponse } from "#/api/model";
import { useDebouncedValue } from "#/hooks/useDebouncedValue";
import { useFeedback } from "#/modules/feedback";
import { ReviewVisibilityField } from "./ReviewVisibilityField";

const DISCOVER_PER_PAGE = 20;

/** The `/user/collegia` page: the user's own collegia plus public-collegium discovery. */
export function CollegiaIndexPage() {
    const queryClient = useQueryClient();

    const { data: myCollegiaData, isLoading: myCollegiaLoading } =
        useListMyCollegia();
    const myCollegia = myCollegiaData?.data?.collegia ?? [];
    const createdCount = myCollegiaData?.data?.created_count ?? 0;
    const maxCreated = myCollegiaData?.data?.max_created ?? 1;
    const canCreate = createdCount < maxCreated;

    const [search, setSearch] = useState("");
    const [page, setPage] = useState(1);
    const debouncedSearch = useDebouncedValue(search.trim(), 300);
    const { data: discoverData, isLoading: discoverLoading } =
        useDiscoverCollegia({
            q: debouncedSearch || undefined,
            page,
            per_page: DISCOVER_PER_PAGE,
        });
    const discovered = discoverData?.data?.collegia ?? [];
    const discoverTotal = discoverData?.data?.total ?? 0;
    const pageCount = Math.max(1, Math.ceil(discoverTotal / DISCOVER_PER_PAGE));

    const invalidate = () => {
        queryClient.invalidateQueries({
            queryKey: getListMyCollegiaQueryKey(),
        });
        queryClient.invalidateQueries({
            queryKey: getDiscoverCollegiaQueryKey(),
        });
    };

    return (
        <div className="w-full max-w-3xl mx-auto px-8 py-16">
            <div className="flex items-center justify-between mb-2">
                <h1 className="text-2xl font-bold text-stone-900">Collegia</h1>
                <CreateCollegiumButton
                    canCreate={canCreate}
                    maxCreated={maxCreated}
                    onCreated={invalidate}
                />
            </div>
            <p className="text-sm text-stone-500 mb-8">
                Collegia are groups where you can workshop your ideas and
                articles with friends and fellow scholars: submit a draft to a
                group for feedback instead of to the Scholia editors.
            </p>

            <h2 className="text-lg font-semibold text-stone-900 mb-3">
                My collegia
            </h2>
            {myCollegiaLoading && (
                <p className="text-sm text-stone-400">Loading...</p>
            )}
            {!myCollegiaLoading && myCollegia.length === 0 && (
                <p className="text-sm text-stone-400 mb-4">
                    You aren't a member of any collegium yet. Create one, or ask
                    to join one below.
                </p>
            )}
            <div className="space-y-2 mb-10">
                {myCollegia.map((collegium) => (
                    <CollegiumRow key={collegium.id} collegium={collegium} />
                ))}
            </div>

            <h2 className="text-lg font-semibold text-stone-900 mb-3">
                Discover
            </h2>
            <TextField
                placeholder="Search public collegia"
                value={search}
                onChange={(e) => {
                    setSearch(e.target.value);
                    setPage(1);
                }}
                size="small"
                fullWidth
                sx={{ mb: 2 }}
            />
            {discoverLoading && (
                <p className="text-sm text-stone-400">Loading...</p>
            )}
            {!discoverLoading && discovered.length === 0 && (
                <p className="text-sm text-stone-400">
                    {debouncedSearch
                        ? "No public collegia match your search."
                        : "No public collegia yet."}
                </p>
            )}
            <div className="space-y-2">
                {discovered.map((collegium) => (
                    <CollegiumRow
                        key={collegium.id}
                        collegium={collegium}
                        onJoinRequested={invalidate}
                    />
                ))}
            </div>
            {pageCount > 1 && (
                <div className="flex items-center justify-center gap-3 mt-4">
                    <Button
                        size="small"
                        disabled={page <= 1}
                        onClick={() => setPage((p) => p - 1)}
                    >
                        Previous
                    </Button>
                    <span className="text-xs text-stone-400">
                        {page} / {pageCount}
                    </span>
                    <Button
                        size="small"
                        disabled={page >= pageCount}
                        onClick={() => setPage((p) => p + 1)}
                    >
                        Next
                    </Button>
                </div>
            )}
        </div>
    );
}

function CollegiumRow({
    collegium,
    onJoinRequested,
}: {
    collegium: CollegiumResponse;
    /** Present on Discover rows — enables the ask-to-join action. */
    onJoinRequested?: () => void;
}) {
    const joinMutation = useCreateJoinRequest();

    const askToJoin = async (e: React.MouseEvent) => {
        // The whole card is a link — asking to join must not navigate.
        e.preventDefault();
        e.stopPropagation();
        try {
            await joinMutation.mutateAsync({ slug: collegium.slug });
            toast.success("Request sent — a collegium steward will review it.");
            onJoinRequested?.();
        } catch (err) {
            toast.error(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to send join request",
            );
        }
    };

    return (
        <Link
            to="/user/collegia/$slug"
            params={{ slug: collegium.slug }}
            className="block no-underline"
        >
            <Paper
                elevation={0}
                sx={{
                    border: "1px solid rgb(214 211 209)",
                    p: 1.5,
                    display: "flex",
                    alignItems: "center",
                    gap: 1.5,
                    cursor: "pointer",
                    position: "relative",
                    overflow: "hidden",
                    transition: "box-shadow 0.15s",
                    "&:hover": { boxShadow: 3 },
                }}
            >
                {collegium.my_role === "steward" && (
                    <Tooltip title="You are a steward of this collegium">
                        <span className="absolute top-0 right-0 w-0 h-0 border-t-[26px] border-l-[26px] border-t-sky-600 border-l-transparent" />
                    </Tooltip>
                )}
                <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-0.5">
                        <span className="text-sm font-medium text-stone-900 truncate">
                            {collegium.name}
                        </span>
                        {collegium.is_private && (
                            <Tooltip title="Private collegium">
                                <LockOutlined
                                    sx={{ fontSize: 14, color: "#a8a29e" }}
                                />
                            </Tooltip>
                        )}
                    </div>
                    {collegium.description && (
                        <p className="text-xs text-stone-500 truncate">
                            {collegium.description}
                        </p>
                    )}
                    <span className="text-[10px] text-stone-400">
                        {collegium.member_count}{" "}
                        {collegium.member_count === 1 ? "member" : "members"}
                    </span>
                </div>
                {onJoinRequested && !collegium.my_role && (
                    <div className="shrink-0">
                        {collegium.my_pending_request ? (
                            <Chip
                                label="Requested"
                                size="small"
                                sx={{ fontSize: "0.65rem", height: 22 }}
                            />
                        ) : (
                            <Button
                                size="small"
                                variant="outlined"
                                disabled={joinMutation.isPending}
                                onClick={askToJoin}
                                sx={{ textTransform: "none" }}
                            >
                                Ask to join
                            </Button>
                        )}
                    </div>
                )}
            </Paper>
        </Link>
    );
}

function CreateCollegiumButton({
    canCreate,
    maxCreated,
    onCreated,
}: {
    canCreate: boolean;
    maxCreated: number;
    onCreated: () => void;
}) {
    const navigate = useNavigate();
    const { openModal: openFeedbackModal } = useFeedback();
    const [open, setOpen] = useState(false);
    const [name, setName] = useState("");
    const [description, setDescription] = useState("");
    const [isPrivate, setIsPrivate] = useState(false);
    const [reviewVisibility, setReviewVisibility] = useState<
        "members" | "stewards"
    >("members");
    const createMutation = useCreateCollegium();

    const create = async () => {
        if (!name.trim()) return;
        try {
            const result = await createMutation.mutateAsync({
                data: {
                    name: name.trim(),
                    description: description.trim() || undefined,
                    is_private: isPrivate,
                    review_visibility: reviewVisibility,
                },
            });
            setOpen(false);
            onCreated();
            navigate({
                to: "/user/collegia/$slug",
                params: { slug: result.data.slug },
            });
        } catch (err) {
            toast.error(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to create collegium",
            );
        }
    };

    return (
        <>
            <Tooltip
                title={
                    canCreate ? (
                        ""
                    ) : maxCreated === 1 ? (
                        <span>
                            You have used your one collegium creation. Become a{" "}
                            <Link
                                to="/membership"
                                className="text-inherit underline"
                            >
                                Scholiast member
                            </Link>{" "}
                            to unlock more or join existing collegia instead.
                        </span>
                    ) : (
                        <span>
                            You have used your {maxCreated} collegium creations.{" "}
                            <button
                                type="button"
                                className="underline cursor-pointer bg-transparent border-0 p-0 text-inherit font-inherit"
                                onClick={openFeedbackModal}
                            >
                                Contact an admin
                            </button>{" "}
                            if you need more, or join existing collegia instead.
                        </span>
                    )
                }
            >
                <span>
                    <Button
                        variant="contained"
                        size="small"
                        startIcon={<AddOutlined />}
                        disabled={!canCreate}
                        onClick={() => setOpen(true)}
                        sx={{ textTransform: "none" }}
                    >
                        New collegium
                    </Button>
                </span>
            </Tooltip>
            <Dialog
                open={open}
                onClose={() => setOpen(false)}
                maxWidth="sm"
                fullWidth
            >
                <DialogTitle>New collegium</DialogTitle>
                <DialogContent
                    sx={{ display: "flex", flexDirection: "column", gap: 2 }}
                >
                    <p className="text-sm text-stone-600">
                        You'll be the collegium's first steward. Collegium
                        creation is limited per account, and deleting a
                        collegium does not give the slot back — name it with
                        care.
                    </p>
                    <TextField
                        autoFocus
                        fullWidth
                        label="Name"
                        value={name}
                        onChange={(e) => setName(e.target.value)}
                        onKeyDown={(e) => {
                            if (e.key === "Enter") create();
                        }}
                    />
                    <TextField
                        fullWidth
                        label="Description (optional)"
                        value={description}
                        onChange={(e) => setDescription(e.target.value)}
                        multiline
                        rows={2}
                    />
                    <ReviewVisibilityField
                        value={reviewVisibility}
                        onChange={setReviewVisibility}
                    />
                    <FormControlLabel
                        control={
                            <Checkbox
                                checked={isPrivate}
                                onChange={(e) => setIsPrivate(e.target.checked)}
                                size="small"
                            />
                        }
                        label={
                            <span className="text-sm">
                                Private. Hidden from Discover. People join via
                                invite link only.
                            </span>
                        }
                    />
                </DialogContent>
                <DialogActions>
                    <Button onClick={() => setOpen(false)}>Cancel</Button>
                    <Button
                        onClick={create}
                        variant="contained"
                        disabled={!name.trim() || createMutation.isPending}
                    >
                        Create
                    </Button>
                </DialogActions>
            </Dialog>
        </>
    );
}
