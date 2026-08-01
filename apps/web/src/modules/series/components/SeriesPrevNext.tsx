import { Link } from "@tanstack/react-router";
import type { ArticleSeriesContext } from "../../../api/model";

/** Prev/next navigation after an article's body, one block per series
 * with at least one published neighbor. Direction follows series
 * position order (position 1 first). */
export function SeriesPrevNext({ series }: { series: ArticleSeriesContext[] }) {
    const withNeighbors = series.filter((s) => s.prev || s.next);
    if (withNeighbors.length === 0) return null;
    return (
        <div className="mt-12 space-y-6">
            {withNeighbors.map((s) => (
                <nav
                    key={s.slug}
                    aria-label={`More in ${s.name}`}
                    className="border-t border-stone-200 pt-4"
                >
                    <p className="text-xs uppercase tracking-wide text-stone-400 mb-2">
                        <Link
                            to="/articles/series/$slug"
                            params={{ slug: s.slug }}
                            className="text-stone-400 no-underline hover:text-stone-600 hover:underline"
                        >
                            {s.name}
                        </Link>
                    </p>
                    <div className="flex justify-between gap-6 text-sm">
                        {s.prev ? (
                            <Link
                                to="/articles/$slug"
                                params={{ slug: s.prev.slug }}
                                className="text-stone-600 no-underline hover:underline"
                            >
                                ← {s.prev.title}
                            </Link>
                        ) : (
                            <span />
                        )}
                        {s.next ? (
                            <Link
                                to="/articles/$slug"
                                params={{ slug: s.next.slug }}
                                className="text-right text-stone-600 no-underline hover:underline"
                            >
                                {s.next.title} →
                            </Link>
                        ) : (
                            <span />
                        )}
                    </div>
                </nav>
            ))}
        </div>
    );
}
