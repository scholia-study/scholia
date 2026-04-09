import { Paper, Skeleton } from "@mui/material";
import parse from "html-react-parser";
import { useGetArticleQuotation } from "../api/article-quotations/article-quotations";

export interface ArticleQuotationCardProps {
    id: string;
}

export function ArticleQuotationCard({ id }: ArticleQuotationCardProps) {
    const { data, isPending } = useGetArticleQuotation(id);
    const quotation = data?.data;

    if (isPending) {
        return (
            <Paper
                variant="outlined"
                sx={{ p: 2, my: 2, borderLeft: "3px solid rgb(180 83 9)" }}
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

    if (!quotation) {
        return (
            <Paper
                variant="outlined"
                sx={{ p: 2, my: 2, borderLeft: "3px solid rgb(239 68 68)" }}
            >
                <p className="text-sm text-red-400 italic">
                    Article quotation not found
                </p>
            </Paper>
        );
    }

    return (
        <Paper
            variant="outlined"
            sx={{
                p: 2,
                my: 2,
                borderLeft: "3px solid rgb(180 83 9)",
                backgroundColor: "#fff",
            }}
        >
            <div
                className="text-sm leading-relaxed text-stone-700"
                style={{ fontFamily: "'Libre Baskerville', serif" }}
            >
                {parse(quotation.html)}
            </div>
            <div className="flex justify-end mt-1">
                {quotation.article_id ? (
                    <a
                        href={`/articles/by-id/${quotation.article_id}`}
                        target="_blank"
                        rel="noreferrer"
                        className="text-xs text-amber-700 no-underline hover:underline transition-colors"
                    >
                        {quotation.article_title} &middot;{" "}
                        {quotation.author_display_name}
                    </a>
                ) : (
                    <span className="text-xs text-stone-400 italic">
                        {quotation.article_title} &middot;{" "}
                        {quotation.author_display_name} &middot; Article no
                        longer available
                    </span>
                )}
            </div>
        </Paper>
    );
}
