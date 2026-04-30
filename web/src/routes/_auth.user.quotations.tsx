import DeleteOutlined from "@mui/icons-material/DeleteOutlined";
import FormatQuoteOutlined from "@mui/icons-material/FormatQuoteOutlined";
import {
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    FormControl,
    IconButton,
    InputLabel,
    MenuItem,
    Paper,
    Select,
    Tooltip,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute, Link } from "@tanstack/react-router";
import parse from "html-react-parser";
import { useMemo, useState } from "react";
import { useUnsaveQuotation } from "#/modules/quotation";
import {
    getListArticleQuotationsQueryKey,
    useDeleteArticleQuotation,
    useGetArticleQuotation,
} from "../api/article-quotations/article-quotations";
import type { UnifiedQuotationResponse } from "../api/model";
import {
    getListAllQuotationsQueryKey,
    useListAllQuotations,
} from "../api/quotations/quotations";

export const Route = createFileRoute("/_auth/user/quotations")({
    component: QuotationsPage,
});

type BookQuotation = Extract<UnifiedQuotationResponse, { source_type: "book" }>;
type ArticleQuotation = Extract<
    UnifiedQuotationResponse,
    { source_type: "article" }
>;

function sentenceLabel(q: BookQuotation): string {
    const start = q.anchor_sentence_start_number;
    const end = q.anchor_sentence_end_number;
    const isFootnote = q.sentence_kind === "footnote";
    const single = isFootnote ? "Footnote sentence" : "Sentence";
    const plural = isFootnote ? "Footnote sentences" : "Sentences";
    if (end == null || end === start) return `${single} ${start}`;
    return `${plural} ${start}\u2013${end}`;
}

function quotationLinkSearch(q: BookQuotation): {
    s: string;
    fs?: string;
    r: string;
    rv: string;
} {
    const startStr = String(q.anchor_sentence_start_number);
    const rangeStr =
        q.anchor_sentence_end_number &&
        q.anchor_sentence_end_number !== q.anchor_sentence_start_number
            ? `${q.anchor_sentence_start_number}-${q.anchor_sentence_end_number}`
            : startStr;
    if (q.sentence_kind === "footnote" && q.anchor_main_sentence_number) {
        return {
            s: String(q.anchor_main_sentence_number),
            fs: rangeStr,
            r: "1",
            rv: "notes",
        };
    }
    return { s: rangeStr, r: "1", rv: "notes" };
}

function QuotationsPage() {
    const [sourceFilter, setSourceFilter] = useState<string>("");
    const [selectedArticleQuotation, setSelectedArticleQuotation] = useState<
        string | null
    >(null);

    const { data: allQuotationsData, isLoading } = useListAllQuotations({});
    const allQuotations = allQuotationsData?.data?.quotations ?? [];
    const limits = allQuotationsData?.data?.limits;
    const showUsage = limits ? limits.max <= 50 : false;

    const bookQuotations = useMemo(
        () =>
            allQuotations.filter(
                (q): q is BookQuotation => q.source_type === "book",
            ),
        [allQuotations],
    );

    const articleQuotations = useMemo(
        () =>
            allQuotations.filter(
                (q): q is ArticleQuotation => q.source_type === "article",
            ),
        [allQuotations],
    );

    const availableBooks = useMemo(() => {
        const map = new Map<string, string>();
        for (const q of bookQuotations) {
            if (!map.has(q.book_slug)) {
                map.set(q.book_slug, q.book_title);
            }
        }
        return [...map.entries()].sort((a, b) => a[1].localeCompare(b[1]));
    }, [bookQuotations]);

    const filtered = useMemo(() => {
        if (!sourceFilter) return allQuotations;
        if (sourceFilter === "__articles__")
            return articleQuotations as UnifiedQuotationResponse[];
        return bookQuotations.filter(
            (q) => q.book_slug === sourceFilter,
        ) as UnifiedQuotationResponse[];
    }, [allQuotations, sourceFilter, bookQuotations, articleQuotations]);

    const { requestUnsave, UnsaveDialog } = useUnsaveQuotation({});

    const queryClient = useQueryClient();
    const deleteArticleQuotation = useDeleteArticleQuotation();
    const handleDeleteArticleQuotation = async (id: string) => {
        await deleteArticleQuotation.mutateAsync({ id });
        queryClient.invalidateQueries({
            queryKey: getListAllQuotationsQueryKey(),
        });
        queryClient.invalidateQueries({
            queryKey: getListArticleQuotationsQueryKey(),
        });
    };

    return (
        <div className="w-full max-w-3xl mx-auto px-8 py-16">
            <div className="flex items-center justify-between mb-2">
                <h1 className="text-2xl font-bold text-stone-900">
                    My Quotations
                </h1>
                <FormControl size="small" sx={{ minWidth: 200 }}>
                    <InputLabel>Filter by source</InputLabel>
                    <Select
                        value={sourceFilter}
                        label="Filter by source"
                        onChange={(e) => setSourceFilter(e.target.value)}
                    >
                        <MenuItem value="">All sources</MenuItem>
                        {articleQuotations.length > 0 && (
                            <MenuItem value="__articles__">
                                From articles
                            </MenuItem>
                        )}
                        {availableBooks.map(([slug, title]) => (
                            <MenuItem key={slug} value={slug}>
                                {title}
                            </MenuItem>
                        ))}
                    </Select>
                </FormControl>
            </div>
            {showUsage && limits && (
                <div className="text-xs text-stone-400 text-right mb-6">
                    {limits.current}/{limits.max} saved
                </div>
            )}
            {!showUsage && <div className="mb-6" />}

            {isLoading && <p className="text-sm text-stone-400">Loading...</p>}

            {!isLoading && filtered.length === 0 && (
                <p className="text-sm text-stone-400">
                    No saved quotations yet.
                </p>
            )}

            <div className="space-y-2">
                {filtered.map((q) =>
                    q.source_type === "book" ? (
                        <BookQuotationRow
                            key={q.id}
                            q={q}
                            requestUnsave={requestUnsave}
                        />
                    ) : (
                        <ArticleQuotationRow
                            key={q.id}
                            q={q}
                            onViewFull={() => setSelectedArticleQuotation(q.id)}
                            onDelete={() => handleDeleteArticleQuotation(q.id)}
                        />
                    ),
                )}
            </div>

            {UnsaveDialog}

            {selectedArticleQuotation && (
                <ArticleQuotationDetailModal
                    id={selectedArticleQuotation}
                    onClose={() => setSelectedArticleQuotation(null)}
                />
            )}
        </div>
    );
}

