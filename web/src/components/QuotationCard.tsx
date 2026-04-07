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
        .map((s) => s.original_html)
        .filter(Boolean)
        .join(" ") || null;

    const sentenceKey =
        end && end !== start ? `${start}-${end}` : String(start);

    const isFootnote = kind === "footnote";
    const prefix = isFootnote ? "fn. s." : "s.";
    const sentenceLabel =
        end && end !== start
            ? `${prefix} ${start}\u2013${end}`
            : `${prefix} ${start}`;

    const srcBook = item.source ?? {
        book_slug: book,
        book_title: item.book_title,
        node_slug: node,
        node_label: item.node_label,
    };
    const sourceAttribution = showSource && (
        <div className="flex justify-end mt-1">
            <Link
                to="/books/$bookSlug/$nodeSlug"
                params={{
                    bookSlug: srcBook.book_slug,
                    nodeSlug: srcBook.node_slug,
                }}
                search={{ s: sentenceKey }}
                target="_blank"
                className="!text-xs !text-stone-400 !no-underline hover:!underline !transition-colors"
            >
                {srcBook.book_title} &middot; {srcBook.node_label} &middot;{" "}
                {sentenceLabel}
            </Link>
        </div>
    );

    const translationAttribution = showTranslation && (
        <div className="flex justify-end mt-1">
            <Link
                to="/books/$bookSlug/$nodeSlug"
                params={{ bookSlug: book, nodeSlug: node }}
                search={{ s: sentenceKey }}
                target="_blank"
                className="!text-xs !text-stone-400 !no-underline hover:!underline !transition-colors"
            >
                {item.book_title} &middot; {item.node_label} &middot;{" "}
                {sentenceLabel}
            </Link>
        </div>
    );

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
            {mode === "source+translation" &&
            layout !== "stacked" &&
            sourceHtml ? (
                <div
                    className="grid grid-cols-2 gap-4"
                    style={{
                        direction:
                            layout === "side-by-side-source-right"
                                ? "rtl"
                                : "ltr",
                    }}
                >
                    <div className="flex flex-col" style={{ direction: "ltr" }}>
                        <div
                            className="text-sm leading-relaxed text-stone-600"
                            style={{
                                fontFamily: "'Libre Baskerville', serif",
                            }}
                        >
                            {parse(sourceHtml)}
                        </div>
                        <div className="mt-auto">{sourceAttribution}</div>
                    </div>
                    <div className="flex flex-col" style={{ direction: "ltr" }}>
                        <div
                            className="text-sm leading-relaxed text-stone-700"
                            style={{
                                fontFamily: "'Libre Baskerville', serif",
                            }}
                        >
                            {parse(translationHtml)}
                        </div>
                        <div className="mt-auto">{translationAttribution}</div>
                    </div>
                </div>
            ) : (
                <div>
                    {showSource && sourceHtml && (
                        <>
                            <div
                                className={`text-sm leading-relaxed ${
                                    mode === "source+translation"
                                        ? "text-stone-600"
                                        : "text-stone-700"
                                }`}
                                style={{
                                    fontFamily: "'Libre Baskerville', serif",
                                }}
                            >
                                {parse(sourceHtml)}
                            </div>
                            {sourceAttribution}
                        </>
                    )}
                    {showTranslation &&
                        mode === "source+translation" &&
                        sourceHtml && (
                            <hr className="my-2 border-stone-200" />
                        )}
                    {showTranslation && (
                        <>
                            <div
                                className="text-sm leading-relaxed text-stone-700"
                                style={{
                                    fontFamily: "'Libre Baskerville', serif",
                                }}
                            >
                                {parse(translationHtml)}
                            </div>
                            {translationAttribution}
                        </>
                    )}
                </div>
            )}
        </Paper>
    );
}
