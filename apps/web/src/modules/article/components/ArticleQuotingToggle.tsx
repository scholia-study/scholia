import FormatQuoteOutlined from "@mui/icons-material/FormatQuoteOutlined";
import { Chip, Tooltip } from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import toast from "react-hot-toast";
import {
    getGetPublishedArticleQueryKey,
    getGetUserArticleQueryKey,
    useSetArticleQuoting,
} from "#/api/articles/articles";
import { FetchError } from "#/api/fetcher";

interface ArticleQuotingToggleProps {
    articleSlug: string;
    /** Current state from the article response. */
    quotingDisabled: boolean;
}

/**
 * Admin-only switch for the reader's sentence-selection layer. Articles
 * that read as blog posts have no use for the quoting apparatus, so a
 * site admin can suppress it per article.
 *
 * Rendered both on the published article and in the author's editor, so
 * it refreshes both reads of the article; whichever one isn't cached
 * shrugs the invalidation off.
 *
 * Caller is responsible for gating visibility via permission
 * (`articles_quoting_manage`); the component assumes the right to click.
 */
export function ArticleQuotingToggle({
    articleSlug,
    quotingDisabled,
}: ArticleQuotingToggleProps) {
    const queryClient = useQueryClient();
    const mutation = useSetArticleQuoting();

    const handleClick = async () => {
        try {
            await mutation.mutateAsync({
                slug: articleSlug,
                data: { quoting_disabled: !quotingDisabled },
            });
            await queryClient.invalidateQueries({
                queryKey: getGetPublishedArticleQueryKey(articleSlug),
            });
            await queryClient.invalidateQueries({
                queryKey: getGetUserArticleQueryKey(articleSlug),
            });
            toast.success(
                quotingDisabled
                    ? "Quoting enabled for this article."
                    : "Quoting disabled for this article.",
            );
        } catch (err) {
            const message =
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to update quoting.";
            toast.error(message);
        }
    };

    return (
        <Tooltip
            title={
                quotingDisabled
                    ? "Readers cannot select sentences — click to allow quoting"
                    : "Readers can select and save sentences — click to turn quoting off"
            }
        >
            <Chip
                icon={<FormatQuoteOutlined sx={{ fontSize: "0.9rem" }} />}
                label={quotingDisabled ? "Quoting off" : "Quoting on"}
                size="small"
                variant="outlined"
                onClick={handleClick}
                disabled={mutation.isPending}
                sx={{
                    fontSize: "0.7rem",
                    borderStyle: "dashed",
                    cursor: "pointer",
                }}
            />
        </Tooltip>
    );
}
