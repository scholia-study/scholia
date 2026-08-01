import { Link } from "@tanstack/react-router";
import type { ArticleSeriesContext } from "../../../api/model";

/** "Part of ‹Series›" lines under an article's header. */
export function SeriesStrip({ series }: { series: ArticleSeriesContext[] }) {
    if (series.length === 0) return null;
    return (
        <div className="mb-8 space-y-1">
            {series.map((s) => (
                <p key={s.slug} className="text-sm text-stone-500">
                    Part of{" "}
                    <Link
                        to="/articles/series/$slug"
                        params={{ slug: s.slug }}
                        className="text-stone-700 no-underline hover:underline"
                    >
                        {s.name}
                    </Link>
                </p>
            ))}
        </div>
    );
}
