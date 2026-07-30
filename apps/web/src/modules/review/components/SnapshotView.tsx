import parse, {
    type DOMNode,
    domToReact,
    type Element,
    type HTMLReactParserOptions,
} from "html-react-parser";
import { ArticleQuotationCard, QuotationCard } from "../../quotation";

/** A sentence-range (or block-level, when start/end are null) anchor. */
export interface SnapshotSelection {
    block: number;
    start: number | null;
    end: number | null;
}

/** An existing comment's anchor, for highlighting. */
export interface AnchorMark {
    commentId: string;
    block: number;
    start: number | null;
    end: number | null;
    resolved: boolean;
}

interface SnapshotViewProps {
    html: string;
    marks: AnchorMark[];
    selection: SnapshotSelection | null;
    activeCommentId: string | null;
    /** Editors can create anchors; authors only focus existing ones. */
    canComment: boolean;
    onSentenceClick: (block: number, sentence: number, extend: boolean) => void;
    onBlockClick: (block: number) => void;
    onMarkClick: (commentId: string) => void;
}

function blockIndexOf(node: Element): number | null {
    let current: Element | null = node;
    while (current) {
        const value = current.attribs?.["data-block"];
        if (value !== undefined) return Number(value);
        current = (current.parent as Element | null) ?? null;
    }
    return null;
}

function marksCovering(
    marks: AnchorMark[],
    block: number,
    sentence: number | null,
): AnchorMark[] {
    return marks.filter((m) => {
        if (m.block !== block) return false;
        if (m.start == null || sentence == null) {
            return m.start == null;
        }
        return sentence >= m.start && sentence <= (m.end ?? m.start);
    });
}

/**
 * Renders a review snapshot (`data-block` / `data-s` annotated HTML)
 * with reader-style sentence interaction: click to select, shift-click
 * to extend, existing comment anchors highlighted, quotation embeds
 * hydrated like the public article page.
 */
export function SnapshotView({
    html,
    marks,
    selection,
    activeCommentId,
    canComment,
    onSentenceClick,
    onBlockClick,
    onMarkClick,
}: SnapshotViewProps) {
    const options: HTMLReactParserOptions = {
        replace: (domNode) => replace(domNode),
    };

    const replace = (domNode: DOMNode) => {
        if (domNode.type !== "tag") return undefined;
        const el = domNode as Element;
        const attrs = el.attribs ?? {};

        if (attrs.class?.includes("article-quotation-embed")) {
            const id = attrs["data-article-quotation-id"];
            const wrapped = id ? <ArticleQuotationCard id={id} /> : null;
            return wrapEmbed(el, wrapped);
        }
        if (attrs.class?.includes("quotation-embed")) {
            return wrapEmbed(
                el,
                <QuotationCard
                    book={attrs["data-quotation-book"] ?? ""}
                    node={attrs["data-quotation-node"] ?? ""}
                    start={Number(attrs["data-quotation-start"]) || 0}
                    end={
                        attrs["data-quotation-end"]
                            ? Number(attrs["data-quotation-end"])
                            : undefined
                    }
                    kind={attrs["data-quotation-kind"] ?? "body"}
                    mode={
                        (attrs["data-quotation-mode"] as
                            | "source"
                            | "translation"
                            | "source+translation") ?? "translation"
                    }
                    layout={
                        (attrs["data-quotation-layout"] as
                            | "stacked"
                            | "side-by-side-source-left"
                            | "side-by-side-source-right") ?? "stacked"
                    }
                />,
            );
        }

        if (attrs["data-s"] !== undefined) {
            const sentence = Number(attrs["data-s"]);
            const block = blockIndexOf(el);
            if (block === null) return undefined;

            const covering = marksCovering(marks, block, sentence);
            const openMarks = covering.filter((m) => !m.resolved);
            const isActive = covering.some(
                (m) => m.commentId === activeCommentId,
            );
            const isSelected =
                selection !== null &&
                selection.block === block &&
                selection.start !== null &&
                sentence >= selection.start &&
                sentence <= (selection.end ?? selection.start);

            const classes = ["review-sentence"];
            if (isSelected) classes.push("bg-sky-100");
            else if (isActive) classes.push("bg-amber-200");
            else if (openMarks.length > 0) classes.push("bg-amber-100");
            if (canComment || covering.length > 0)
                classes.push("cursor-pointer");

            return (
                <span
                    data-block={block}
                    data-s={sentence}
                    className={classes.join(" ")}
                    onMouseDown={(e) => {
                        // Shift-click extends the sentence selection;
                        // suppress the browser's native text-selection
                        // for that gesture.
                        if (canComment && e.shiftKey) e.preventDefault();
                    }}
                    onClick={(e) => {
                        e.stopPropagation();
                        if (canComment && (e.shiftKey || selection !== null)) {
                            onSentenceClick(block, sentence, e.shiftKey);
                            return;
                        }
                        if (openMarks.length > 0) {
                            onMarkClick(openMarks[0].commentId);
                            return;
                        }
                        if (covering.length > 0) {
                            onMarkClick(covering[0].commentId);
                            return;
                        }
                        if (canComment) {
                            onSentenceClick(block, sentence, e.shiftKey);
                        }
                    }}
                >
                    {domToReact(el.children as DOMNode[], options)}
                </span>
            );
        }

        return undefined;
    };

    /**
     * Embeds carry no sentence spans, so comments attach at block level.
     * The wrapper renders the highlight state and the click target.
     */
    function wrapEmbed(
        el: Element,
        child: React.JSX.Element | null,
    ): React.JSX.Element | undefined {
        const block = Number(el.attribs?.["data-block"]);
        if (Number.isNaN(block)) return child ?? undefined;

        const covering = marksCovering(marks, block, null);
        const openMarks = covering.filter((m) => !m.resolved);
        const isActive = covering.some((m) => m.commentId === activeCommentId);
        const isSelected =
            selection !== null &&
            selection.block === block &&
            selection.start === null;

        const classes = ["review-embed-block rounded"];
        if (isSelected) classes.push("ring-2 ring-sky-300");
        else if (isActive) classes.push("ring-2 ring-amber-300");
        else if (openMarks.length > 0) classes.push("ring-2 ring-amber-100");
        if (canComment || covering.length > 0) classes.push("cursor-pointer");

        return (
            <div
                data-block={block}
                className={classes.join(" ")}
                onClick={(e) => {
                    e.stopPropagation();
                    if (openMarks.length > 0 && !canComment) {
                        onMarkClick(openMarks[0].commentId);
                        return;
                    }
                    if (canComment) onBlockClick(block);
                    else if (covering.length > 0)
                        onMarkClick(covering[0].commentId);
                }}
            >
                {child}
            </div>
        );
    }

    return (
        <div className="prose prose-stone max-w-none">
            {parse(html, options)}
        </div>
    );
}
