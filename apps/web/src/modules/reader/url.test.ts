import { describe, expect, it } from "vitest";
import { createPanel, type Panel, type ReaderState } from "./state";
import { decode, encode, type ReaderSearch, validateSearch } from "./url";

function panel(
    bookSlug: string,
    nodeSlug?: string,
    overrides: Partial<Panel> = {},
): Panel {
    return { ...createPanel(bookSlug, nodeSlug), ...overrides };
}

describe("validateSearch", () => {
    it("keeps only known string keys", () => {
        const out = validateSearch({
            s: "abc",
            r: "1",
            unknown: "ignored",
            p2: "book/node",
            extra: 42,
        });
        expect(out).toEqual({ s: "abc", r: "1", p2: "book/node" });
    });

    it("drops empty strings", () => {
        const out = validateSearch({ s: "", p2: "" });
        expect(out).toEqual({});
    });

    it("coerces JSON-parsed numeric values back to strings", () => {
        // The router's default search parser JSON-parses each value, so
        // "?s=12&r=1" arrives as numbers on a fresh page load.
        const out = validateSearch({ s: 12, r: 1 });
        expect(out).toEqual({ s: "12", r: "1" });
    });
});

describe("decode with raw (unvalidated) search values", () => {
    it("coerces numeric values seen during the hydration render", () => {
        const state = decode({
            bookSlug: "kant1",
            nodeSlug: "preface",
            search: { s: 12, r: 1 } as unknown as ReaderSearch,
        });
        expect(state.panels[0].selectedSentenceId).toBe("12");
        expect(state.panels[0].resourcesOpen).toBe(true);
    });
});

describe("decode → encode round-trip", () => {
    const cases: Array<{ name: string; state: ReaderState }> = [
        {
            name: "single panel, defaults",
            state: { panels: [panel("kant1", "preface")] },
        },
        {
            name: "single panel, all primary fields set",
            state: {
                panels: [
                    panel("kant1", "preface", {
                        selectedSentenceId: "12-21",
                        resourcesOpen: true,
                        showOriginal: true,
                        resourceView: "verbatim",
                        viewMode: "st",
                        viewLayout: "bpl",
                        companionSlug: "kant1-en",
                        footnoteSentenceId: "fn-3",
                    }),
                ],
            },
        },
        {
            name: "two panels with selections",
            state: {
                panels: [
                    panel("kant1", "preface", { selectedSentenceId: "5" }),
                    panel("hegel-wdl", "intro", {
                        selectedSentenceId: "12-21",
                        resourcesOpen: true,
                    }),
                ],
            },
        },
        {
            name: "four panels, mixed feature use",
            state: {
                panels: [
                    panel("a", "n1", { showOriginal: true }),
                    panel("b", "n2", {
                        resourcesOpen: true,
                        resourceView: "notes",
                    }),
                    panel("c", "n3", {
                        viewMode: "st",
                        viewLayout: "ss",
                        companionSlug: "c-tr",
                    }),
                    panel("d", "n4", { footnoteSentenceId: "fn-1" }),
                ],
            },
        },
        {
            name: "primary panel without nodeSlug",
            state: { panels: [panel("kant1")] },
        },
        {
            name: "secondary panel without nodeSlug",
            state: { panels: [panel("kant1", "preface"), panel("hegel-wdl")] },
        },
    ];

    for (const { name, state } of cases) {
        it(name, () => {
            const url = encode(state);
            const decoded = decode(url);
            expect(decoded).toEqual(state);
        });
    }
});

describe("encode → decode normalization", () => {
    it("encode(decode(url)) is idempotent", () => {
        const url = {
            bookSlug: "kant1",
            nodeSlug: "preface",
            search: validateSearch({
                p2: "hegel-wdl/intro",
                s2: "5",
                r2: "1",
                vm: "st",
                vl: "bpl",
                vt: "kant1-en",
            }),
        };
        const once = encode(decode(url));
        const twice = encode(decode(once));
        expect(twice).toEqual(once);
    });
});

describe("decode — strict on dangling state", () => {
    it("drops s2 when p2 is missing", () => {
        const state = decode({
            bookSlug: "kant1",
            nodeSlug: "preface",
            search: { s2: "5" } as ReaderSearch,
        });
        expect(state.panels.length).toBe(1);
        expect(state.panels[0].selectedSentenceId).toBeUndefined();
    });

    it("stops at the first missing p (no skipping)", () => {
        const state = decode({
            bookSlug: "a",
            nodeSlug: "n1",
            search: { p3: "c/n3" } as ReaderSearch,
        });
        expect(state.panels.length).toBe(1);
    });

    it("drops viewLayout when viewMode is not 'st'", () => {
        const state = decode({
            bookSlug: "kant1",
            nodeSlug: "preface",
            search: { vm: "s", vl: "bpl" } as ReaderSearch,
        });
        expect(state.panels[0].viewMode).toBe("s");
        expect(state.panels[0].viewLayout).toBeUndefined();
    });

    it("drops companionSlug when viewMode is not 'st'", () => {
        const state = decode({
            bookSlug: "kant1",
            nodeSlug: "preface",
            search: { vm: "s", vt: "kant1-en" } as ReaderSearch,
        });
        expect(state.panels[0].companionSlug).toBeUndefined();
    });

    it("drops viewLayout when viewMode is missing entirely", () => {
        const state = decode({
            bookSlug: "kant1",
            nodeSlug: "preface",
            search: { vl: "bpl", vt: "kant1-en" } as ReaderSearch,
        });
        expect(state.panels[0].viewLayout).toBeUndefined();
        expect(state.panels[0].companionSlug).toBeUndefined();
    });
});

