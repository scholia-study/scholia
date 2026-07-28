import CheckCircleOutlined from "@mui/icons-material/CheckCircleOutlined";
import ReplayOutlined from "@mui/icons-material/ReplayOutlined";
import SendOutlined from "@mui/icons-material/SendOutlined";
import { Button, Chip, IconButton, TextField, Tooltip } from "@mui/material";
import { useEffect, useState } from "react";
import type { ArticleReviewCommentResponse } from "#/api/model";
import type { SnapshotSelection } from "./SnapshotView";

interface CommentRailProps {
    comments: ArticleReviewCommentResponse[];
    /** Pending anchor for a new comment (editor selected sentences). */
    selection: SnapshotSelection | null;
    selectionQuote: string | null;
    activeCommentId: string | null;
    isReviewer: boolean;
    busy: boolean;
    onFocusComment: (comment: ArticleReviewCommentResponse) => void;
    onCreateComment: (body: string) => void;
    onCancelSelection: () => void;
    onReply: (commentId: string, body: string) => void;
    onSetResolved: (commentId: string, resolved: boolean) => void;
}

function anchorLabel(c: ArticleReviewCommentResponse): string {
    if (c.block_index == null) return "";
    if (c.sentence_start == null) return `¶${c.block_index + 1}`;
    const range =
        c.sentence_end != null && c.sentence_end !== c.sentence_start
            ? `${c.sentence_start + 1}–${c.sentence_end + 1}`
            : `${c.sentence_start + 1}`;
    return `¶${c.block_index + 1} · s. ${range}`;
}

function timestamp(iso: string): string {
    return new Date(iso).toLocaleDateString(undefined, {
        month: "short",
        day: "numeric",
    });
}

/**
 * The right-hand rail of the review page: a compose box while an anchor
 * is selected, then every comment thread ordered by document position.
 */
export function CommentRail({
    comments,
    selection,
    selectionQuote,
    activeCommentId,
    isReviewer,
    busy,
    onFocusComment,
    onCreateComment,
    onCancelSelection,
    onReply,
    onSetResolved,
}: CommentRailProps) {
    const [draft, setDraft] = useState("");
    const [filter, setFilter] = useState<"open" | "resolved" | "all">("open");

    const allTopLevel = comments
        .filter((c) => !c.parent_id)
        .sort((a, b) => {
            const blockDiff = (a.block_index ?? 0) - (b.block_index ?? 0);
            if (blockDiff !== 0) return blockDiff;
            return (a.sentence_start ?? -1) - (b.sentence_start ?? -1);
        });
    const topLevel = allTopLevel.filter((c) =>
        filter === "all"
            ? true
            : filter === "resolved"
              ? !!c.resolved_at
              : !c.resolved_at,
    );
    const openCount = allTopLevel.filter((c) => !c.resolved_at).length;
    const resolvedCount = allTopLevel.length - openCount;
    const repliesFor = (id: string) =>
        comments.filter((c) => c.parent_id === id);

    // Focusing a thread the current filter hides (e.g. clicking a
    // resolved highlight while on "Open") widens the filter so the
    // thread is actually visible.
    useEffect(() => {
        if (
            activeCommentId &&
            allTopLevel.some((c) => c.id === activeCommentId) &&
            !topLevel.some((c) => c.id === activeCommentId)
        ) {
            setFilter("all");
        }
    }, [activeCommentId, allTopLevel, topLevel]);

    const filters = [
        { key: "open", label: `Open (${openCount})` },
        { key: "resolved", label: `Resolved (${resolvedCount})` },
        { key: "all", label: "All" },
    ] as const;

    return (
        <div className="flex flex-col gap-3">
            {allTopLevel.length > 0 && (
                <div className="flex gap-1">
                    {filters.map((f) => (
                        <Chip
                            key={f.key}
                            label={f.label}
                            size="small"
                            variant={filter === f.key ? "filled" : "outlined"}
                            color={filter === f.key ? "primary" : "default"}
                            clickable
                            onClick={() => setFilter(f.key)}
                            sx={{ height: 22, fontSize: "0.65rem" }}
                        />
                    ))}
                </div>
            )}
            {selection && (
                <div className="border border-sky-200 bg-sky-50 rounded p-3">
                    <div className="text-xs uppercase tracking-wide text-sky-700 mb-1">
                        New comment
                    </div>
                    {selectionQuote && (
                        <blockquote className="text-xs text-stone-500 border-l-2 border-sky-200 pl-2 mb-2 line-clamp-3">
                            {selectionQuote}
                        </blockquote>
                    )}
                    <TextField
                        value={draft}
                        onChange={(e) => setDraft(e.target.value)}
                        onKeyDown={(e) => {
                            if (e.key === "Enter" && !e.shiftKey) {
                                e.preventDefault();
                                if (busy || !draft.trim()) return;
                                onCreateComment(draft.trim());
                                setDraft("");
                            }
                        }}
                        placeholder="Comment for the author…"
                        size="small"
                        multiline
                        minRows={2}
                        fullWidth
                        autoFocus
                    />
                    <div className="flex justify-end gap-2 mt-2">
                        <Button
                            size="small"
                            onClick={() => {
                                setDraft("");
                                onCancelSelection();
                            }}
                            disabled={busy}
                        >
                            Cancel
                        </Button>
                        <Button
                            size="small"
                            variant="contained"
                            disabled={busy || !draft.trim()}
                            onClick={() => {
                                onCreateComment(draft.trim());
                                setDraft("");
                            }}
                        >
                            Comment
                        </Button>
                    </div>
                </div>
            )}

            {topLevel.length === 0 && !selection && (
                <p className="text-sm text-stone-400">
                    {allTopLevel.length > 0
                        ? `No ${filter} comments.`
                        : isReviewer
                          ? "No comments yet. Click a sentence in the text to anchor one."
                          : "No comments yet."}
                </p>
            )}

            {topLevel.map((comment) => (
                <CommentThread
                    key={comment.id}
                    comment={comment}
                    replies={repliesFor(comment.id)}
                    active={comment.id === activeCommentId}
                    isReviewer={isReviewer}
                    busy={busy}
                    onFocus={() => onFocusComment(comment)}
                    onReply={(body) => onReply(comment.id, body)}
                    onSetResolved={(resolved) =>
                        onSetResolved(comment.id, resolved)
                    }
                />
            ))}
        </div>
    );
}

