import { Paper } from "@mui/material";
import { Link } from "@tanstack/react-router";
import type { ArticleResponse } from "#/api/model";

interface ArticleCardProps {
    article: ArticleResponse;
    /**
     * Render the author name in the meta row. Default `true`. Set
     * `false` on contexts where the author is implicit (e.g. on the
     * user's own public profile page).
     */
    showAuthor?: boolean;
}

/**
 * Shared article preview card. Used in the public articles listing
 * and on the public user profile. Square-edged Paper with elevation
 * on hover; meta row holds author (optional) · date · topics.
 */
export function ArticleCard({ article, showAuthor = true }: ArticleCardProps) {
    const metaParts: Array<{ key: string; node: React.ReactNode }> = [];
    if (showAuthor) {
        metaParts.push({
            key: "author",
            node: <span>{article.author_display_name}</span>,
        });
    }
    if (article.published_at) {
        metaParts.push({
            key: "date",
            node: (
                <span>
                    {new Date(article.published_at).toLocaleDateString(
                        undefined,
                        { month: "long", day: "numeric", year: "numeric" },
                    )}
                </span>
            ),
        });
    }
    for (const t of article.topics) {
        metaParts.push({
            key: `topic-${t.id}`,
            node: <span>{t.name}</span>,
        });
    }

    return (
        <Link
            to="/articles/$slug"
            params={{ slug: article.slug }}
            className="block group"
        >
            <Paper
                component="article"
                square
                elevation={2}
                sx={{
                    p: 2.5,
                    transition: "box-shadow 0.15s",
                    "&:hover": {
                        boxShadow:
                            "3px 5px 10px -1px rgba(0,0,0,0.16), 2px 2px 3px 0 rgba(0,0,0,0.10)",
                    },
                }}
            >
                <h2 className="text-lg font-semibold text-stone-900 group-hover:underline mb-1">
                    {article.title}
                </h2>
                {article.description && (
                    <p className="text-sm text-stone-500 mb-2 line-clamp-2">
                        {article.description}
                    </p>
                )}
                {metaParts.length > 0 && (
                    <div className="flex items-center gap-2 text-xs text-stone-400 flex-wrap">
                        {metaParts.map((part, i) => (
                            <span key={part.key} className="contents">
                                {i > 0 ? <span>&middot;</span> : null}
                                {part.node}
                            </span>
                        ))}
                    </div>
                )}
            </Paper>
        </Link>
    );
}
