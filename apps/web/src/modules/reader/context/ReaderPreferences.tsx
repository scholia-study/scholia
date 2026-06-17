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

/**
 * Reader display preferences (currently just text size).
 *
 * Mounted high in the tree (see `__root.tsx`) so the stored value is ready
 * before the reader renders. The effective size — an explicit override, else a
 * responsive default — is written to the `--reader-font-size` CSS variable on
 * <html> in a layout effect (before paint, so no flash); the reading column
 * consumes it via `font-size: var(--reader-font-size)`. The constants below are
 * the single source of truth — no duplicate defaults in CSS or a head script.
 */

// useLayoutEffect warns when run during SSR; fall back to useEffect there.
const useIsomorphicLayoutEffect =
    typeof window !== "undefined" ? useLayoutEffect : useEffect;

const STORAGE_KEY = "scholia:reader-font-size";
/** Bounded, vetted size steps (px). */
const STEPS = [6, 8, 10, 12, 14, 16, 18, 20, 22, 24] as const;
const MIN = STEPS[0];
const MAX = STEPS[STEPS.length - 1];
/** Responsive defaults, used until the reader stores an explicit choice. */
const DEFAULT_MOBILE = 12;
const DEFAULT_DESKTOP = 18;
const MD_QUERY = "(min-width: 768px)";

/**
 * Responsive default as critical CSS, injected into <head> (see `__root.tsx`)
 * so the reading column is correctly sized on the very first paint — no
 * post-hydration resize, no scroll-shift. Generated from the constants above,
 * which stay the single source of truth; an explicit override is layered on top
 * inline by the provider.
 */
export const READER_FONT_SIZE_CSS = `:root{--reader-font-size:${DEFAULT_MOBILE}px}@media ${MD_QUERY}{:root{--reader-font-size:${DEFAULT_DESKTOP}px}}`;

/**
 * One job: if the reader has a saved size, apply it before first paint so a
 * hard refresh doesn't flash from the CSS default to the override. Runs in
 * <head> (see `__root.tsx`); `parseInt` sanitises the stored value, so it can
 * only ever produce a numeric pixel size. With no saved value it does nothing
 * and the critical CSS default stands.
 */
export const READER_FONT_SIZE_INIT_SCRIPT = `(function () {
  try {
    var n = parseInt(localStorage.getItem(${JSON.stringify(STORAGE_KEY)}), 10);
    if (n > 0) {
      document.documentElement.style.setProperty("--reader-font-size", n + "px");
    }
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

function readStoredOverride(): number | null {
    if (typeof window === "undefined") return null;
    try {
        const raw = window.localStorage.getItem(STORAGE_KEY);
        if (raw == null) return null;
        const n = Number.parseInt(raw, 10);
        return Number.isFinite(n) ? clampToRange(n) : null;
    } catch {
        // localStorage can throw under privacy modes; fall back to default.
        return null;
    }
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
}

const Ctx = createContext<ReaderPreferences | null>(null);

export function ReaderPreferencesProvider({
    children,
}: {
    children: ReactNode;
}) {
    // Client-only lazy init. Nothing override-dependent is in the SSR markup
    // (the scroller renders a constant `var(--reader-font-size)`), so reading
    // localStorage here causes no hydration mismatch.
    const [override, setOverride] = useState<number | null>(readStoredOverride);
    const [isDesktop, setIsDesktop] = useState(() =>
        typeof window === "undefined"
            ? true
            : window.matchMedia(MD_QUERY).matches,
    );

    const responsiveDefault = isDesktop ? DEFAULT_DESKTOP : DEFAULT_MOBILE;
    const fontSizePx = override ?? responsiveDefault;

    // Layer an explicit override over the responsive default (the critical CSS
    // in <head>). Clearing it falls back to that default. Runs before paint so
    // a stepped change applies without a flash; the no-override default needs
    // no JS at all, which is what keeps a hard refresh shift-free.
    useIsomorphicLayoutEffect(() => {
        const root = document.documentElement;
        if (override == null) root.style.removeProperty("--reader-font-size");
        else root.style.setProperty("--reader-font-size", `${override}px`);
    }, [override]);

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
                try {
                    window.localStorage.setItem(STORAGE_KEY, String(next));
                } catch {
                    // ignore persistence failures
                }
                return next;
            });
        },
        [responsiveDefault],
    );

    const increaseFontSize = useCallback(() => step(1), [step]);
    const decreaseFontSize = useCallback(() => step(-1), [step]);
    const resetFontSize = useCallback(() => {
        try {
            window.localStorage.removeItem(STORAGE_KEY);
        } catch {
            // ignore persistence failures
        }
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
        }),
        [
            fontSizePx,
            override,
            increaseFontSize,
            decreaseFontSize,
            resetFontSize,
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