describe("decode — coerces garbage to defaults", () => {
    it("invalid viewMode → undefined", () => {
        const state = decode({
            bookSlug: "kant1",
            nodeSlug: "preface",
            search: { vm: "xyzzy" } as ReaderSearch,
        });
        expect(state.panels[0].viewMode).toBeUndefined();
    });

    it("invalid viewLayout → undefined (even with vm='st')", () => {
        const state = decode({
            bookSlug: "kant1",
            nodeSlug: "preface",
            search: { vm: "st", vl: "wat" } as ReaderSearch,
        });
        expect(state.panels[0].viewMode).toBe("st");
        expect(state.panels[0].viewLayout).toBeUndefined();
    });

    it("ignores unknown query params (forward-compat)", () => {
        const search = validateSearch({
            p2: "b/n2",
            s: "5",
            p5: "future",
            unknownKey: "anything",
            x: "1",
        });
        const state = decode({
            bookSlug: "kant1",
            nodeSlug: "preface",
            search,
        });
        expect(state.panels.length).toBe(2);
        expect(state.panels[0].selectedSentenceId).toBe("5");
    });
});

describe("encode — wire format", () => {
    it("uses 's' (no suffix) for primary, 's2'/'s3'/'s4' for secondary", () => {
        const { search } = encode({
            panels: [
                panel("a", "n", { selectedSentenceId: "p1" }),
                panel("b", "m", { selectedSentenceId: "p2" }),
            ],
        });
        expect(search.s).toBe("p1");
        expect(search.s2).toBe("p2");
    });

    it("encodes boolean flags as '1'", () => {
        const { search } = encode({
            panels: [
                panel("a", "n", { resourcesOpen: true, showOriginal: true }),
            ],
        });
        expect(search.r).toBe("1");
        expect(search.og).toBe("1");
    });

    it("omits defaults from the wire format", () => {
        const { search } = encode({
            panels: [panel("a", "n")],
        });
        expect(search).toEqual({});
    });

    it("encodes secondary panel as 'bookSlug/nodeSlug'", () => {
        const { search } = encode({
            panels: [panel("a", "n1"), panel("b", "n2")],
        });
        expect(search.p2).toBe("b/n2");
    });

    it("encodes secondary panel without nodeSlug as just 'bookSlug'", () => {
        const { search } = encode({
            panels: [panel("a", "n1"), panel("b")],
        });
        expect(search.p2).toBe("b");
    });

    it("does not emit vl or vt when vm is not 'st'", () => {
        const { search } = encode({
            panels: [
                panel("a", "n", {
                    viewMode: "s",
                    viewLayout: "bpl",
                    companionSlug: "x",
                }),
            ],
        });
        expect(search.vm).toBe("s");
        expect(search.vl).toBeUndefined();
        expect(search.vt).toBeUndefined();
    });
});

describe("decode — pinned URL fixtures (wire format must not change)", () => {
    it("single panel, no search", () => {
        const state = decode({
            bookSlug: "kant1",
            nodeSlug: "preface",
            search: {},
        });
        expect(state).toEqual({ panels: [panel("kant1", "preface")] });
    });

    it("four-panel sharing URL", () => {
        const state = decode({
            bookSlug: "kant1",
            nodeSlug: "preface",
            search: validateSearch({
                p2: "hegel-wdl/intro",
                p3: "fichte/grundlage",
                p4: "schelling/system",
                s: "12-21",
                r: "1",
                og: "1",
                vm: "st",
                vl: "bpl",
                vt: "kant1-en",
                rv2: "verbatim",
                fs3: "fn-7",
            }),
        });
        expect(state.panels.length).toBe(4);
        expect(state.panels[0]).toEqual(
            panel("kant1", "preface", {
                selectedSentenceId: "12-21",
                resourcesOpen: true,
                showOriginal: true,
                viewMode: "st",
                viewLayout: "bpl",
                companionSlug: "kant1-en",
            }),
        );
        expect(state.panels[1].resourceView).toBe("verbatim");
        expect(state.panels[2].footnoteSentenceId).toBe("fn-7");
    });
});
