import { useCallback, useMemo, useRef } from "react";
import toast from "react-hot-toast";

/** Distinguishes a regular click (which sets a new anchor) from a shift-click
 *  (which extends the selection from the existing anchor). Callers may want to
 *  apply different post-select logic per mode — e.g. only the anchor branch
 *  should guard against re-emitting the same key, since shift-click results
 *  are the user's explicit intent. */
export type SelectionMode = "anchor" | "range";

interface UseRangeSelectionOptions<T> {
    /** URL-friendly key for a single item (sentence_number or id). */
    keyOf: (item: T) => string;
    /** Sentence number used for range computation. Null/undefined disables range math for that item. */
    sentenceNumberOf: (item: T) => number | null | undefined;
    /** Called when a selection is produced. `mode` tells you which branch fired. */
    onSelect: (key: string, item: T, mode: SelectionMode) => void;
    /** Maximum range size (inclusive). Exceeding this fires a toast and aborts. Default 10. */
    maxRange?: number;
}

interface RangeSelector<T> {
    /** Process a click. Shift-click extends a range from the anchor; regular click sets a new anchor. */
    select: (item: T, shiftKey: boolean) => void;
    /** Forget the current anchor (e.g. when external state clears the selection). */
    clear: () => void;
}

/** Anchor-based range-selection state machine for sentence-like items.
 *
 *  Regular click sets a new anchor and emits a single-item key.
 *  Shift-click computes [start, end] from anchor and clicked sentence_number,
 *  emits either a single-item key (when start === end) or a range key like "12-21".
 *  Range size is capped at `maxRange`; exceeding fires a toast and aborts. */
export function useRangeSelection<T>(
    options: UseRangeSelectionOptions<T>,
): RangeSelector<T> {
    const { keyOf, sentenceNumberOf, onSelect, maxRange = 10 } = options;
    const anchorRef = useRef<T | null>(null);

    const select = useCallback(
        (item: T, shiftKey: boolean) => {
            const anchor = anchorRef.current;
            const anchorNum = anchor ? sentenceNumberOf(anchor) : null;
            const targetNum = sentenceNumberOf(item);

            if (shiftKey && anchorNum != null && targetNum != null) {
                const start = Math.min(anchorNum, targetNum);
                const end = Math.max(anchorNum, targetNum);
                if (end - start + 1 > maxRange) {
                    toast.error(
                        `Range selection is limited to ${maxRange} sentences`,
                    );
                    return;
                }
                const key = start === end ? keyOf(item) : `${start}-${end}`;
                onSelect(key, item, "range");
            } else {
                anchorRef.current = item;
                onSelect(keyOf(item), item, "anchor");
            }
        },
        [keyOf, sentenceNumberOf, onSelect, maxRange],
    );

    const clear = useCallback(() => {
        anchorRef.current = null;
    }, []);

    return useMemo(() => ({ select, clear }), [select, clear]);
}
