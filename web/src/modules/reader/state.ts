export const MAX_PANELS = 4;

export const VALID_VIEW_MODES = ["s", "t", "st"] as const;
export type ViewMode = (typeof VALID_VIEW_MODES)[number];

export const VALID_VIEW_LAYOUTS = [
    "sp",
    "ss",
    "bpl",
    "bpr",
    "bsl",
    "bsr",
] as const;
export type ViewLayout = (typeof VALID_VIEW_LAYOUTS)[number];

export interface Panel {
    bookSlug: string;
    nodeSlug: string | undefined;
    selectedSentenceId: string | undefined;
    resourcesOpen: boolean;
    showOriginal: boolean;
    resourceView: string | undefined;
    viewMode: ViewMode | undefined;
    viewLayout: ViewLayout | undefined;
    companionSlug: string | undefined;
    footnoteSentenceId: string | undefined;
}

export interface ReaderState {
    panels: Panel[];
}

export function createPanel(
    bookSlug: string,
    nodeSlug: string | undefined,
): Panel {
    return {
        bookSlug,
        nodeSlug,
        selectedSentenceId: undefined,
        resourcesOpen: false,
        showOriginal: false,
        resourceView: undefined,
        viewMode: undefined,
        viewLayout: undefined,
        companionSlug: undefined,
        footnoteSentenceId: undefined,
    };
}
