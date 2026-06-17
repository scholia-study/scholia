import { useCallback, useState } from "react";

export const LOC_STORAGE_KEYS = {
    readerFontSize: "scholia:reader-font-size",
    readerLineHeight: "scholia:reader-line-height",
    readerWidth: "scholia:reader-width",
    readerTourSeen: "scholia:reader-tour:seen:v1",
    /** Per Bible-shape group; legacy un-namespaced key (kept verbatim). */
    bibleTranslation: (groupId: string) => `bible-translation:${groupId}`,
} as const;

export function getLocalStorage(key: string): string | null {
    if (typeof window === "undefined") return null;
    try {
        return window.localStorage.getItem(key);
    } catch {
        return null;
    }
}

export function setLocalStorage(key: string, value: string): void {
    if (typeof window === "undefined") return;
    try {
        window.localStorage.setItem(key, value);
    } catch {
        // ignore (SSR / privacy mode / quota)
    }
}

export function removeLocalStorage(key: string): void {
    if (typeof window === "undefined") return;
    try {
        window.localStorage.removeItem(key);
    } catch {
        // ignore (SSR / privacy mode)
    }
}

export function useLocalStorageState(
    key: string,
    defaultValue: string,
): [string, (value: string) => void] {
    const [value, setValue] = useState<string>(
        () => getLocalStorage(key) ?? defaultValue,
    );
    const set = useCallback(
        (next: string) => {
            setLocalStorage(key, next);
            setValue(next);
        },
        [key],
    );
    return [value, set];
}
