import { Chip } from "@mui/material";
import { Link } from "@tanstack/react-router";
import type { EditorialLabelResponse } from "#/api/model";

/**
 * Mapping from label slug to MUI Chip color. Defaults to `info` for any
 * label not explicitly listed — keeps newly-seeded labels visible without
 * a UI change. Tuned for the seeded set (Featured = warning/gold-ish,
 * High Quality = success/green).
 */
const LABEL_COLOR: Record<
    string,
    "default" | "primary" | "secondary" | "success" | "warning" | "info"
> = {
    featured: "warning",
    "high-quality": "success",
};

interface EditorialLabelChipsProps {
    labels: EditorialLabelResponse[];
    /**
     * When true, each chip is wrapped in a `Link` to `/articles?label=<slug>`
     * so readers can drill into the filtered listing. Disabled on the
     * listing page itself (no point linking to where you already are).
     */
    clickable?: boolean;
    /** Override MUI Chip size. Defaults to small. */
    size?: "small" | "medium";
}

/** Renders the editorial-label chip row for an article. Returns null on empty. */
export function EditorialLabelChips({
    labels,
    clickable = true,
    size = "small",
}: EditorialLabelChipsProps) {
    if (labels.length === 0) return null;
    return (
        <div className="flex flex-wrap items-center gap-1.5">
            {labels.map((l) => {
                const chip = (
                    <Chip
                        label={l.name}
                        size={size}
                        color={LABEL_COLOR[l.slug] ?? "info"}
                        sx={{
                            fontSize: size === "small" ? "0.7rem" : "0.8rem",
                        }}
                    />
                );
                if (!clickable) {
                    return <span key={l.id}>{chip}</span>;
                }
                return (
                    <Link
                        key={l.id}
                        to="/articles"
                        search={{ label_slug: l.slug }}
                    >
                        {chip}
                    </Link>
                );
            })}
        </div>
    );
}
