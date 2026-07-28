import RateReviewOutlined from "@mui/icons-material/RateReviewOutlined";
import { Button, Chip } from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { Link, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { getGetUserArticleQueryKey } from "#/api/articles/articles";
import type { ArticleDetailResponse } from "#/api/model";
import { SubmitReviewDialog } from "./SubmitReviewDialog";

/**
 * The author's entry point to editorial review, rendered in the article
 * editor header: a "Request review" button while no request is pending,
 * or an "In review" chip linking to the review page.
 */
export function ReviewControls({
    article,
}: {
    article: ArticleDetailResponse;
}) {
    const [dialogOpen, setDialogOpen] = useState(false);
    const navigate = useNavigate();
    const queryClient = useQueryClient();

    if (article.status === "archived") return null;

    if (article.pending_review_request_id) {
        return (
            <Link
                to="/articles/review/$requestId"
                params={{ requestId: article.pending_review_request_id }}
                className="inline-flex items-center"
            >
                <Chip
                    label="In review — view"
                    size="small"
                    color="info"
                    clickable
                    sx={{ fontSize: "0.65rem", height: 20 }}
                />
            </Link>
        );
    }

    return (
        <>
            {article.latest_review_request_id && (
                <Link
                    to="/articles/review/$requestId"
                    params={{
                        requestId: article.latest_review_request_id,
                    }}
                    className="text-xs text-blue-500 hover:text-blue-700 underline"
                >
                    Past reviews
                </Link>
            )}
            <Button
                size="small"
                variant="outlined"
                startIcon={<RateReviewOutlined />}
                onClick={() => setDialogOpen(true)}
                sx={{ textTransform: "none" }}
            >
                Request review
            </Button>
            <SubmitReviewDialog
                open={dialogOpen}
                onClose={() => setDialogOpen(false)}
                articleSlug={article.slug}
                articleStatus={article.status}
                onSubmitted={(requestId) => {
                    setDialogOpen(false);
                    queryClient.invalidateQueries({
                        queryKey: getGetUserArticleQueryKey(article.slug),
                    });
                    navigate({
                        to: "/articles/review/$requestId",
                        params: { requestId },
                    });
                }}
            />
        </>
    );
}
