import FavoriteBorderOutlined from "@mui/icons-material/FavoriteBorderOutlined";
import { Popover } from "@mui/material";
import { Link } from "@tanstack/react-router";
import parse, {
    attributesToProps,
    type DOMNode,
    Element,
    Text,
} from "html-react-parser";
import {
    createElement,
    type JSX,
    type MouseEvent,
    type ReactNode,
    useCallback,
    useEffect,
    useMemo,
    useRef,
    useState,
} from "react";
import toast from "react-hot-toast";
import { useCreateArticleQuotation } from "../../api/article-quotations/article-quotations";
import { FetchError } from "../../api/fetcher";
import { useAuth } from "../../hooks/useAuth";

interface SegmentedSentence {
    key: string;
    text: string;
    html: string;
}

interface SegmentRange {
    text: string;
    from: number;
    to: number;
}

const MAX_RANGE = 10;

function segmentText(text: string): string[] {
    if (typeof Intl !== "undefined" && "Segmenter" in Intl) {
        const segmenter = new Intl.Segmenter(undefined, {
            granularity: "sentence",
        });
        return Array.from(segmenter.segment(text), (s) => s.segment);
    }
    // Fallback: split on sentence-ending punctuation followed by space
    return text.split(/(?<=[.!?])\s+/).filter(Boolean);
}

/**
 * Sentence segments paired with their offsets into the block's text
 * stream, so the render pass can cut the block's DOM at the same points
 * `segmentText` cut its flattened text.
 */
function segmentRanges(text: string): SegmentRange[] {
    const ranges: SegmentRange[] = [];
    let cursor = 0;
    for (const segment of segmentText(text)) {
        const from = text.indexOf(segment, cursor);
        if (from < 0) continue;
        cursor = from + segment.length;
        ranges.push({ text: segment, from, to: cursor });
    }
    return ranges;
}

/**
 * Rebuild the part of `nodes` covering text offsets [from, to), keeping
 * inline elements (emphasis, code, links, citation spans) intact around
 * the cut. Without this a sentence would render as the bare text
 * `blockText` flattened it to, silently dropping all inline markup.
 * `cursor` tracks the position in the block's text stream across the
 * whole recursion.
 */
function sliceInline(
    nodes: DOMNode[],
    from: number,
    to: number,
    cursor: { pos: number },
): ReactNode[] {
    const out: ReactNode[] = [];

    nodes.forEach((node, i) => {
        if (node instanceof Text) {
            const start = cursor.pos;
            cursor.pos += node.data.length;
            const lo = Math.max(start, from);
            const hi = Math.min(cursor.pos, to);
            if (hi > lo) out.push(node.data.slice(lo - start, hi - start));
            return;
        }
        if (!(node instanceof Element)) return;

        const start = cursor.pos;
        const children = sliceInline(
            (node.children ?? []) as DOMNode[],
            from,
            to,
            cursor,
        );
        // Childless elements (<br>, <img>) carry no text to overlap the
        // range, so place them by their own offset instead.
        const empty = cursor.pos === start;
        if (children.length === 0 && !(empty && start >= from && start < to)) {
            return;
        }

        const props = { ...attributesToProps(node.attribs), key: `e${i}` };
        out.push(
            children.length > 0
                ? createElement(node.name, props, children)
                : createElement(node.name, props),
        );
    });

    return out;
}

const VOID_TAGS = new Set([
    "area",
    "base",
    "br",
    "col",
    "embed",
    "hr",
    "img",
    "input",
    "link",
    "meta",
    "source",
    "track",
    "wbr",
]);

function escapeText(value: string): string {
    return value
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;");
}

/**
 * String counterpart of `sliceInline`, for the `html` a saved quotation
 * stores — the reader sees emphasis and links in the article, so the
 * snapshot has to keep them too. The two must agree byte for byte; the
 * suite pins that against the rendered spans' `innerHTML`.
 */