function CommentThread({
    comment,
    replies,
    active,
    isReviewer,
    busy,
    onFocus,
    onReply,
    onSetResolved,
}: {
    comment: ArticleReviewCommentResponse;
    replies: ArticleReviewCommentResponse[];
    active: boolean;
    isReviewer: boolean;
    busy: boolean;
    onFocus: () => void;
    onReply: (body: string) => void;
    onSetResolved: (resolved: boolean) => void;
}) {
    const [replyDraft, setReplyDraft] = useState("");
    const resolved = !!comment.resolved_at;

    const sendReply = () => {
        if (busy || !replyDraft.trim()) return;
        onReply(replyDraft.trim());
        setReplyDraft("");
    };

    return (
        <div
            data-comment-card={comment.id}
            className={`border rounded p-3 ${
                active
                    ? "border-amber-400 bg-amber-50"
                    : resolved
                      ? "border-stone-200 bg-stone-50 opacity-70"
                      : "border-stone-200 bg-white"
            }`}
        >
            <button
                type="button"
                className="w-full text-left"
                onClick={onFocus}
            >
                <div className="flex items-center justify-between gap-2 mb-1">
                    <span className="text-xs font-medium text-stone-700">
                        {comment.sender?.display_name ?? "Former member"}
                    </span>
                    <span className="flex items-center gap-1">
                        {resolved && (
                            <Chip
                                label="resolved"
                                size="small"
                                sx={{ height: 16, fontSize: "0.6rem" }}
                            />
                        )}
                        <span className="text-[0.65rem] text-stone-400">
                            {anchorLabel(comment)} ·{" "}
                            {timestamp(comment.created_at)}
                        </span>
                    </span>
                </div>
                {comment.quoted_text && (
                    <blockquote className="text-xs text-stone-500 border-l-2 border-stone-200 pl-2 mb-1 line-clamp-2">
                        {comment.quoted_text}
                    </blockquote>
                )}
                <p className="text-sm text-stone-800 whitespace-pre-wrap">
                    {comment.body}
                </p>
            </button>

            {replies.map((reply) => (
                <div
                    key={reply.id}
                    className="mt-2 ml-3 pl-2 border-l-2 border-stone-100"
                >
                    <div className="text-xs font-medium text-stone-700">
                        {reply.sender?.display_name ?? "Former member"}
                        <span className="ml-1 font-normal text-[0.65rem] text-stone-400">
                            {timestamp(reply.created_at)}
                        </span>
                    </div>
                    <p className="text-sm text-stone-800 whitespace-pre-wrap">
                        {reply.body}
                    </p>
                </div>
            ))}

            <div className="flex items-end gap-1 mt-2">
                <TextField
                    value={replyDraft}
                    onChange={(e) => setReplyDraft(e.target.value)}
                    onKeyDown={(e) => {
                        if (e.key === "Enter" && !e.shiftKey) {
                            e.preventDefault();
                            sendReply();
                        }
                    }}
                    placeholder="Reply…"
                    size="small"
                    multiline
                    maxRows={4}
                    fullWidth
                />
                <IconButton
                    size="small"
                    color="primary"
                    disabled={busy || !replyDraft.trim()}
                    onClick={sendReply}
                >
                    <SendOutlined sx={{ fontSize: 16 }} />
                </IconButton>
                {isReviewer && (
                    <Tooltip title={resolved ? "Reopen" : "Resolve"}>
                        <span>
                            <IconButton
                                size="small"
                                disabled={busy}
                                onClick={() => onSetResolved(!resolved)}
                            >
                                {resolved ? (
                                    <ReplayOutlined sx={{ fontSize: 16 }} />
                                ) : (
                                    <CheckCircleOutlined
                                        sx={{ fontSize: 16 }}
                                    />
                                )}
                            </IconButton>
                        </span>
                    </Tooltip>
                )}
            </div>
        </div>
    );
}
