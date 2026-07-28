import {
    Button,
    Chip,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    TextField,
    Tooltip,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useEffect, useRef, useState } from "react";
import toast from "react-hot-toast";
import {
    getGetReviewRequestQueryKey,
    getListReviewCommentsQueryKey,
    getListReviewMessagesQueryKey,
    useCreateReviewComment,
    useCreateReviewMessage,
    useCreateReviewReply,
    useDecideArticleReview,
    useGetReviewActivity,
    useGetReviewRequest,
    useListReviewComments,
    useListReviewMessages,
    useUpdateReviewComment,
    useWithdrawReviewRequest,
} from "#/api/article-reviews/article-reviews";
import { FetchError } from "#/api/fetcher";
import type { ArticleReviewCommentResponse } from "#/api/model";
import { useAuth } from "#/hooks/useAuth";
import { ChannelPanel } from "./ChannelPanel";
import { CommentConnector } from "./CommentConnector";
import { CommentRail } from "./CommentRail";
import {
    type AnchorMark,
    type SnapshotSelection,
    SnapshotView,
} from "./SnapshotView";

const STATUS_COLORS: Record<
    string,
    "warning" | "success" | "error" | "default"
> = {
    pending: "warning",
    approved: "success",
    declined: "error",
    resolved: "default",
    withdrawn: "default",
};

function errorMessage(err: unknown, fallback: string): string {
    return err instanceof FetchError && err.message ? err.message : fallback;
}

/**
 * The shared review page: the frozen snapshot with anchored comment
 * threads, the per-article conversation, and the decision actions.
 * Rendered for both the author and reviewers; the API enforces access.
 */