function BookQuotationRow({
    q,
    requestUnsave,
}: {
    q: BookQuotation;
    requestUnsave: (q: {
        id: string;
        book_slug: string;
        note_count: number;
    }) => void;
}) {
    return (
        <Paper
            elevation={0}
            sx={{
                border: "1px solid rgb(214 211 209)",
                borderLeft: "3px solid rgb(168 162 158)",
                p: 1.5,
                display: "flex",
                alignItems: "flex-start",
                gap: 1,
                transition: "box-shadow 0.15s",
                "&:hover": { boxShadow: 3 },
                "&:hover .action-btns": { opacity: 1 },
            }}
        >
            <Link
                to="/books/$bookSlug/$nodeSlug"
                params={{
                    bookSlug: q.book_slug,
                    nodeSlug: q.node_slug,
                }}
                search={quotationLinkSearch(q)}
                className="flex-1 min-w-0"
            >
                <div className="text-xs text-stone-400 mb-1">
                    {q.book_title} &middot; {q.node_label} &middot;{" "}
                    {sentenceLabel(q)}
                </div>
                {q.start_text_snippet && (
                    <p className="text-sm text-stone-700 truncate">
                        &ldquo;{q.start_text_snippet}&rdquo;
                        {q.end_text_snippet && (
                            <span className="text-stone-400">
                                {" "}
                                &hellip; &ldquo;{q.end_text_snippet}&rdquo;
                            </span>
                        )}
                    </p>
                )}
            </Link>
            <div className="relative shrink-0 self-center">
                <div className="text-right">
                    {q.note_count > 0 && (
                        <span className="text-xs text-stone-400">
                            {q.note_count} note{q.note_count > 1 ? "s" : ""}
                        </span>
                    )}
                    <div className="text-[10px] text-stone-300 mt-0.5">
                        {new Date(q.created_at).toLocaleDateString(undefined, {
                            month: "short",
                            day: "numeric",
                            year: "numeric",
                        })}
                    </div>
                </div>
                <div className="action-btns absolute inset-0 flex items-center justify-center gap-0.5 bg-white opacity-0 transition-opacity">
                    <IconButton
                        size="small"
                        onClick={() => requestUnsave(q)}
                        title="Remove quotation"
                        sx={{ color: "rgb(168 162 158)" }}
                    >
                        <DeleteOutlined fontSize="small" />
                    </IconButton>
                </div>
            </div>
        </Paper>
    );
}

