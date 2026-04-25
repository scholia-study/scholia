import FavoriteBorderOutlined from "@mui/icons-material/FavoriteBorderOutlined";
import { Popover } from "@mui/material";
import { Link } from "@tanstack/react-router";
import parse, { type DOMNode, Element, Text } from "html-react-parser";
import {
    type JSX,
    type MouseEvent,
    useCallback,
    useEffect,
    useMemo,
    useRef,
    useState,
} from "react";
import toast from "react-hot-toast";
import { useCreateArticleQuotation } from "../api/article-quotations/article-quotations";
import { FetchError } from "../api/fetcher";
import { useAuth } from "../hooks/useAuth";

interface SegmentedSentence {
    key: string;
    text: string;
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

    // Build a flat list of all sentences from all paragraphs
    const allSentences = useMemo(() => {
        const sentences: SegmentedSentence[] = [];
        let blockIndex = 0;

        // Parse to find <p> and <blockquote> elements
        parse(html, {
            replace: (domNode: DOMNode) => {
                if (domNode instanceof Element) {
                    const tag = domNode.name;
                    if (tag === "p" || tag === "blockquote") {
                        // Extract text content
                        const textParts: string[] = [];
                        const extractText = (node: DOMNode) => {
                            if (node instanceof Text) {
                                textParts.push(node.data);
                            } else if (node instanceof Element) {
                                for (const child of node.children || []) {
                                    extractText(child as DOMNode);
                                }
                            }
                        };
                        for (const child of domNode.children || []) {
                            extractText(child as DOMNode);
                        }
                        const fullText = textParts.join("");
                        const blockSentences = segmentText(fullText);

                        for (let i = 0; i < blockSentences.length; i++) {
                            const trimmed = blockSentences[i].trim();
                            if (!trimmed) continue;
                            sentences.push({
                                key: `b${blockIndex}-s${i}`,
                                text: trimmed,
                            });
                        }
                        blockIndex++;
                    }
                }
                return undefined;
            },
        });

        return sentences;
    }, [html]);

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
        return { text, html: text };
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
                const textParts: string[] = [];
                const extractText = (node: DOMNode) => {
                    if (node instanceof Text) {
                        textParts.push(node.data);
                    } else if (node instanceof Element) {
                        for (const child of node.children || []) {
                            extractText(child as DOMNode);
                        }
                    }
                };
                for (const child of domNode.children || []) {
                    extractText(child as DOMNode);
                }
                const fullText = textParts.join("");
                const segments = segmentText(fullText);
                const currentBlock = blockIndex;
                blockIndex++;

                const Tag = tag as keyof React.JSX.IntrinsicElements;
                let sentIdx = 0;

                return (
                    <Tag>
                        {segments.map((segment) => {
                            const trimmed = segment.trim();
                            if (!trimmed) return null;
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
                                    onClick={(e) => handleSentenceClick(key, e)}
                                    className={`cursor-pointer transition-colors rounded-sm ${
                                        selected
                                            ? "bg-amber-200"
                                            : "hover:bg-stone-100"
                                    }`}
                                >
                                    {segment}
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