export function ReviewPage({ requestId }: { requestId: string }) {
    const queryClient = useQueryClient();
    const { user, hasPermission } = useAuth();
    const isReviewer = hasPermission("articles_review");

    const { data: detailData } = useGetReviewRequest(requestId);
    const detail = detailData?.data;
    const { data: commentsData } = useListReviewComments(requestId);
    const comments = commentsData?.data?.comments ?? [];
    const articleId = detail?.article.id;
    const { data: messagesData } = useListReviewMessages(articleId ?? "", {
        query: { enabled: !!articleId },
    });
    const messages = messagesData?.data?.messages ?? [];

    // Page-local polling: a cheap activity stamp every 15s; when it
    // changes, invalidate the real queries. Stops when the page unmounts
    // and pauses in background tabs (React Query default).
    const { data: activityData } = useGetReviewActivity(articleId ?? "", {
        query: { enabled: !!articleId, refetchInterval: 15_000 },
    });
    const activityToken = activityData
        ? JSON.stringify(activityData.data)
        : null;
    const lastActivityToken = useRef<string | null>(null);
    useEffect(() => {
        if (activityToken === null || !articleId) return;
        if (
            lastActivityToken.current !== null &&
            lastActivityToken.current !== activityToken
        ) {
            queryClient.invalidateQueries({
                queryKey: getListReviewMessagesQueryKey(articleId),
            });
            queryClient.invalidateQueries({
                queryKey: getListReviewCommentsQueryKey(requestId),
            });
            queryClient.invalidateQueries({
                queryKey: getGetReviewRequestQueryKey(requestId),
            });
        }
        lastActivityToken.current = activityToken;
    }, [activityToken, articleId, requestId, queryClient]);

    const commentMutation = useCreateReviewComment();
    const replyMutation = useCreateReviewReply();
    const resolveMutation = useUpdateReviewComment();
    const messageMutation = useCreateReviewMessage();
    const decideMutation = useDecideArticleReview();
    const withdrawMutation = useWithdrawReviewRequest();

    const [selection, setSelection] = useState<SnapshotSelection | null>(null);
    const [activeCommentId, setActiveCommentId] = useState<string | null>(null);
    const [decision, setDecision] = useState<
        "approved" | "declined" | "resolved" | null
    >(null);
    const [decisionMessage, setDecisionMessage] = useState("");
    const snapshotRef = useRef<HTMLDivElement>(null);
    const layoutRef = useRef<HTMLDivElement>(null);

    if (!detail) {
        return (
            <div className="flex-1 bg-white">
                <div className="max-w-[88rem] mx-auto px-8 py-16">
                    <p className="text-sm text-stone-400">Loading…</p>
                </div>
            </div>
        );
    }

    const request = detail.request;
    const isOwner = user?.id === detail.article.author_user_id;
    const isPending = request.status === "pending";
    const canComment = isReviewer && request.status !== "withdrawn";

    const invalidateComments = () =>
        queryClient.invalidateQueries({
            queryKey: getListReviewCommentsQueryKey(requestId),
        });
    const invalidateMessages = () =>
        articleId &&
        queryClient.invalidateQueries({
            queryKey: getListReviewMessagesQueryKey(articleId),
        });

    const marks: AnchorMark[] = comments
        .filter((c) => !c.parent_id && c.block_index != null)
        .map((c) => ({
            commentId: c.id,
            block: c.block_index as number,
            start: c.sentence_start ?? null,
            end: c.sentence_end ?? null,
            resolved: !!c.resolved_at,
        }));

    const quoteForSelection = (sel: SnapshotSelection): string | null => {
        const root = snapshotRef.current;
        if (!root) return null;
        if (sel.start == null) {
            const blockEl = root.querySelector(`[data-block="${sel.block}"]`);
            return blockEl?.textContent?.trim().slice(0, 500) ?? null;
        }
        const parts: string[] = [];
        for (let s = sel.start; s <= (sel.end ?? sel.start); s++) {
            const span = root.querySelector(
                `[data-block="${sel.block}"] [data-s="${s}"], span[data-block="${sel.block}"][data-s="${s}"]`,
            );
            if (span?.textContent) parts.push(span.textContent.trim());
        }
        return parts.length ? parts.join(" ").slice(0, 500) : null;
    };

    const handleSentenceClick = (
        block: number,
        sentence: number,
        extend: boolean,
    ) => {
        setActiveCommentId(null);
        setSelection((prev) => {
            if (extend && prev && prev.block === block && prev.start !== null) {
                return {
                    block,
                    start: Math.min(prev.start, sentence),
                    end: Math.max(prev.end ?? prev.start, sentence),
                };
            }
            return { block, start: sentence, end: sentence };
        });
    };

    const handleBlockClick = (block: number) => {
        setActiveCommentId(null);
        setSelection({ block, start: null, end: null });
    };

    const focusComment = (comment: ArticleReviewCommentResponse) => {
        setSelection(null);
        setActiveCommentId(comment.id);
        if (comment.block_index == null) return;
        const root = snapshotRef.current;
        const target =
            comment.sentence_start != null
                ? root?.querySelector(
                      `[data-block="${comment.block_index}"][data-s="${comment.sentence_start}"], [data-block="${comment.block_index}"] [data-s="${comment.sentence_start}"]`,
                  )
                : root?.querySelector(`[data-block="${comment.block_index}"]`);
        target?.scrollIntoView({ behavior: "smooth", block: "center" });
    };

    const createComment = async (body: string) => {
        if (!selection) return;
        try {
            await commentMutation.mutateAsync({
                id: requestId,
                data: {
                    block_index: selection.block,
                    sentence_start: selection.start ?? undefined,
                    sentence_end:
                        selection.start != null
                            ? (selection.end ?? selection.start)
                            : undefined,
                    quoted_text: quoteForSelection(selection) ?? undefined,
                    body,
                },
            });
            setSelection(null);
            invalidateComments();
        } catch (err) {
            toast.error(errorMessage(err, "Failed to create comment"));
        }
    };

    const reply = async (commentId: string, body: string) => {
        try {
            await replyMutation.mutateAsync({ id: commentId, data: { body } });
            invalidateComments();
        } catch (err) {
            toast.error(errorMessage(err, "Failed to reply"));
        }
    };

    const setResolved = async (commentId: string, resolved: boolean) => {
        try {
            await resolveMutation.mutateAsync({
                id: commentId,
                data: { resolved },
            });
            invalidateComments();
        } catch (err) {
            toast.error(errorMessage(err, "Failed to update comment"));
        }
    };

    const sendMessage = async (body: string) => {
        if (!articleId) return;
        try {
            await messageMutation.mutateAsync({ articleId, data: { body } });
            invalidateMessages();
        } catch (err) {
            toast.error(errorMessage(err, "Failed to send message"));
        }
    };

    const decide = async () => {
        if (!decision) return;
        try {
            await decideMutation.mutateAsync({
                id: requestId,
                data: {
                    status: decision,
                    message: decisionMessage.trim()
                        ? decisionMessage.trim()
                        : undefined,
                },
            });
            toast.success(
                decision === "approved"
                    ? "Request approved."
                    : decision === "declined"
                      ? "Request declined."
                      : "Request resolved.",
            );
            setDecision(null);
            setDecisionMessage("");
            queryClient.invalidateQueries({
                queryKey: getGetReviewRequestQueryKey(requestId),
            });
            invalidateMessages();
        } catch (err) {
            toast.error(errorMessage(err, "Failed to decide request"));
        }
    };

    const withdraw = async () => {
        try {
            await withdrawMutation.mutateAsync({ id: requestId });
            toast.success("Request withdrawn.");
            queryClient.invalidateQueries({
                queryKey: getGetReviewRequestQueryKey(requestId),
            });
        } catch (err) {
            toast.error(errorMessage(err, "Failed to withdraw request"));
        }
    };

    const publishesOnApprove =
        request.intent === "publication" && detail.article.status === "draft";

    return (
        <div className="flex-1 bg-white">
            <div className="max-w-[88rem] mx-auto px-8 py-12">
                <header className="mb-6">
                    <div className="flex flex-wrap items-center gap-2 mb-2">
                        <h1 className="text-2xl font-bold text-stone-900 mr-2">
                            {detail.article.title}
                        </h1>
                        <Chip
                            label={request.intent}
                            size="small"
                            variant="outlined"
                            sx={{ height: 20, fontSize: "0.65rem" }}
                        />
                        <Chip
                            label={request.status}
                            size="small"
                            color={STATUS_COLORS[request.status] ?? "default"}
                            sx={{ height: 20, fontSize: "0.65rem" }}
                        />
                        <span className="text-xs text-stone-400">
                            submitted{" "}
                            {new Date(
                                request.submitted_at,
                            ).toLocaleDateString()}{" "}
                            by {detail.article.author_display_name}
                        </span>
                    </div>
                    <div className="flex flex-wrap items-center gap-3">
                        {isOwner && (
                            <Link
                                to="/user/articles/$slug"
                                params={{ slug: detail.article.slug }}
                                className="text-xs text-blue-500 hover:text-blue-700 underline"
                            >
                                Open in editor
                            </Link>
                        )}
                        {detail.article.status !== "draft" && (
                            <Link
                                to="/articles/$slug"
                                params={{ slug: detail.article.slug }}
                                target="_blank"
                                className="text-xs text-blue-500 hover:text-blue-700 underline"
                            >
                                View published
                            </Link>
                        )}
                        {detail.requests.length > 1 && (
                            <span className="text-xs text-stone-400">
                                Rounds:{" "}
                                {detail.requests
                                    .slice()
                                    .reverse()
                                    .map((r, i) =>
                                        r.id === requestId ? (
                                            <span
                                                key={r.id}
                                                className="font-semibold text-stone-600"
                                            >
                                                {i + 1}{" "}
                                            </span>
                                        ) : (
                                            <Link
                                                key={r.id}
                                                to="/articles/review/$requestId"
                                                params={{ requestId: r.id }}
                                                className="underline mr-1"
                                            >
                                                {i + 1}
                                            </Link>
                                        ),
                                    )}
                            </span>
                        )}
                        <span className="flex-1" />
                        {isOwner && isPending && (
                            <Tooltip
                                title="Cancels this review request. If it's your only submission, editors lose access to your draft until you submit again. The conversation is kept."
                                arrow
                            >
                                <span>
                                    <Button
                                        size="small"
                                        variant="outlined"
                                        color="inherit"
                                        disabled={withdrawMutation.isPending}
                                        onClick={withdraw}
                                        sx={{ textTransform: "none" }}
                                    >
                                        Withdraw
                                    </Button>
                                </span>
                            </Tooltip>
                        )}
                        {isReviewer && isPending && (
                            <>
                                {request.intent === "publication" ? (
                                    <>
                                        <Button
                                            size="small"
                                            color="error"
                                            variant="outlined"
                                            onClick={() =>
                                                setDecision("declined")
                                            }
                                            sx={{ textTransform: "none" }}
                                        >
                                            Decline
                                        </Button>
                                        <Button
                                            size="small"
                                            variant="contained"
                                            onClick={() =>
                                                setDecision("approved")
                                            }
                                            sx={{ textTransform: "none" }}
                                        >
                                            Approve
                                        </Button>
                                    </>
                                ) : (
                                    <Button
                                        size="small"
                                        variant="contained"
                                        onClick={() => setDecision("resolved")}
                                        sx={{ textTransform: "none" }}
                                    >
                                        Mark resolved
                                    </Button>
                                )}
                            </>
                        )}
                    </div>
                    {detail.draft_changed && (
                        <div className="mt-3 px-3 py-2 bg-amber-50 border border-amber-200 rounded text-sm text-amber-800">
                            The article has been edited since this snapshot was
                            submitted. Comments refer to the submitted version.
                        </div>
                    )}
                </header>

                <div
                    ref={layoutRef}
                    className="relative grid grid-cols-1 lg:grid-cols-[360px_minmax(0,1fr)_320px] xl:grid-cols-[420px_minmax(0,1fr)_320px] gap-8"
                >
                    <CommentConnector
                        containerRef={layoutRef}
                        comments={comments}
                        activeCommentId={activeCommentId}
                    />
                    <aside className="order-3 lg:order-1">
                        <div className="lg:sticky lg:top-4 lg:h-full lg:min-h-96 lg:max-h-[70dvh]">
                            <ChannelPanel
                                messages={messages}
                                viewerUserId={user?.id ?? null}
                                busy={messageMutation.isPending}
                                onSend={sendMessage}
                            />
                        </div>
                    </aside>
                    <div className="order-1 lg:order-2" ref={snapshotRef}>
                        <SnapshotView
                            html={detail.snapshot_html}
                            marks={marks}
                            selection={selection}
                            activeCommentId={activeCommentId}
                            canComment={canComment}
                            onSentenceClick={handleSentenceClick}
                            onBlockClick={handleBlockClick}
                            onMarkClick={(id) => {
                                const c = comments.find((x) => x.id === id);
                                if (c) focusComment(c);
                            }}
                        />
                    </div>
                    <aside className="order-2 lg:order-3">
                        <div className="lg:sticky lg:top-4">
                            <div className="text-xs uppercase tracking-wide text-stone-400 mb-2">
                                Comments
                            </div>
                            <CommentRail
                                comments={comments}
                                selection={selection}
                                selectionQuote={
                                    selection
                                        ? quoteForSelection(selection)
                                        : null
                                }
                                activeCommentId={activeCommentId}
                                isReviewer={canComment}
                                busy={
                                    commentMutation.isPending ||
                                    replyMutation.isPending ||
                                    resolveMutation.isPending
                                }
                                onFocusComment={focusComment}
                                onCreateComment={createComment}
                                onCancelSelection={() => setSelection(null)}
                                onReply={reply}
                                onSetResolved={setResolved}
                            />
                        </div>
                    </aside>
                </div>
            </div>

            <Dialog
                open={decision !== null}
                onClose={() => setDecision(null)}
                maxWidth="xs"
                fullWidth
            >
                <DialogTitle sx={{ fontSize: 16 }}>
                    {decision === "approved"
                        ? "Approve this article?"
                        : decision === "declined"
                          ? "Decline this request?"
                          : "Mark this request resolved?"}
                </DialogTitle>
                <DialogContent
                    sx={{ display: "flex", flexDirection: "column", gap: 1.5 }}
                >
                    {decision === "approved" && (
                        <p className="text-sm text-stone-600">
                            {publishesOnApprove
                                ? "This will publish the article on the author's behalf and apply the Imprimatur label."
                                : "This will apply the Imprimatur label to the published article."}
                        </p>
                    )}
                    {decision === "declined" && (
                        <p className="text-sm text-stone-600">
                            The article stays as it is. Use the message to tell
                            the author why — they can revise and resubmit.
                        </p>
                    )}
                    {decision === "resolved" && (
                        <p className="text-sm text-stone-600">
                            Closes this feedback round. The conversation stays
                            open.
                        </p>
                    )}
                    <TextField
                        label="Message to the author (optional)"
                        value={decisionMessage}
                        onChange={(e) => setDecisionMessage(e.target.value)}
                        size="small"
                        multiline
                        rows={3}
                        fullWidth
                    />
                </DialogContent>
                <DialogActions sx={{ px: 3, pb: 2 }}>
                    <Button
                        size="small"
                        onClick={() => setDecision(null)}
                        disabled={decideMutation.isPending}
                    >
                        Cancel
                    </Button>
                    <Button
                        size="small"
                        variant="contained"
                        color={decision === "declined" ? "error" : "primary"}
                        onClick={decide}
                        disabled={decideMutation.isPending}
                    >
                        {decision === "approved"
                            ? "Approve"
                            : decision === "declined"
                              ? "Decline"
                              : "Resolve"}
                    </Button>
                </DialogActions>
            </Dialog>
        </div>
    );
}
