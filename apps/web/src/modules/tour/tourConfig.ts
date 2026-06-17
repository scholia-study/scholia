import type { LibraryResponse, TocNodeResponse } from "#/api/model";
import { LOC_STORAGE_KEYS } from "#/hooks/local-storage";

/** The tour is pinned to the KJV Bible — the foundational seed text, always
 *  ingested first (see CLAUDE.md). Only the book slug is hardcoded; the entry
 *  node and verse anchors are resolved/standard, so re-slugged TOC nodes don't
 *  break the tour. */
export const TOUR_BOOK_SLUG = "kjv-bible";

/** localStorage flag — set the first time the welcome prompt is shown so it
 *  never auto-nags again (the tour stays replayable from the resources panel). */
export const TOUR_SEEN_KEY = LOC_STORAGE_KEYS.readerTourSeen;

/** Verse-number keys used as anchors. Every chapter starts at verse 1, and
 *  "1-3" demonstrates a range — both safe across any KJV entry node. */
const SINGLE = "1";
const RANGE = "1-3";

/** Search params the tour sets per step (a subset of the reader's ReaderSearch).
 *  Setting these replaces the panel state, driving the UI without faking clicks. */
export type TourSearch = { s?: string; r?: string; rv?: string };

export interface TourStep {
    /** Reader URL search to apply before the step is shown. */
    search: TourSearch;
    /** CSS selector the step anchors to; omit for a centered popover. */
    element?: string;
    popover: {
        title: string;
        description: string;
        side?: "top" | "right" | "bottom" | "left" | "over";
        align?: "start" | "center" | "end";
    };
}

/** The ordered reader tour. Each step navigates the reader into the state it
 *  describes (`s` = selected sentence, `r` = resources open), then anchors a
 *  popover to a stable DOM contract (`data-sentence-key` / `data-tour`). */
export function buildTourSteps(): TourStep[] {
    return [
        {
            search: {},
            popover: {
                title: "Welcome to the Scholia reader",
                description:
                    "This is where you read and work with a text. Here, for example, is the Book of Genesis. Let's walk through what you can do.",
            },
        },
        {
            search: { s: SINGLE },
            element: `[data-sentence-key="${SINGLE}"]`,
            popover: {
                title: "Select a sentence",
                description:
                    "Click any sentence to select it. Everything else—cross-references, notes, quotations—keys off your selection.",
                side: "right",
                align: "start",
            },
        },
        {
            search: { s: RANGE },
            element: `[data-sentence-key="3"]`,
            popover: {
                title: "Select a range",
                description:
                    "Shift-click a second sentence to select everything in between. Handy for quoting a longer passage!",
                side: "right",
                align: "start",
            },
        },
        {
            search: { s: RANGE, r: "1" },
            element: `[data-tour="resources-panel"]`,
            popover: {
                title: "The resources panel",
                description:
                    "Selecting text opens this panel. It collects everything tied to your selection in one place.",
                side: "left",
                align: "start",
            },
        },
        {
            search: { s: RANGE, r: "1" },
            element: `[data-tour="toc"]`,
            popover: {
                title: "Jump anywhere",
                description:
                    "While you can scroll up and down freely across a text, open the Table of Contents to jump straight to any chapter or section.",
                side: "left",
                align: "start",
            },
        },
        {
            search: { s: RANGE, r: "1" },
            element: `[data-tour="commentary"]`,
            popover: {
                title: "Cross-references",
                description:
                    "See where other works quote, paraphrase, or allude to the selected passage—grouped by how closely they echo it.",
                side: "left",
                align: "start",
            },
        },
        {
            search: { s: RANGE, r: "1" },
            element: `[data-tour="tools"]`,
            popover: {
                title: "Save & annotate",
                description:
                    "Save a selection as a quotation or attach your own notes. These live in your account and can be cited in articles on the platform. Note you must be logged in.",
                side: "left",
                align: "start",
            },
        },
        {
            search: { s: RANGE, r: "1" },
            element: `[data-tour="compare"]`,
            popover: {
                title: "Compare translations",
                description:
                    "Open a second panel beside this one to read another translation in parallel, or a completely different book—you build the system!",
                side: "left",
                align: "start",
            },
        },
        {
            search: { s: RANGE, r: "1" },
            element: `[data-tour="view-mode"]`,
            popover: {
                title: "Display options",
                description:
                    "Adjust how the text is shown and switch to a side-by-side layout when comparing translations, or translation to its source.",
                side: "bottom",
                align: "end",
            },
        },
        {
            search: { s: RANGE, r: "1" },
            popover: {
                title: "That's the tour",
                description:
                    "Explore freely. Your selections, notes, and quotations are always a click away. You can use saved quotations in our article editor to craft your own commentary and make it public. This tour can be replayed from the resources panel anytime. If you come across issues or bugs, you can use the feedback button the resouce panel. Thank you and have a nice day!",
            },
        },
    ];
}

/** Whether the library actually contains a book with `slug` — gates the tour so
 *  its entry points stay hidden if the demo text isn't ingested. */
export function libraryHasBook(
    library: LibraryResponse,
    slug: string,
): boolean {
    return library.groups.some((group) =>
        group.books.some((work) =>
            work.versions.some((version) => version.book_slug === slug),
        ),
    );
}

/** First TOC node flagged as having content (depth-first). The tour opens here,
 *  so it never depends on a hardcoded chapter slug. */
export function firstContentNodeSlug(
    toc: TocNodeResponse[],
): string | undefined {
    for (const node of toc) {
        if (node.has_content) return node.slug;
        const child = firstContentNodeSlug(node.children);
        if (child) return child;
    }
    return undefined;
}