function ArticleQuotationRow({
    q,
    onViewFull,
    onDelete,
}: {
    q: ArticleQuotation;
    onViewFull: () => void;
    onDelete: () => void;
}) {
    return (
        <Paper
            elevation={0}
            component={q.article_id ? "a" : "div"}
            {...(q.article_id
                ? { href: `/articles/by-id/${q.article_id}` }
                : {})}
            sx={{
                border: "1px solid rgb(214 211 209)",
                borderLeft: "3px solid rgb(180 83 9)",
                p: 1.5,
                display: "flex",
                alignItems: "flex-start",
                gap: 1,
                cursor: q.article_id ? "pointer" : "default",
                textDecoration: "none",
                color: "inherit",
                transition: "box-shadow 0.15s",
                "&:hover": { boxShadow: 3 },
                "&:hover .action-btns": { opacity: 1 },
            }}
        >
            <div className="flex-1 min-w-0">
                <div className="text-xs text-amber-700 mb-1">
                    {q.article_title} &middot; {q.author_display_name}
                    {!q.article_id && (
                        <span className="text-stone-400 italic">
                            {" "}
                            &middot; Article no longer available
                        </span>
                    )}
                </div>
                <p className="text-sm text-stone-700 truncate">
                    &ldquo;{q.text_snippet}&rdquo;
                </p>
            </div>
            <div className="relative shrink-0 self-center">
                <div className="text-right">
                    {q.note_count > 0 && (
                        <span className="text-xs text-stone-400">
                            {q.note_count} note{q.note_count > 1 ? "s" : ""}
                        </span>
                    )}
                    <div className="text-[10px] text-stone-300 mt-0.5">
                        {new Date(q.created_at).toLocaleDateString(undefined, {
                            month: "short",
                            day: "numeric",
                            year: "numeric",
                        })}
                    </div>
                </div>
                <div className="action-btns absolute inset-0 flex items-center justify-center gap-0.5 bg-white opacity-0 transition-opacity">
                    <Tooltip title="View full quote">
                        <IconButton
                            size="small"
                            onClick={(e) => {
                                e.preventDefault();
                                e.stopPropagation();
                                onViewFull();
                            }}
                            sx={{ color: "rgb(180 83 9)" }}
                        >
                            <FormatQuoteOutlined fontSize="small" />
                        </IconButton>
                    </Tooltip>
                    <Tooltip title="Delete quotation">
                        <IconButton
                            size="small"
                            onClick={(e) => {
                                e.preventDefault();
                                e.stopPropagation();
                                onDelete();
                            }}
                            sx={{ color: "rgb(168 162 158)" }}
                        >
                            <DeleteOutlined fontSize="small" />
                        </IconButton>
                    </Tooltip>
                </div>
            </div>
        </Paper>
    );
}

function ArticleQuotationDetailModal({
    id,
    onClose,
}: {
    id: string;
    onClose: () => void;
}) {
    const queryClient = useQueryClient();
    const { data, isPending } = useGetArticleQuotation(id);
    const quotation = data?.data ?? null;

    const deleteMutation = useDeleteArticleQuotation();

    const handleDelete = async () => {
        await deleteMutation.mutateAsync({ id });
        queryClient.invalidateQueries({
            queryKey: getListAllQuotationsQueryKey(),
        });
        queryClient.invalidateQueries({
            queryKey: getListArticleQuotationsQueryKey(),
        });
        onClose();
    };

    return (
        <Dialog open onClose={onClose} maxWidth="sm" fullWidth>
            <DialogTitle sx={{ pb: 1 }}>
                {isPending
                    ? "Loading..."
                    : quotation
                      ? quotation.article_title
                      : "Quotation not found"}
            </DialogTitle>
            {quotation && (
                <DialogContent>
                    <div className="text-xs text-stone-400 mb-3">
                        {quotation.author_display_name}
                        {!quotation.article_id && (
                            <span className="italic">
                                {" "}
                                &middot; Article no longer available
                            </span>
                        )}
                        {" \u00B7 "}
                        Saved{" "}
                        {new Date(quotation.created_at).toLocaleDateString(
                            undefined,
                            {
                                month: "long",
                                day: "numeric",
                                year: "numeric",
                            },
                        )}
                    </div>
                    <Paper
                        variant="outlined"
                        sx={{
                            p: 2,
                            borderLeft: "3px solid rgb(180 83 9)",
                            backgroundColor: "rgb(255 255 255)",
                        }}
                    >
                        <div
                            className="text-sm leading-relaxed text-stone-700"
                            style={{
                                fontFamily: "'Libre Baskerville', serif",
                            }}
                        >
                            {parse(quotation.html) || null}
                        </div>
                    </Paper>
                    {quotation.article_id && (
                        <div className="mt-3">
                            <a
                                href={`/articles/by-id/${quotation.article_id}`}
                                className="text-xs text-amber-700 hover:underline"
                            >
                                View source article
                            </a>
                        </div>
                    )}
                </DialogContent>
            )}
            <DialogActions sx={{ px: 3, pb: 2 }}>
                <Button
                    onClick={handleDelete}
                    size="small"
                    color="error"
                    startIcon={<DeleteOutlined />}
                    disabled={deleteMutation.isPending}
                >
                    Delete
                </Button>
                <div className="flex-1" />
                <Button onClick={onClose} size="small">
                    Close
                </Button>
            </DialogActions>
        </Dialog>
    );
}
