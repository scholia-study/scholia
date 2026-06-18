import {
    createContext,
    type ReactNode,
    useCallback,
    useContext,
    useEffect,
    useLayoutEffect,
    useMemo,
    useState,
} from "react";
import {
    getLocalStorage,
    LOC_STORAGE_KEYS,
    removeLocalStorage,
    setLocalStorage,
    useLocalStorageState,
} from "#/hooks/local-storage";

// useLayoutEffect warns when run during SSR; fall back to useEffect there.
const useIsomorphicLayoutEffect =
    typeof window !== "undefined" ? useLayoutEffect : useEffect;

const STEPS = [6, 8, 10, 12, 14, 16, 18, 20, 22, 24] as const;
const MIN = STEPS[0];
const MAX = STEPS[STEPS.length - 1];
const DEFAULT_MOBILE = 14;
const DEFAULT_DESKTOP = 18;
const MD_QUERY = "(min-width: 768px)";

export const LINE_SPACINGS = [
    { key: "compact", label: "Compact", value: "1.4" },
    { key: "normal", label: "Normal", value: "1.65" },
    { key: "relaxed", label: "Relaxed", value: "1.9" },
] as const;
const DEFAULT_LINE_HEIGHT = "1.65";

export const READING_WIDTHS = [
    { key: "narrow", label: "Narrow", value: "36rem" },
    { key: "medium", label: "Medium", value: "42rem" },
    { key: "wide", label: "Wide", value: "48rem" },
] as const;
const DEFAULT_READING_WIDTH = "42rem";

/**
 * Critical CSS injected into <head> (see `__root.tsx`) so the reading column is
 * correctly sized/spaced on the very first paint — no post-hydration resize, no
 * scroll-shift. Generated from the constants above; explicit overrides are
 * layered on top inline by the provider (and the init script, pre-paint).
 */
export const READER_DISPLAY_CSS = `:root{--reader-font-size:${DEFAULT_MOBILE}px;--reader-line-height:${DEFAULT_LINE_HEIGHT};--reader-width:${DEFAULT_READING_WIDTH}}@media ${MD_QUERY}{:root{--reader-font-size:${DEFAULT_DESKTOP}px}}`;

/**
 * Apply any saved overrides before first paint so a hard refresh doesn't flash
 * from the CSS default to the stored value. Runs in <head>; with no saved value
 * it does nothing and the critical CSS default stands. `parseInt` sanitises the
 * font size; the others only ever feed `var()` (an invalid value is ignored).
 */
export const READER_DISPLAY_INIT_SCRIPT = `(function () {
  try {
    var d = document.documentElement.style;
    var fs = parseInt(localStorage.getItem(${JSON.stringify(LOC_STORAGE_KEYS.readerFontSize)}), 10);
    if (fs > 0) d.setProperty("--reader-font-size", fs + "px");
    var lh = localStorage.getItem(${JSON.stringify(LOC_STORAGE_KEYS.readerLineHeight)});
    if (lh) d.setProperty("--reader-line-height", lh);
    var w = localStorage.getItem(${JSON.stringify(LOC_STORAGE_KEYS.readerWidth)});
    if (w) d.setProperty("--reader-width", w);
  } catch (e) {}
})();`;

function clampToRange(px: number): number {
    return Math.min(MAX, Math.max(MIN, px));
}

function nearestStepIndex(px: number): number {
    let best = 0;
    let bestDist = Number.POSITIVE_INFINITY;
    for (let i = 0; i < STEPS.length; i++) {
        const d = Math.abs(STEPS[i] - px);
        if (d < bestDist) {
            bestDist = d;
            best = i;
        }
    }
    return best;
}

function readStoredFontSize(): number | null {
    const raw = getLocalStorage(LOC_STORAGE_KEYS.readerFontSize);
    if (raw == null) return null;
    const n = Number.parseInt(raw, 10);
    return Number.isFinite(n) ? clampToRange(n) : null;
}

interface ReaderPreferences {
    /** Effective reading font size in px (for the menu readout + stepping). */
    fontSizePx: number;
    increaseFontSize: () => void;
    decreaseFontSize: () => void;
    /** Drop the explicit override, returning to the responsive default. */
    resetFontSize: () => void;
    /** True when an explicit size has been chosen (differs from default). */
    hasFontSizeOverride: boolean;
    canIncrease: boolean;
    canDecrease: boolean;
    /** Current line-height (unitless string, e.g. "1.65"); see LINE_SPACINGS. */
    lineHeight: string;
    setLineHeight: (value: string) => void;
    /** Current text-column max-width (e.g. "42rem"); see READING_WIDTHS. */
    readingWidth: string;
    setReadingWidth: (value: string) => void;
}