function sliceHtml(
    nodes: DOMNode[],
    from: number,
    to: number,
    cursor: { pos: number },
): string {
    let out = "";

    for (const node of nodes) {
        if (node instanceof Text) {
            const start = cursor.pos;
            cursor.pos += node.data.length;
            const lo = Math.max(start, from);
            const hi = Math.min(cursor.pos, to);
            if (hi > lo)
                out += escapeText(node.data.slice(lo - start, hi - start));
            continue;
        }
        if (!(node instanceof Element)) continue;

        const start = cursor.pos;
        const inner = sliceHtml(
            (node.children ?? []) as DOMNode[],
            from,
            to,
            cursor,
        );
        const empty = cursor.pos === start;
        if (!inner && !(empty && start >= from && start < to)) continue;

        const attrs = Object.entries(node.attribs ?? {})
            .map(
                ([name, value]) =>
                    ` ${name}="${escapeText(value).replace(/"/g, "&quot;")}"`,
            )
            .join("");
        out += VOID_TAGS.has(node.name)
            ? `<${node.name}${attrs}>`
            : `<${node.name}${attrs}>${inner}</${node.name}>`;
    }

    return out;
}

function blockText(el: Element): string {
    const parts: string[] = [];
    const walk = (node: DOMNode) => {
        if (node instanceof Text) {
            parts.push(node.data);
        } else if (node instanceof Element) {
            for (const child of node.children || []) {
                walk(child as DOMNode);
            }
        }
    };
    for (const child of el.children || []) {
        walk(child as DOMNode);
    }
    return parts.join("");
}

/**
 * Containers the render pass replaces wholesale (whole blockquotes,
 * quotation-embed placeholders) — their inner `<p>`s never become blocks
 * of their own, so the flat sentence list must skip them too or the two
 * passes disagree on block numbering, shifting every key after the first
 * blockquote.
 */
function insideReplacedContainer(node: Element): boolean {
    let cur = node.parent;
    while (cur) {
        if (
            cur instanceof Element &&
            (cur.name === "blockquote" ||
                cur.attribs?.class?.includes("quotation-embed"))
        ) {
            return true;
        }
        cur = cur.parent;
    }
    return false;
}

/**
 * Flat sentence list keyed identically to the spans the component
 * renders (`b<block>-s<sentence>`). Exported for the pass-agreement
 * test — any keying drift between this and the render pass makes the
 * save-quotation preview show the wrong text.
 */
export function buildSentenceList(html: string): SegmentedSentence[] {
    const sentences: SegmentedSentence[] = [];
    let blockIndex = 0;

    parse(html, {
        replace: (domNode: DOMNode) => {
            if (domNode instanceof Element) {
                const tag = domNode.name;
                if (
                    (tag === "p" || tag === "blockquote") &&
                    !insideReplacedContainer(domNode)
                ) {
                    const children = (domNode.children ?? []) as DOMNode[];
                    let sentIdx = 0;
                    for (const range of segmentRanges(blockText(domNode))) {
                        const trimmed = range.text.trim();
                        if (!trimmed) continue;
                        sentences.push({
                            key: `b${blockIndex}-s${sentIdx}`,
                            text: trimmed,
                            html: sliceHtml(children, range.from, range.to, {
                                pos: 0,
                            }),
                        });
                        sentIdx++;
                    }
                    blockIndex++;
                }
            }
            return undefined;
        },
    });

    return sentences;
}

interface ArticleSentencesProps {
    html: string;
    articleId: string;
    replaceEmbed?: (domNode: Element) => JSX.Element | undefined;
    disabled?: boolean;
}

