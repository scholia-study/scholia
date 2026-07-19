import { useEffect, useState } from "react";

/**
 * The hostname check runs in an effect so it's client-only: the server
 * and the first client render both produce nothing, so there's no
 * hydration mismatch (cf. the SSR-default pattern in routes/index.tsx).
 * On the dev host the notice simply appears just after hydration.
 */
export function DevServerNotice() {
    const [isDevHost, setIsDevHost] = useState(false);

    useEffect(() => {
        setIsDevHost(window.location.hostname === "dev.scholia.study");
    }, []);

    if (!isDevHost) return null;

    return (
        <div
            role="note"
            className="mb-8 rounded border-2 border-dashed border-amber-400 bg-amber-50/60 px-4 py-3"
        >
            <p className="text-sm font-semibold uppercase tracking-wider text-amber-800">
                Development server
            </p>
            <p className="mt-1 text-sm leading-relaxed text-amber-900/80">
                This is a preview deployment of Scholia. The data here is
                volatile: accounts, quotations, and notes you create here may
                change or disappear without notice. Only use for testing!
            </p>
            <button
                type="button"
                className="mt-2 rounded border border-amber-400 px-2 py-0.5 text-xs font-semibold uppercase tracking-wider text-amber-800 hover:bg-amber-100"
                onClick={() => {
                    throw new Error(
                        `error-reporting test (${new Date().toISOString()})`,
                    );
                }}
            >
                Test error reporting
            </button>
        </div>
    );
}
