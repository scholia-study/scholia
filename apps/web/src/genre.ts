// The library's colour scheme. Genre gives a work its hue, the era of
// composition gives that hue its depth. Shared by the library index (shelf
// spines and catalogue rules) and the reader's toolbar, so a work carries the
// same colour from the shelf into the text.

export type Genre = "philosophy" | "poetry" | "drama" | "unclassified";
export type Era = "ancient" | "medieval" | "earlyModern" | "modern";

/**
 * A work reads paler and more faded the older it is, deeper and more
 * saturated the nearer it stands to us.
 *
 * Each ramp is solved backwards from a contrast target rather than set to a
 * common lightness — green is perceptually far lighter than navy at the same
 * lightness, so matching lightness would have left the pale greens unable to
 * carry white lettering. Every step here clears 4.5:1 against white (ancient
 * ~5:1 rising to ~11:1 for modern), which means an era step reads as the same
 * step in every genre and no surface tinted with one needs dark text.
 */
export const GENRE_RAMPS: Record<Genre, Record<Era, string>> = {
    philosophy: {
        ancient: "#53729a",
        medieval: "#3e5f89",
        earlyModern: "#2c4d78",
        modern: "#1d3c64",
    },
    poetry: {
        ancient: "#427a5e",
        medieval: "#2f674b",
        earlyModern: "#20563b",
        modern: "#14442c",
    },
    drama: {
        ancient: "#a45963",
        medieval: "#95434e",
        earlyModern: "#83303b",
        modern: "#6d202a",
    },
    // Deliberately drab: an unmapped author should look unplaced rather than
    // quietly borrow a genre it may not belong to.
    unclassified: {
        ancient: "#8a8178",
        medieval: "#736a61",
        earlyModern: "#5c544c",
        modern: "#454039",
    },
};

/** Scripture stands outside the scheme, and outside the era ramp with it. */
export const SCRIPTURE_ACCENT = "#4b0082";

/**
 * The sheen that keeps a tint from reading as flat paint: a shaded near edge,
 * a highlight a fifth of the way in, then a long fall to a darker far edge —
 * a surface curving away from the light rather than a rectangle of colour.
 *
 * Spines are lit across their width and the reader's toolbar down its height,
 * so both take the same stops turned ninety degrees. Being one definition,
 * the bar and the book it belongs to are the same material by construction.
 */
function sheen(direction: string): string {
    return `linear-gradient(${direction}, rgba(0,0,0,0.18), rgba(255,255,255,0.14) 18%, rgba(0,0,0,0.05) 55%, rgba(0,0,0,0.26))`;
}

export const SPINE_SHEEN = sheen("to right");
export const BAR_SHEEN = sheen("to bottom");

/**
 * Stopgap until genre is carried on the source itself: no endpoint reports a
 * work's genre, so the mapping lives here, keyed by author. Anything absent
 * falls through to `unclassified` rather than being guessed at.
 */
const GENRE_BY_AUTHOR: Record<string, Genre> = {
    "Georg Wilhelm Friedrich Hegel": "philosophy",
    "Immanuel Kant": "philosophy",
    "Thomas Hobbes": "philosophy",
    "John Milton": "poetry",
    "William Shakespeare": "poetry",
    "Henrik Ibsen": "drama",
};

/**
 * Scripture is recognised two ways because the two callers see different
 * things. The library index knows the compilation by its group heading, while
 * a Bible book reached through the reader reports no author at all — the
 * translation is the work — so there the slug is the only handle.
 */
const SCRIPTURE_LABELS = new Set(["The Bible"]);

function isScripture(authorLabel: string, slug?: string): boolean {
    return (
        SCRIPTURE_LABELS.has(authorLabel) || (!!slug && /-bible$/.test(slug))
    );
}

export function eraOf(year: number | null | undefined): Era {
    if (!year) return "modern";
    if (year < 500) return "ancient";
    if (year < 1450) return "medieval";
    if (year < 1800) return "earlyModern";
    return "modern";
}

export function accentFor(
    authorLabel: string,
    year: number | null | undefined,
    slug?: string,
): string {
    if (isScripture(authorLabel, slug)) return SCRIPTURE_ACCENT;
    return GENRE_RAMPS[GENRE_BY_AUTHOR[authorLabel] ?? "unclassified"][
        eraOf(year)
    ];
}

/**
 * A tint seen through frosted glass: the accent lifted toward white so a
 * surface can sit a shade above the toolbar without leaving the work's
 * colour. Returns the composited colour rather than layering a translucent
 * fill, so `inkOn` weighs the lettering against what is actually seen.
 */
export function glassOver(hex: string, alpha = 0.14): string {
    const h = hex.replace("#", "");
    const mixed = [0, 2, 4].map((i) => {
        const channel = Number.parseInt(h.slice(i, i + 2), 16);
        return Math.round(channel * (1 - alpha) + 255 * alpha);
    });
    return `#${mixed.map((c) => c.toString(16).padStart(2, "0")).join("")}`;
}

function relativeLuminance(hex: string): number {
    const h = hex.replace("#", "");
    const channels = [0, 2, 4].map((i) => {
        const v = Number.parseInt(h.slice(i, i + 2), 16) / 255;
        return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
    });
    return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

/**
 * Lettering that stays legible on a tinted surface. Every colour above is
 * dark enough for white, but the ramps' pale ends are reserved for ancient
 * works we do not hold yet — so this decides rather than assumes, and a light
 * tint arriving later gets dark ink instead of silently becoming unreadable.
 */
export function inkOn(background: string): { text: string; muted: string } {
    const contrastWithWhite = 1.05 / (relativeLuminance(background) + 0.05);
    return contrastWithWhite >= 4.5
        ? { text: "#ffffff", muted: "rgba(255,255,255,0.72)" }
        : { text: "#1c1917", muted: "rgba(28,25,23,0.62)" };
}
