import OpenInNewOutlined from "@mui/icons-material/OpenInNewOutlined";
import { Paper, Skeleton } from "@mui/material";
import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import parse from "html-react-parser";
import { batchSentences } from "../api/sentences/sentences";

export interface QuotationCardProps {
    book: string;
    node: string;
    start: number;
    end?: number;
    kind: string;
    mode: "source" | "translation" | "source+translation";
    layout:
        | "stacked"
        | "side-by-side-source-left"
        | "side-by-side-source-right";
}

export function QuotationCard({
    book,
    node,
    start,
    end,
    kind,
    mode,
    layout,
}: QuotationCardProps) {
    const { data, isPending } = useQuery({
        queryKey: ["quotation-card", book, node, start, end, kind],
        queryFn: () =>
            batchSentences({
                items: [
                    {
                        book_slug: book,
                        node_slug: node,
                        start_number: start,
                        end_number: end,
                        kind,
                    },
                ],
            }),
        staleTime: 5 * 60 * 1000,
    });

    const item = data?.data?.items?.[0];

    if (isPending) {
        return (
            <Paper
                variant="outlined"
                sx={{ p: 2, my: 2, borderLeft: "3px solid rgb(214 211 209)" }}
            >
                <Skeleton variant="text" width="40%" height={16} />
                <Skeleton
                    variant="text"
                    width="100%"
                    height={20}
                    sx={{ mt: 1 }}
                />
                <Skeleton variant="text" width="80%" height={20} />
            </Paper>
        );
    }

    if (!item || item.sentences.length === 0) {
        return (
            <Paper
                variant="outlined"
                sx={{ p: 2, my: 2, borderLeft: "3px solid rgb(239 68 68)" }}
            >
                <p className="text-sm text-red-400 italic">
                    Quotation not found
                </p>
            </Paper>
        );
    }

    // Determine which content to show based on mode
    const showTranslation =
        mode === "translation" || mode === "source+translation";
    const showSource = mode === "source" || mode === "source+translation";

    // Build sentence HTML blocks
    const translationHtml = item.sentences.map((s) => s.html).join(" ");
    const sourceHtml = item.sentences
        .map((s) => s.original_html ?? s.html)
        .join(" ");

    const sentenceKey =
        end && end !== start ? `${start}-${end}` : String(start);

    return (
        <Paper
            variant="outlined"
            sx={{
                p: 2,
                my: 2,
                borderLeft: "3px solid rgb(168 162 158)",
                backgroundColor: "rgb(250 250 249)",
            }}
        >
            {/* Header */}
            <div className="flex items-center justify-between mb-2">
                <span className="text-xs text-stone-400">
                    {item.book_title} &middot; {item.node_label}
                </span>
                <Link
                    to="/books/$bookSlug/$nodeSlug"
                    params={{ bookSlug: book, nodeSlug: node }}
                    search={{ s: sentenceKey }}
                    target="_blank"
                    className="text-stone-400 hover:text-stone-600 transition-colors"
                    title="View in context"
                >
                    <OpenInNewOutlined sx={{ fontSize: 14 }} />
                </Link>
            </div>

            {/* Content */}
            {mode === "source+translation" && layout !== "stacked" ? (
                <div
                    className="grid grid-cols-2 gap-4"
                    style={{
                        direction:
                            layout === "side-by-side-source-right"
                                ? "rtl"
                                : "ltr",
                    }}
                >
                    <div
                        className="text-sm leading-relaxed text-stone-600 italic"
                        style={{
                            fontFamily: "'Libre Baskerville', serif",
                            direction: "ltr",
                        }}
                    >
                        {parse(sourceHtml)}
                    </div>
                    <div
                        className="text-sm leading-relaxed text-stone-700"
                        style={{
                            fontFamily: "'Libre Baskerville', serif",
                            direction: "ltr",
                        }}
                    >
                        {parse(translationHtml)}
                    </div>
                </div>
            ) : (
                <div>
                    {showSource && (
                        <div
                            className={`text-sm leading-relaxed ${
                                mode === "source+translation"
                                    ? "text-stone-600 italic mb-2"
                                    : "text-stone-700"
                            }`}
                            style={{ fontFamily: "'Libre Baskerville', serif" }}
                        >
                            {parse(sourceHtml)}
                        </div>
                    )}
                    {showTranslation && mode === "source+translation" && (
                        <hr className="my-2 border-stone-200" />
                    )}
                    {showTranslation && (
                        <div
                            className="text-sm leading-relaxed text-stone-700"
                            style={{ fontFamily: "'Libre Baskerville', serif" }}
                        >
                            {parse(translationHtml)}
                        </div>
                    )}
                </div>
            )}
        </Paper>
    );
}
