import DeleteOutlined from "@mui/icons-material/DeleteOutlined";
import FormatQuoteOutlined from "@mui/icons-material/FormatQuoteOutlined";
import {
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    IconButton,
    Paper,
    Tooltip,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import parse from "html-react-parser";
import type { UnifiedQuotationResponse } from "#/api/model";
import {
    getListArticleQuotationsQueryKey,
    useDeleteArticleQuotation,
    useGetArticleQuotation,
} from "../../api/article-quotations/article-quotations";
import { getListAllQuotationsQueryKey } from "../../api/quotations/quotations";
import { FigureEmbed } from "./FigureEmbed";

export type ArticleQuotation = Extract<
    UnifiedQuotationResponse,
    { source_type: "article" }
>;

export function SavedArticleQuotation({
    quotation: q,
    onViewFull,
    onDelete,
}: {
    quotation: ArticleQuotation;
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
                "&:hover .card-actions": { opacity: 1 },
            }}
        >
            {q.figure_src?.startsWith("/media/") && (
                <img
                    src={q.figure_src}
                    alt={q.figure_alt ?? ""}
                    loading="lazy"
                    className="h-12 w-12 shrink-0 rounded border border-stone-200 object-cover"
                />
            )}
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
                    {q.figure_src ? (
                        q.text_snippet
                    ) : (
                        <>&ldquo;{q.text_snippet}&rdquo;</>
                    )}
                </p>
            </div>
            <div className="relative shrink-0 self-center">
                <div className="text-[10px] text-stone-300 text-right">
                    {new Date(q.created_at).toLocaleDateString(undefined, {
                        month: "short",
                        day: "numeric",
                        year: "numeric",
                    })}
                </div>
                <div className="card-actions absolute inset-0 flex items-center justify-end gap-0.5 bg-white opacity-0 transition-opacity">
                    <Tooltip title="View full quote">
                        <IconButton
                            size="small"
                            onClick={(e) => {
                                e.preventDefault();
                                e.stopPropagation();
                                onViewFull();
                            }}
                            sx={{ p: 0.5, color: "rgb(180 83 9)" }}
                        >
                            <FormatQuoteOutlined sx={{ fontSize: 16 }} />
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
                            sx={{ p: 0.5, color: "rgb(168 162 158)" }}
                        >
                            <DeleteOutlined sx={{ fontSize: 16 }} />
                        </IconButton>
                    </Tooltip>
                </div>
            </div>
        </Paper>
    );
}

export function ArticleQuotationDetailModal({
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
                        {" · "}
                        Saved{" "}
                        {new Date(quotation.created_at).toLocaleDateString(
                            undefined,
                            { month: "long", day: "numeric", year: "numeric" },
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
                            style={{ fontFamily: "'Libre Baskerville', serif" }}
                        >
                            {quotation.figure ? (
                                <FigureEmbed
                                    src={quotation.figure.src}
                                    alt={quotation.figure.alt ?? undefined}
                                    caption={
                                        quotation.figure.caption ?? undefined
                                    }
                                    width={quotation.figure.width ?? undefined}
                                    height={
                                        quotation.figure.height ?? undefined
                                    }
                                />
                            ) : (
                                parse(quotation.html) || null
                            )}
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
