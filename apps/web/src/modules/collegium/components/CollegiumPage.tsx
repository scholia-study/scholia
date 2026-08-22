import ContentCopyOutlined from "@mui/icons-material/ContentCopyOutlined";
import LockOutlined from "@mui/icons-material/LockOutlined";
import SettingsOutlined from "@mui/icons-material/SettingsOutlined";
import {
    Button,
    Checkbox,
    Chip,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    FormControlLabel,
    IconButton,
    Paper,
    TextField,
    Tooltip,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { Link, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import toast from "react-hot-toast";
import { useListCollegiumReviewQueue } from "#/api/article-reviews/article-reviews";
import {
    getGetCollegiumQueryKey,
    getListJoinRequestsQueryKey,
    getListMyCollegiaQueryKey,
    useCreateJoinRequest,
    useDecideJoinRequest,
    useDisableInviteToken,
    useGetCollegium,
    useListJoinRequests,
    useRemoveMember,
    useRotateInviteToken,
    useUpdateCollegium,
    useUpdateMemberRole,
} from "#/api/collegia/collegia";
import { FetchError } from "#/api/fetcher";
import type {
    CollegiumDetailResponse,
    CollegiumMemberResponse,
} from "#/api/model";
import { useAuth } from "#/hooks/useAuth";
import {
    type ReviewVisibility,
    ReviewVisibilityField,
} from "./ReviewVisibilityField";

/** The `/user/collegia/$slug` page: workshop queue, members, and admin tools. */
export function CollegiumPage({ slug }: { slug: string }) {
    const { data, isLoading, error } = useGetCollegium(slug, {
        query: { retry: false },
    });
    const collegium = data?.data;

    if (isLoading) {
        return (
            <div className="w-full max-w-3xl mx-auto px-8 py-16">
                <p className="text-sm text-stone-400">Loading...</p>
            </div>
        );
    }
    if (error || !collegium) {
        return (
            <div className="w-full max-w-3xl mx-auto px-8 py-16">
                <h1 className="text-xl font-bold text-stone-900 mb-2">
                    Collegium not found
                </h1>
                <p className="text-sm text-stone-500">
                    This collegium doesn't exist, was deleted, or you don't have
                    access to it.{" "}
                    <Link to="/user/collegia" className="underline">
                        Back to Collegia
                    </Link>
                </p>
            </div>
        );
    }
    return <CollegiumDetail collegium={collegium} slug={slug} />;
}

function CollegiumDetail({
    collegium,
    slug,
}: {
    collegium: CollegiumDetailResponse;
    slug: string;
}) {
    const { user } = useAuth();
    const queryClient = useQueryClient();
    const navigate = useNavigate();

    const isMember = !!collegium.my_role;
    const isAdmin = collegium.my_role === "steward";

    const invalidateGroup = () => {
        queryClient.invalidateQueries({
            queryKey: getGetCollegiumQueryKey(slug),
        });
        queryClient.invalidateQueries({
            queryKey: getListMyCollegiaQueryKey(),
        });
    };

    const joinMutation = useCreateJoinRequest();
    const removeMutation = useRemoveMember();
    const [leaveOpen, setLeaveOpen] = useState(false);

    const askToJoin = async () => {
        try {
            await joinMutation.mutateAsync({ slug });
            toast.success("Request sent — a collegium steward will review it.");
            invalidateGroup();
        } catch (err) {
            toast.error(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to send join request",
            );
        }
    };

    const leave = async () => {
        if (!user) return;
        try {
            const result = await removeMutation.mutateAsync({
                slug,
                userId: user.id,
            });
            setLeaveOpen(false);
            if (result.data.collegium_deleted) {
                toast.success(
                    "You left as the last member; the collegium was deleted.",
                );
            } else {
                toast.success("You left the collegium.");
            }
            queryClient.invalidateQueries({
                queryKey: getListMyCollegiaQueryKey(),
            });
            navigate({ to: "/user/collegia" });
        } catch (err) {
            setLeaveOpen(false);
            toast.error(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to leave the collegium",
            );
        }
    };

    return (
        <div className="w-full max-w-3xl mx-auto px-8 py-16">
            <div className="flex items-start justify-between mb-1">
                <div className="flex items-center gap-2 min-w-0">
                    <h1 className="text-2xl font-bold text-stone-900 truncate">
                        {collegium.name}
                    </h1>
                    {collegium.is_private && (
                        <Tooltip title="Private collegium — joinable via invite link only">
                            <LockOutlined
                                sx={{ fontSize: 18, color: "#a8a29e" }}
                            />
                        </Tooltip>
                    )}
                    {collegium.my_role && (
                        <Chip
                            label={collegium.my_role}
                            size="small"
                            color={isAdmin ? "info" : "default"}
                            sx={{ fontSize: "0.65rem", height: 20 }}
                        />
                    )}
                </div>
                <div className="flex items-center gap-1 shrink-0">
                    {isAdmin && (
                        <CollegiumSettings
                            collegium={collegium}
                            slug={slug}
                            onChanged={invalidateGroup}
                        />
                    )}
                    {isMember && (
                        <Button
                            size="small"
                            color="inherit"
                            onClick={() => setLeaveOpen(true)}
                            sx={{ textTransform: "none", color: "#78716c" }}
                        >
                            Leave
                        </Button>
                    )}
                    {!isMember &&
                        !collegium.is_private &&
                        (collegium.my_pending_request ? (
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
                        ))}
                </div>
            </div>
            {collegium.description && (
                <p className="text-sm text-stone-600 mb-1">
                    {collegium.description}
                </p>
            )}
            <p className="text-xs text-stone-400 mb-8">
                {collegium.member_count}{" "}
                {collegium.member_count === 1 ? "member" : "members"}
                {" · "}
                {collegium.review_visibility === "stewards"
                    ? "feedback by stewards only"
                    : "feedback by all members"}
            </p>

            {isAdmin && <InviteLinkPanel collegium={collegium} slug={slug} />}
            {isAdmin && (collegium.pending_join_request_count ?? 0) > 0 && (
                <JoinRequestsPanel slug={slug} onDecided={invalidateGroup} />
            )}
            {isMember && (
                <WorkshopQueue
                    slug={slug}
                    adminsOnlyMode={
                        collegium.review_visibility === "stewards" && !isAdmin
                    }
                />
            )}
            {collegium.members && (
                <MembersPanel
                    slug={slug}
                    members={collegium.members}
                    isAdmin={isAdmin}
                    selfId={user?.id}
                    onChanged={invalidateGroup}
                />
            )}

            <Dialog open={leaveOpen} onClose={() => setLeaveOpen(false)}>
                <DialogTitle sx={{ fontSize: 16 }}>
                    Leave {collegium.name}?
                </DialogTitle>
                <DialogContent>
                    <p className="text-sm text-stone-600">
                        You'll lose access to the collegium's reviews, and any
                        of your pending review requests to this collegium will
                        be withdrawn.
                        {collegium.member_count === 1 &&
                            " You are the last member — leaves the collegium deleted permanently."}
                    </p>
                </DialogContent>
                <DialogActions>
                    <Button onClick={() => setLeaveOpen(false)}>Cancel</Button>
                    <Button
                        color="error"
                        variant="contained"
                        onClick={leave}
                        disabled={removeMutation.isPending}
                    >
                        Leave collegium
                    </Button>
                </DialogActions>
            </Dialog>
        </div>
    );
}

function WorkshopQueue({
    slug,
    adminsOnlyMode,
}: {
    slug: string;
    /** Classroom mode, non-admin viewer: the queue is their own work. */
    adminsOnlyMode: boolean;
}) {
    const { data, isLoading } = useListCollegiumReviewQueue(slug);
    const items = data?.data?.items ?? [];

    return (
        <section className="mb-10">
            <h2 className="text-lg font-semibold text-stone-900 mb-3">
                {adminsOnlyMode ? "Your submissions" : "Awaiting feedback"}
            </h2>
            {isLoading && <p className="text-sm text-stone-400">Loading...</p>}
            {!isLoading && items.length === 0 && (
                <p className="text-sm text-stone-400">
                    {adminsOnlyMode
                        ? 'No pending submissions. Submit an article from its editor page via "Request review". Only the group\'s admins will see it.'
                        : 'Nothing in the queue. Members can submit an article here from its editor page via "Request review".'}
                </p>
            )}
            <div className="space-y-2">
                {items.map((item) => (
                    <Paper
                        key={item.id}
                        elevation={0}
                        sx={{
                            border: "1px solid rgb(214 211 209)",
                            p: 1.5,
                            transition: "box-shadow 0.15s",
                            "&:hover": { boxShadow: 3 },
                        }}
                    >
                        <div className="flex items-center gap-2">
                            <Link
                                to="/articles/review/$requestId"
                                params={{ requestId: item.id }}
                                className="text-sm font-medium text-stone-900 hover:underline truncate"
                            >
                                {item.article_title}
                            </Link>
                            {item.open_comment_count > 0 && (
                                <Chip
                                    label={`${item.open_comment_count} open`}
                                    size="small"
                                    sx={{ fontSize: "0.65rem", height: 20 }}
                                />
                            )}
                        </div>
                        <div className="text-[11px] text-stone-400 mt-0.5">
                            by {item.author_display_name}
                            {" · "}
                            {new Date(item.submitted_at).toLocaleDateString(
                                undefined,
                                {
                                    month: "short",
                                    day: "numeric",
                                    year: "numeric",
                                },
                            )}
                        </div>
                    </Paper>
                ))}
            </div>
        </section>
    );
}

function MembersPanel({
    slug,
    members,
    isAdmin,
    selfId,
    onChanged,
}: {
    slug: string;
    members: CollegiumMemberResponse[];
    isAdmin: boolean;
    selfId?: string;
    onChanged: () => void;
}) {
    const roleMutation = useUpdateMemberRole();
    const removeMutation = useRemoveMember();

    const changeRole = async (userId: string, role: "steward" | "member") => {
        try {
            await roleMutation.mutateAsync({ slug, userId, data: { role } });
            onChanged();
        } catch (err) {
            toast.error(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to change role",
            );
        }
    };

    const remove = async (userId: string) => {
        try {
            await removeMutation.mutateAsync({ slug, userId });
            toast.success("Member removed.");
            onChanged();
        } catch (err) {
            toast.error(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to remove member",
            );
        }
    };

    return (
        <section className="mb-10">
            <h2 className="text-lg font-semibold text-stone-900 mb-3">
                Members
            </h2>
            <div className="space-y-1">
                {members.map((member) => {
                    const isSelf = member.user_id === selfId;
                    return (
                        <div
                            key={member.user_id}
                            className="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-stone-50"
                        >
                            <div className="flex-1 min-w-0 flex items-center gap-2">
                                {member.handle ? (
                                    <Link
                                        to="/users/$handle"
                                        params={{ handle: member.handle }}
                                        className="text-sm text-stone-900 hover:underline truncate"
                                    >
                                        {member.display_name}
                                    </Link>
                                ) : (
                                    <span className="text-sm text-stone-900 truncate">
                                        {member.display_name}
                                    </span>
                                )}
                                {member.role === "steward" && (
                                    <Chip
                                        label="steward"
                                        size="small"
                                        color="info"
                                        sx={{ fontSize: "0.65rem", height: 20 }}
                                    />
                                )}
                                {isSelf && (
                                    <span className="text-[10px] text-stone-400">
                                        you
                                    </span>
                                )}
                            </div>
                            <div className="shrink-0 flex items-center gap-1">
                                {isAdmin &&
                                    !isSelf &&
                                    member.role === "member" && (
                                        <>
                                            <Button
                                                size="small"
                                                sx={{
                                                    textTransform: "none",
                                                    fontSize: "0.7rem",
                                                }}
                                                disabled={
                                                    roleMutation.isPending
                                                }
                                                onClick={() =>
                                                    changeRole(
                                                        member.user_id,
                                                        "steward",
                                                    )
                                                }
                                            >
                                                Make steward
                                            </Button>
                                            <Button
                                                size="small"
                                                color="error"
                                                sx={{
                                                    textTransform: "none",
                                                    fontSize: "0.7rem",
                                                }}
                                                disabled={
                                                    removeMutation.isPending
                                                }
                                                onClick={() =>
                                                    remove(member.user_id)
                                                }
                                            >
                                                Remove
                                            </Button>
                                        </>
                                    )}
                                {isSelf && member.role === "steward" && (
                                    <Tooltip title="Become a regular member. Another steward must remain.">
                                        <span>
                                            <Button
                                                size="small"
                                                color="inherit"
                                                sx={{
                                                    textTransform: "none",
                                                    fontSize: "0.7rem",
                                                    color: "#78716c",
                                                }}
                                                disabled={
                                                    roleMutation.isPending
                                                }
                                                onClick={() =>
                                                    changeRole(
                                                        member.user_id,
                                                        "member",
                                                    )
                                                }
                                            >
                                                Step down
                                            </Button>
                                        </span>
                                    </Tooltip>
                                )}
                            </div>
                        </div>
                    );
                })}
            </div>
        </section>
    );
}

function JoinRequestsPanel({
    slug,
    onDecided,
}: {
    slug: string;
    onDecided: () => void;
}) {
    const queryClient = useQueryClient();
    const { data, isLoading } = useListJoinRequests(slug);
    const requests = data?.data?.requests ?? [];
    const decideMutation = useDecideJoinRequest();

    const decide = async (id: string, status: "approved" | "rejected") => {
        try {
            await decideMutation.mutateAsync({ slug, id, data: { status } });
            queryClient.invalidateQueries({
                queryKey: getListJoinRequestsQueryKey(slug),
            });
            onDecided();
        } catch (err) {
            toast.error(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to decide request",
            );
        }
    };

    if (isLoading || requests.length === 0) return null;

    return (
        <section className="mb-10">
            <h2 className="text-lg font-semibold text-stone-900 mb-3">
                Join requests
            </h2>
            <div className="space-y-1">
                {requests.map((request) => (
                    <div
                        key={request.id}
                        className="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-stone-50"
                    >
                        <div className="flex-1 min-w-0">
                            {request.handle ? (
                                <Link
                                    to="/users/$handle"
                                    params={{ handle: request.handle }}
                                    className="text-sm text-stone-900 hover:underline"
                                >
                                    {request.display_name}
                                </Link>
                            ) : (
                                <span className="text-sm text-stone-900">
                                    {request.display_name}
                                </span>
                            )}
                        </div>
                        <Button
                            size="small"
                            variant="outlined"
                            sx={{ textTransform: "none", fontSize: "0.7rem" }}
                            disabled={decideMutation.isPending}
                            onClick={() => decide(request.id, "approved")}
                        >
                            Approve
                        </Button>
                        <Button
                            size="small"
                            color="inherit"
                            sx={{
                                textTransform: "none",
                                fontSize: "0.7rem",
                                color: "#78716c",
                            }}
                            disabled={decideMutation.isPending}
                            onClick={() => decide(request.id, "rejected")}
                        >
                            Reject
                        </Button>
                    </div>
                ))}
            </div>
        </section>
    );
}

function InviteLinkPanel({
    collegium,
    slug,
}: {
    collegium: CollegiumDetailResponse;
    slug: string;
}) {
    const queryClient = useQueryClient();
    const rotateMutation = useRotateInviteToken();
    const disableMutation = useDisableInviteToken();

    const inviteUrl = collegium.invite_token
        ? `${window.location.origin}/user/collegia/join/${collegium.invite_token}`
        : null;

    const refresh = () =>
        queryClient.invalidateQueries({
            queryKey: getGetCollegiumQueryKey(slug),
        });

    const rotate = async () => {
        try {
            await rotateMutation.mutateAsync({ slug });
            refresh();
            toast.success(
                collegium.invite_token
                    ? "Invite link replaced — the old link no longer works."
                    : "Invite link created.",
            );
        } catch (err) {
            toast.error(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to update invite link",
            );
        }
    };

    const disable = async () => {
        try {
            await disableMutation.mutateAsync({ slug });
            refresh();
            toast.success("Invite link disabled.");
        } catch (err) {
            toast.error(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to disable invite link",
            );
        }
    };

    return (
        <section className="mb-10">
            <h2 className="text-lg font-semibold text-stone-900 mb-3">
                Invite link
            </h2>
            {inviteUrl ? (
                <div className="flex items-center gap-2">
                    <code className="text-xs bg-stone-100 border border-stone-200 rounded px-2 py-1.5 truncate flex-1">
                        {inviteUrl}
                    </code>
                    <Tooltip title="Copy link">
                        <IconButton
                            size="small"
                            onClick={() => {
                                navigator.clipboard.writeText(inviteUrl);
                                toast.success("Copied.");
                            }}
                        >
                            <ContentCopyOutlined fontSize="small" />
                        </IconButton>
                    </Tooltip>
                    <Button
                        size="small"
                        sx={{ textTransform: "none", fontSize: "0.7rem" }}
                        disabled={rotateMutation.isPending}
                        onClick={rotate}
                    >
                        Replace
                    </Button>
                    <Button
                        size="small"
                        color="inherit"
                        sx={{
                            textTransform: "none",
                            fontSize: "0.7rem",
                            color: "#78716c",
                        }}
                        disabled={disableMutation.isPending}
                        onClick={disable}
                    >
                        Disable
                    </Button>
                </div>
            ) : (
                <div className="flex items-center gap-3">
                    <p className="text-sm text-stone-500">
                        Anyone with the link joins instantly — no approval step.
                    </p>
                    <Button
                        size="small"
                        variant="outlined"
                        sx={{ textTransform: "none" }}
                        disabled={rotateMutation.isPending}
                        onClick={rotate}
                    >
                        Create invite link
                    </Button>
                </div>
            )}
        </section>
    );
}

function CollegiumSettings({
    collegium,
    slug,
    onChanged,
}: {
    collegium: CollegiumDetailResponse;
    slug: string;
    onChanged: () => void;
}) {
    const [open, setOpen] = useState(false);
    const [name, setName] = useState(collegium.name);
    const [description, setDescription] = useState(collegium.description ?? "");
    const [isPrivate, setIsPrivate] = useState(collegium.is_private);
    const [reviewVisibility, setReviewVisibility] = useState<ReviewVisibility>(
        collegium.review_visibility === "stewards" ? "stewards" : "members",
    );
    const updateMutation = useUpdateCollegium();

    const openDialog = () => {
        setName(collegium.name);
        setDescription(collegium.description ?? "");
        setIsPrivate(collegium.is_private);
        setReviewVisibility(
            collegium.review_visibility === "stewards" ? "stewards" : "members",
        );
        setOpen(true);
    };

    const save = async () => {
        try {
            await updateMutation.mutateAsync({
                slug,
                data: {
                    name: name.trim(),
                    // Empty string clears the description server-side.
                    description: description.trim(),
                    is_private: isPrivate,
                    review_visibility: reviewVisibility,
                },
            });
            setOpen(false);
            onChanged();
            toast.success("Collegium updated.");
        } catch (err) {
            toast.error(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to update collegium",
            );
        }
    };

    return (
        <>
            <Tooltip title="Collegium settings">
                <IconButton size="small" onClick={openDialog}>
                    <SettingsOutlined fontSize="small" />
                </IconButton>
            </Tooltip>
            <Dialog
                open={open}
                onClose={() => setOpen(false)}
                maxWidth="sm"
                fullWidth
            >
                <DialogTitle>Collegium settings</DialogTitle>
                <DialogContent
                    sx={{ display: "flex", flexDirection: "column", gap: 2 }}
                >
                    <TextField
                        fullWidth
                        label="Name"
                        value={name}
                        onChange={(e) => setName(e.target.value)}
                        sx={{ mt: 1 }}
                    />
                    <TextField
                        fullWidth
                        label="Description"
                        value={description}
                        onChange={(e) => setDescription(e.target.value)}
                        multiline
                        rows={2}
                    />
                    <ReviewVisibilityField
                        value={reviewVisibility}
                        onChange={setReviewVisibility}
                    />
                    {reviewVisibility !== collegium.review_visibility && (
                        <p className="text-xs text-stone-500">
                            Applies to future submissions only — every pending
                            or past submission keeps the visibility it was
                            submitted under.
                        </p>
                    )}
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
                                Private — hidden from Discover; people join via
                                invite link only.
                            </span>
                        }
                    />
                </DialogContent>
                <DialogActions>
                    <Button onClick={() => setOpen(false)}>Cancel</Button>
                    <Button
                        onClick={save}
                        variant="contained"
                        disabled={!name.trim() || updateMutation.isPending}
                    >
                        Save
                    </Button>
                </DialogActions>
            </Dialog>
        </>
    );
}
