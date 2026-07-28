import { useEffect, useState } from "react";
import type { ArticleReviewCommentResponse } from "#/api/model";

interface CommentConnectorProps {
    /** The positioned (`relative`) element containing both the snapshot
     * and the comment rail; the SVG overlay fills it. */
    containerRef: React.RefObject<HTMLDivElement | null>;
    comments: ArticleReviewCommentResponse[];
    activeCommentId: string | null;
}

/**
 * Draws a curve from the focused comment's highlighted sentences to its
 * card in the rail (Docs-style). Only the active comment gets a line —
 * one per comment would need collision layout and reads as clutter.
 */
export function CommentConnector({
    containerRef,
    comments,
    activeCommentId,
}: CommentConnectorProps) {
    const [path, setPath] = useState<string | null>(null);

    useEffect(() => {
        if (!activeCommentId) {
            setPath(null);
            return;
        }

        const update = () => {
            const root = containerRef.current;
            const comment = comments.find(
                (c) => c.id === activeCommentId && !c.parent_id,
            );
            if (!root || !comment || comment.block_index == null) {
                setPath(null);
                return;
            }
            const anchorEl = root.querySelector(
                comment.sentence_start != null
                    ? `[data-block="${comment.block_index}"] [data-s="${comment.sentence_start}"], [data-block="${comment.block_index}"][data-s="${comment.sentence_start}"]`
                    : `[data-block="${comment.block_index}"]`,
            );
            const cardEl = root.querySelector(
                `[data-comment-card="${activeCommentId}"]`,
            );
            if (!anchorEl || !cardEl) {
                setPath(null);
                return;
            }
            const rootRect = root.getBoundingClientRect();
            const a = anchorEl.getBoundingClientRect();
            const c = cardEl.getBoundingClientRect();
            const x1 = a.right - rootRect.left + 4;
            const y1 = a.top + a.height / 2 - rootRect.top;
            const x2 = c.left - rootRect.left - 4;
            const y2 = c.top + 14 - rootRect.top;
            const bend = Math.min(80, Math.max(24, (x2 - x1) / 3));
            setPath(
                `M ${x1} ${y1} C ${x1 + bend} ${y1}, ${x2 - bend} ${y2}, ${x2} ${y2}`,
            );
        };

        update();
        // The app scrolls inside <main>, not the window — capture-phase
        // listening catches that scroll without knowing the container.
        window.addEventListener("scroll", update, true);
        window.addEventListener("resize", update);
        const observer = new ResizeObserver(update);
        if (containerRef.current) observer.observe(containerRef.current);
        return () => {
            window.removeEventListener("scroll", update, true);
            window.removeEventListener("resize", update);
            observer.disconnect();
        };
    }, [activeCommentId, comments, containerRef]);

    if (!path) return null;

    return (
        <svg
            className="pointer-events-none absolute inset-0 w-full h-full hidden lg:block"
            aria-hidden="true"
        >
            <path
                d={path}
                fill="none"
                stroke="rgb(217 119 6)"
                strokeWidth="1.5"
                strokeDasharray="5 4"
                opacity="0.6"
            />
            <circle
                cx={Number(path.split(" ")[1])}
                cy={Number(path.split(" ")[2])}
                r="3"
                fill="rgb(217 119 6)"
                opacity="0.6"
            />
        </svg>
    );
}
