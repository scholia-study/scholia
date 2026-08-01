import { Chip } from "@mui/material";
import { Link } from "@tanstack/react-router";
import type { SeriesResponse } from "../../../api/model";

/** Compact swipeable series row for small screens (pinned first, from
 * the server order), capped with an "All series" link. */
export function SeriesChipRow({ series }: { series: SeriesResponse[] }) {
    if (series.length === 0) return null;
    return (
        <div className="flex gap-1.5 overflow-x-auto pb-1 mb-6">
            {series.map((s) => (
                <Link
                    key={s.id}
                    to="/articles/series/$slug"
                    params={{ slug: s.slug }}
                    className="shrink-0"
                >
                    <Chip
                        label={s.name}
                        size="small"
                        variant="outlined"
                        clickable
                        sx={{ fontSize: "0.75rem" }}
                    />
                </Link>
            ))}
        </div>
    );
}
