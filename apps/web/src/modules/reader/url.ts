import {
    MAX_PANELS,
    type Panel,
    type ReaderState,
    VALID_VIEW_LAYOUTS,
    VALID_VIEW_MODES,
    type ViewLayout,
    type ViewMode,
} from "./state";

export type ReaderSearch = {
    p2?: string;
    p3?: string;
    p4?: string;
    s?: string;
    s2?: string;
    s3?: string;
    s4?: string;
    r?: string;
    r2?: string;
    r3?: string;
    r4?: string;
    og?: string;
    og2?: string;
    og3?: string;
    og4?: string;
    rv?: string;
    rv2?: string;
    rv3?: string;
    rv4?: string;
    vm?: string;
    vm2?: string;
    vm3?: string;
    vm4?: string;
    vl?: string;
    vl2?: string;
    vl3?: string;
    vl4?: string;
    vt?: string;
    vt2?: string;
    vt3?: string;
    vt4?: string;
    fs?: string;
    fs2?: string;
    fs3?: string;
    fs4?: string;
};

const SEARCH_KEYS = [
    "p2",
    "p3",
    "p4",
    "s",
    "s2",
    "s3",
    "s4",
    "r",
    "r2",
    "r3",
    "r4",
    "og",
    "og2",
    "og3",
    "og4",
    "rv",
    "rv2",
    "rv3",
    "rv4",
    "vm",
    "vm2",
    "vm3",
    "vm4",
    "vl",
    "vl2",
    "vl3",
    "vl4",
    "vt",
    "vt2",
    "vt3",
    "vt4",
    "fs",
    "fs2",
    "fs3",
    "fs4",
] as const satisfies ReadonlyArray<keyof ReaderSearch>;

/** Coerce a raw search-param record into the typed `ReaderSearch` shape used by
 *  TanStack Router's `validateSearch`. Drops unknown keys; keeps only string values. */
export function validateSearch(search: Record<string, unknown>): ReaderSearch {
    const out: ReaderSearch = {};
    for (const key of SEARCH_KEYS) {
        const v = search[key];
        if (typeof v === "string" && v.length > 0) out[key] = v;
    }
    return out;
}

interface ReaderURL {
    bookSlug: string;
    nodeSlug: string | undefined;
    search: ReaderSearch;
}

function suffix(idx: number): string {
    return idx === 0 ? "" : String(idx + 1);
}

function key<K extends keyof ReaderSearch>(prefix: string, idx: number): K {
    return `${prefix}${suffix(idx)}` as K;
}

function parsePanelParam(param: string): {
    bookSlug: string;
    nodeSlug: string | undefined;
} | null {
    if (!param) return null;
    const slashIdx = param.indexOf("/");
    if (slashIdx === -1) return { bookSlug: param, nodeSlug: undefined };
    const bookSlug = param.slice(0, slashIdx);
    if (!bookSlug) return null;
    return {
        bookSlug,
        nodeSlug: param.slice(slashIdx + 1) || undefined,
    };
}

function isViewMode(v: string | undefined): v is ViewMode {
    return v != null && (VALID_VIEW_MODES as readonly string[]).includes(v);
}

function isViewLayout(v: string | undefined): v is ViewLayout {
    return v != null && (VALID_VIEW_LAYOUTS as readonly string[]).includes(v);
}

function read(
    search: ReaderSearch,
    prefix: string,
    idx: number,
): string | undefined {
    return search[key<keyof ReaderSearch>(prefix, idx)];
}

/** Convert URL pieces (path params + typed search) into canonical `ReaderState`.
 *  Strict on dangling state: `s2` without `p2`, `vl` without `vm="st"`, garbage
 *  view-mode/layout values are silently dropped. Re-encoding the result yields
 *  a clean URL — `encode(decode(x))` is normalized. */
export function decode(url: ReaderURL): ReaderState {
    const { bookSlug, nodeSlug, search } = url;

    const slugs: Array<{ bookSlug: string; nodeSlug: string | undefined }> = [
        { bookSlug, nodeSlug },
    ];

    // Secondary panels: stop at the first missing/invalid `p` (no skipping)
    for (let i = 1; i < MAX_PANELS; i++) {
        const param = read(search, "p", i);
        if (!param) break;
        const parsed = parsePanelParam(param);
        if (!parsed) break;
        slugs.push(parsed);
    }

    const panels: Panel[] = slugs.map((s, i) => {
        const viewModeRaw = read(search, "vm", i);
        const viewMode = isViewMode(viewModeRaw) ? viewModeRaw : undefined;

        const viewLayoutRaw = read(search, "vl", i);
        const viewLayout =
            viewMode === "st" && isViewLayout(viewLayoutRaw)
                ? viewLayoutRaw
                : undefined;

        const companionSlug =
            viewMode === "st" ? read(search, "vt", i) : undefined;

        return {
            bookSlug: s.bookSlug,
            nodeSlug: s.nodeSlug,
            selectedSentenceId: read(search, "s", i),
            resourcesOpen: !!read(search, "r", i),
            showOriginal: !!read(search, "og", i),
            resourceView: read(search, "rv", i),
            viewMode,
            viewLayout,
            companionSlug,
            footnoteSentenceId: read(search, "fs", i),
        };
    });

    return { panels };
}

/** Convert canonical `ReaderState` into URL pieces. Defaults are omitted from
 *  the wire format. Does not emit dangling state (e.g. `vl` without `vm="st"`). */
export function encode(state: ReaderState): ReaderURL {
    const search: ReaderSearch = {};
    const primary = state.panels[0];

    for (let i = 0; i < state.panels.length; i++) {
        const p = state.panels[i];

        if (i > 0) {
            search[key<keyof ReaderSearch>("p", i)] = p.nodeSlug
                ? `${p.bookSlug}/${p.nodeSlug}`
                : p.bookSlug;
        }

        if (p.selectedSentenceId)
            search[key<keyof ReaderSearch>("s", i)] = p.selectedSentenceId;
        if (p.resourcesOpen) search[key<keyof ReaderSearch>("r", i)] = "1";
        if (p.showOriginal) search[key<keyof ReaderSearch>("og", i)] = "1";
        if (p.resourceView)
            search[key<keyof ReaderSearch>("rv", i)] = p.resourceView;
        if (p.viewMode) search[key<keyof ReaderSearch>("vm", i)] = p.viewMode;
        if (p.viewMode === "st" && p.viewLayout)
            search[key<keyof ReaderSearch>("vl", i)] = p.viewLayout;
        if (p.viewMode === "st" && p.companionSlug)
            search[key<keyof ReaderSearch>("vt", i)] = p.companionSlug;
        if (p.footnoteSentenceId)
            search[key<keyof ReaderSearch>("fs", i)] = p.footnoteSentenceId;
    }

    return {
        bookSlug: primary.bookSlug,
        nodeSlug: primary.nodeSlug,
        search,
    };
}