const Ctx = createContext<ReaderPreferences | null>(null);

export function ReaderPreferencesProvider({
    children,
}: {
    children: ReactNode;
}) {
    // Client-only lazy init. Nothing override-dependent is in the SSR markup
    // (the reading column renders constant `var(...)`s), so reading localStorage
    // here causes no hydration mismatch.
    const [override, setOverride] = useState<number | null>(readStoredFontSize);
    const [isDesktop, setIsDesktop] = useState(() =>
        typeof window === "undefined"
            ? true
            : window.matchMedia(MD_QUERY).matches,
    );
    const [lineHeight, setLineHeight] = useLocalStorageState(
        LOC_STORAGE_KEYS.readerLineHeight,
        DEFAULT_LINE_HEIGHT,
    );
    const [readingWidth, setReadingWidth] = useLocalStorageState(
        LOC_STORAGE_KEYS.readerWidth,
        DEFAULT_READING_WIDTH,
    );

    const responsiveDefault = isDesktop ? DEFAULT_DESKTOP : DEFAULT_MOBILE;
    const fontSizePx = override ?? responsiveDefault;

    // Layer an explicit font-size override over the responsive default (the
    // critical CSS in <head>). Clearing it falls back to that default. Runs
    // before paint so a stepped change applies without a flash; the no-override
    // default needs no JS at all, which keeps a hard refresh shift-free.
    useIsomorphicLayoutEffect(() => {
        const root = document.documentElement;
        if (override == null) root.style.removeProperty("--reader-font-size");
        else root.style.setProperty("--reader-font-size", `${override}px`);
    }, [override]);

    // Line spacing / width have no responsive default, so just mirror state to
    // the variable (before paint). Matches the critical CSS / init script on
    // first load, so no flash.
    useIsomorphicLayoutEffect(() => {
        document.documentElement.style.setProperty(
            "--reader-line-height",
            lineHeight,
        );
    }, [lineHeight]);
    useIsomorphicLayoutEffect(() => {
        document.documentElement.style.setProperty(
            "--reader-width",
            readingWidth,
        );
    }, [readingWidth]);

    // Track the md breakpoint so the responsive default updates live.
    useEffect(() => {
        const mq = window.matchMedia(MD_QUERY);
        const update = () => setIsDesktop(mq.matches);
        update();
        mq.addEventListener("change", update);
        return () => mq.removeEventListener("change", update);
    }, []);

    const step = useCallback(
        (dir: 1 | -1) => {
            setOverride((prev) => {
                const current = prev ?? responsiveDefault;
                const idx = nearestStepIndex(current);
                const next =
                    STEPS[Math.min(STEPS.length - 1, Math.max(0, idx + dir))];
                setLocalStorage(LOC_STORAGE_KEYS.readerFontSize, String(next));
                return next;
            });
        },
        [responsiveDefault],
    );

    const increaseFontSize = useCallback(() => step(1), [step]);
    const decreaseFontSize = useCallback(() => step(-1), [step]);
    const resetFontSize = useCallback(() => {
        removeLocalStorage(LOC_STORAGE_KEYS.readerFontSize);
        setOverride(null);
    }, []);

    const value = useMemo<ReaderPreferences>(
        () => ({
            fontSizePx,
            increaseFontSize,
            decreaseFontSize,
            resetFontSize,
            hasFontSizeOverride: override != null,
            canIncrease: fontSizePx < MAX,
            canDecrease: fontSizePx > MIN,
            lineHeight,
            setLineHeight,
            readingWidth,
            setReadingWidth,
        }),
        [
            fontSizePx,
            override,
            increaseFontSize,
            decreaseFontSize,
            resetFontSize,
            lineHeight,
            setLineHeight,
            readingWidth,
            setReadingWidth,
        ],
    );

    return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useReaderPreferences(): ReaderPreferences {
    const ctx = useContext(Ctx);
    if (!ctx) {
        throw new Error(
            "useReaderPreferences must be used within a ReaderPreferencesProvider",
        );
    }
    return ctx;
}
