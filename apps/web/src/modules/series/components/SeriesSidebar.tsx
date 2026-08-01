import PushPinOutlined from "@mui/icons-material/PushPinOutlined";
import { TextField } from "@mui/material";
import { Link } from "@tanstack/react-router";
import { useState } from "react";
import type { SeriesResponse } from "../../../api/model";

const PAGE_SIZE = 8;

/** Right-column series directory on the articles landing page: local
 * search over the full (pinned-first) list, paged client-side. */
export function SeriesSidebar({ series }: { series: SeriesResponse[] }) {
    const [query, setQuery] = useState("");
    const [page, setPage] = useState(0);

    const q = query.trim().toLowerCase();
    const filtered = q
        ? series.filter((s) => s.name.toLowerCase().includes(q))
        : series;
    const pageCount = Math.max(1, Math.ceil(filtered.length / PAGE_SIZE));
    const clamped = Math.min(page, pageCount - 1);
    const visible = filtered.slice(
        clamped * PAGE_SIZE,
        clamped * PAGE_SIZE + PAGE_SIZE,
    );

    if (series.length === 0) return null;

    return (
        <div>
            <h2 className="text-xs uppercase tracking-wide text-stone-400 mb-3">
                Series
            </h2>
            <TextField
                placeholder="Search series…"
                value={query}
                onChange={(e) => {
                    setQuery(e.target.value);
                    setPage(0);
                }}
                size="small"
                fullWidth
                sx={{ mb: 1.5 }}
                slotProps={{ input: { style: { fontSize: "0.85rem" } } }}
            />
            {visible.length === 0 && (
                <p className="text-xs text-stone-400">No matching series.</p>
            )}
            <ul className="space-y-0.5">
                {visible.map((s) => (
                    <li key={s.id}>
                        <Link
                            to="/articles/series/$slug"
                            params={{ slug: s.slug }}
                            className="flex items-center gap-1.5 px-2 py-1.5 rounded no-underline text-sm text-stone-600 hover:text-stone-900 hover:bg-stone-100"
                        >
                            {s.pinned && (
                                <PushPinOutlined
                                    sx={{ fontSize: 13 }}
                                    className="text-stone-400 shrink-0"
                                />
                            )}
                            <span className="flex-1 min-w-0 truncate">
                                {s.name}
                            </span>
                            <span className="text-xs text-stone-400 tabular-nums shrink-0">
                                {s.article_count}
                            </span>
                        </Link>
                    </li>
                ))}
            </ul>
            {pageCount > 1 && (
                <div className="flex items-center justify-between mt-2 text-xs text-stone-400">
                    <button
                        type="button"
                        disabled={clamped === 0}
                        onClick={() => setPage(clamped - 1)}
                        className="px-2 py-1 rounded enabled:cursor-pointer enabled:hover:bg-stone-100 disabled:opacity-40"
                        aria-label="Previous series page"
                    >
                        ←
                    </button>
                    <span className="tabular-nums">
                        {clamped + 1} / {pageCount}
                    </span>
                    <button
                        type="button"
                        disabled={clamped >= pageCount - 1}
                        onClick={() => setPage(clamped + 1)}
                        className="px-2 py-1 rounded enabled:cursor-pointer enabled:hover:bg-stone-100 disabled:opacity-40"
                        aria-label="Next series page"
                    >
                        →
                    </button>
                </div>
            )}
        </div>
    );
}