export function ArticleSentences({
    html,
    articleId,
    replaceEmbed,
    disabled = false,
}: ArticleSentencesProps) {
    const { isAuthenticated } = useAuth();
    const createMutation = useCreateArticleQuotation();

    const [selectedRange, setSelectedRange] = useState<{
        start: string;
        end: string | null;
    } | null>(null);
    const anchorRef = useRef<string | null>(null);
    const [popoverAnchor, setPopoverAnchor] = useState<HTMLElement | null>(
        null,
    );
    const [saveStatus, setSaveStatus] = useState<
        "idle" | "saving" | "saved" | "duplicate"
    >("idle");

    // Flat list of all sentences, keyed like the rendered spans below.
    const allSentences = useMemo(() => buildSentenceList(html), [html]);

    const sentenceKeys = useMemo(
        () => allSentences.map((s) => s.key),
        [allSentences],
    );

    const isInRange = useCallback(
        (key: string) => {
            if (!selectedRange) return false;
            if (!selectedRange.end) return key === selectedRange.start;
            const startIdx = sentenceKeys.indexOf(selectedRange.start);
            const endIdx = sentenceKeys.indexOf(selectedRange.end);
            const keyIdx = sentenceKeys.indexOf(key);
            const lo = Math.min(startIdx, endIdx);
            const hi = Math.max(startIdx, endIdx);
            return keyIdx >= lo && keyIdx <= hi;
        },
        [selectedRange, sentenceKeys],
    );

    const handleSentenceClick = useCallback(
        (key: string, e: MouseEvent) => {
            setSaveStatus("idle");

            if (e.shiftKey && anchorRef.current) {
                const anchorIdx = sentenceKeys.indexOf(anchorRef.current);
                const targetIdx = sentenceKeys.indexOf(key);
                if (
                    anchorIdx >= 0 &&
                    targetIdx >= 0 &&
                    Math.abs(targetIdx - anchorIdx) < MAX_RANGE
                ) {
                    const lo = Math.min(anchorIdx, targetIdx);
                    const hi = Math.max(anchorIdx, targetIdx);
                    setSelectedRange({
                        start: sentenceKeys[lo],
                        end: sentenceKeys[hi],
                    });
                    setPopoverAnchor(e.currentTarget as HTMLElement);
                    return;
                }
            }

            anchorRef.current = key;
            setSelectedRange({ start: key, end: null });
            setPopoverAnchor(e.currentTarget as HTMLElement);
        },
        [sentenceKeys],
    );

    const getSelectedText = useCallback(() => {
        if (!selectedRange) return { text: "", html: "" };
        const startIdx = sentenceKeys.indexOf(selectedRange.start);
        const endIdx = selectedRange.end
            ? sentenceKeys.indexOf(selectedRange.end)
            : startIdx;
        const lo = Math.min(startIdx, endIdx);
        const hi = Math.max(startIdx, endIdx);
        const selected = allSentences.slice(lo, hi + 1);
        const text = selected.map((s) => s.text).join(" ");
        // Each slice already carries the whitespace that followed its
        // sentence, so the pieces concatenate without a joiner.
        const html = selected
            .map((s) => s.html)
            .join("")
            .trim();
        return { text, html };
    }, [selectedRange, sentenceKeys, allSentences]);

    const handleSave = useCallback(async () => {
        const { text, html: selectedHtml } = getSelectedText();
        if (!text) return;

        setSaveStatus("saving");
        try {
            const result = await createMutation.mutateAsync({
                data: {
                    article_id: articleId,
                    text,
                    html: selectedHtml,
                },
            });
            if (
                result.data &&
                "created" in result.data &&
                result.data.created
            ) {
                setSaveStatus("saved");
            } else {
                setSaveStatus("duplicate");
            }
        } catch (err) {
            setSaveStatus("idle");
            const message =
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to save quotation";
            toast.error(message);
        }
    }, [getSelectedText, createMutation, articleId]);

    const handleClosePopover = useCallback(() => {
        setPopoverAnchor(null);
        setSelectedRange(null);
        setSaveStatus("idle");
    }, []);

    // Close popover on outside click
    useEffect(() => {
        const handler = (e: globalThis.MouseEvent) => {
            const target = e.target as HTMLElement;
            if (target.closest("[data-article-sentence]")) return;
            if (target.closest(".MuiPopover-root")) return;
            handleClosePopover();
        };
        document.addEventListener("mousedown", handler);
        return () => document.removeEventListener("mousedown", handler);
    }, [handleClosePopover]);

    // Render the HTML with sentence segmentation
    let blockIndex = 0;
    const rendered = parse(html, {
        replace: (domNode: DOMNode) => {
            if (!(domNode instanceof Element)) return undefined;
            const tag = domNode.name;

            // Delegate quotation embeds to the parent's replaceEmbed callback
            if (
                domNode.attribs?.class?.includes("quotation-embed") ||
                domNode.attribs?.class?.includes("article-quotation-embed")
            ) {
                return replaceEmbed?.(domNode) ?? undefined;
            }

            if (disabled) return undefined;

            if (tag === "p" || tag === "blockquote") {
                const children = (domNode.children ?? []) as DOMNode[];
                const ranges = segmentRanges(blockText(domNode));
                const currentBlock = blockIndex;
                blockIndex++;

                const Tag = tag as keyof React.JSX.IntrinsicElements;
                let sentIdx = 0;

                return (
                    <Tag>
                        {ranges.map((range) => {
                            if (!range.text.trim()) return null;
                            const key = `b${currentBlock}-s${sentIdx}`;
                            sentIdx++;
                            const selected = isInRange(key);

                            return (
                                <span
                                    key={key}
                                    data-article-sentence={key}
                                    onMouseDown={(e) => {
                                        if (e.shiftKey) e.preventDefault();
                                    }}
                                    onClick={(e) => {
                                        // Let embedded links navigate
                                        // instead of selecting the sentence.
                                        if (
                                            (e.target as HTMLElement).closest(
                                                "a",
                                            )
                                        ) {
                                            return;
                                        }
                                        handleSentenceClick(key, e);
                                    }}
                                    className={`cursor-pointer transition-colors rounded-sm ${
                                        selected
                                            ? "bg-amber-200"
                                            : "hover:bg-stone-100"
                                    }`}
                                >
                                    {sliceInline(
                                        children,
                                        range.from,
                                        range.to,
                                        { pos: 0 },
                                    )}
                                </span>
                            );
                        })}
                    </Tag>
                );
            }

            return undefined;
        },
    });

    const { text: selectedText } = getSelectedText();

    return (
        <>
            {rendered}
            <Popover
                open={!disabled && !!popoverAnchor}
                anchorEl={popoverAnchor}
                onClose={handleClosePopover}
                anchorOrigin={{ vertical: "bottom", horizontal: "center" }}
                transformOrigin={{ vertical: "top", horizontal: "center" }}
                slotProps={{
                    paper: {
                        sx: { mt: 1, maxWidth: 360 },
                    },
                }}
            >
                <div className="p-3">
                    <p className="text-xs text-stone-500 mb-2 line-clamp-3">
                        {selectedText}
                    </p>
                    {isAuthenticated ? (
                        <>
                            {saveStatus === "idle" && (
                                <button
                                    type="button"
                                    onClick={handleSave}
                                    className="w-full flex items-center justify-center gap-1.5 text-sm px-3 py-1.5 bg-amber-700 text-white rounded hover:bg-amber-800 transition-colors"
                                >
                                    <FavoriteBorderOutlined
                                        sx={{ fontSize: 16 }}
                                    />
                                    Save quotation
                                </button>
                            )}
                            {saveStatus === "saving" && (
                                <p className="text-xs text-stone-400 text-center">
                                    Saving...
                                </p>
                            )}
                            {saveStatus === "saved" && (
                                <p className="text-xs text-green-600 text-center">
                                    Quotation saved!
                                </p>
                            )}
                            {saveStatus === "duplicate" && (
                                <p className="text-xs text-amber-600 text-center">
                                    Already in your collection
                                </p>
                            )}
                        </>
                    ) : (
                        <p className="text-xs text-stone-500 text-center">
                            <Link
                                to="/login"
                                className="text-amber-700 underline"
                            >
                                Log in
                            </Link>{" "}
                            to save quotations
                        </p>
                    )}
                </div>
            </Popover>
        </>
    );
}
