import { describe, expect, it } from "vitest";
import type { LibraryResponse, TocNodeResponse } from "#/api/model";
import {
    buildTourSteps,
    firstContentNodeSlug,
    libraryHasBook,
    TOUR_BOOK_SLUG,
} from "./tourConfig";

function node(
    slug: string,
    has_content: boolean,
    children: TocNodeResponse[] = [],
): TocNodeResponse {
    return {
        slug,
        has_content,
        children,
        id: slug,
        depth: 0,
        label: slug,
        sort_order: 0,
        source_ref: slug,
    };
}

describe("firstContentNodeSlug", () => {
    it("returns the first content node depth-first", () => {
        const toc = [
            node("genesis", false, [
                node("genesis-1", true),
                node("genesis-2", true),
            ]),
        ];
        expect(firstContentNodeSlug(toc)).toBe("genesis-1");
    });

    it("descends past content-less ancestors", () => {
        const toc = [node("part-1", false, [node("intro", false)])];
        expect(firstContentNodeSlug(toc)).toBeUndefined();
    });

    it("returns undefined for an empty toc", () => {
        expect(firstContentNodeSlug([])).toBeUndefined();
    });
});

describe("libraryHasBook", () => {
    const library = {
        groups: [
            {
                books: [
                    { versions: [{ book_slug: "kjv-bible" }] },
                    { versions: [{ book_slug: "web-bible" }] },
                ],
            },
        ],
    } as unknown as LibraryResponse;

    it("finds a present book", () => {
        expect(libraryHasBook(library, TOUR_BOOK_SLUG)).toBe(true);
    });

    it("rejects an absent book", () => {
        expect(libraryHasBook(library, "kant-critique")).toBe(false);
    });
});

describe("buildTourSteps", () => {
    const steps = buildTourSteps();

    it("opens and closes with anchorless (centered) steps", () => {
        expect(steps[0].element).toBeUndefined();
        expect(steps[steps.length - 1].element).toBeUndefined();
    });

    it("opens the resources panel for every panel-anchored step", () => {
        for (const step of steps) {
            if (step.element?.startsWith('[data-tour="') === true) {
                expect(step.search.r).toBe("1");
            }
        }
    });
});
