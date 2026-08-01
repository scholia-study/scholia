import { useEffect, useRef } from "react";

/**
 * Cloudflare Turnstile widget (explicit rendering — the SPA-safe mode).
 * Renders nothing until the script loads; calls `onToken` with a token
 * on success and with `null` when the token expires or errors, so the
 * parent can gate its submit button. Tokens are single-use and valid
 * for 300s — remount (via `key`) after a failed submit to get a fresh
 * one.
 */

const SCRIPT_ID = "cf-turnstile-script";
const SCRIPT_SRC =
    "https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit";

interface TurnstileApi {
    render: (
        el: HTMLElement,
        opts: {
            sitekey: string;
            callback: (token: string) => void;
            "expired-callback": () => void;
            "error-callback": () => void;
        },
    ) => string;
    remove: (widgetId: string) => void;
}

declare global {
    interface Window {
        turnstile?: TurnstileApi;
    }
}

function loadScript(onLoad: () => void): () => void {
    if (window.turnstile) {
        onLoad();
        return () => {};
    }
    let script = document.getElementById(SCRIPT_ID) as HTMLScriptElement | null;
    if (!script) {
        script = document.createElement("script");
        script.id = SCRIPT_ID;
        script.src = SCRIPT_SRC;
        script.defer = true;
        document.head.appendChild(script);
    }
    script.addEventListener("load", onLoad);
    return () => script.removeEventListener("load", onLoad);
}

export function Turnstile({
    siteKey,
    onToken,
}: {
    siteKey: string;
    onToken: (token: string | null) => void;
}) {
    const containerRef = useRef<HTMLDivElement>(null);
    const onTokenRef = useRef(onToken);
    onTokenRef.current = onToken;

    useEffect(() => {
        let widgetId: string | undefined;
        let cancelled = false;

        const removeListener = loadScript(() => {
            if (cancelled || !containerRef.current || !window.turnstile) {
                return;
            }
            widgetId = window.turnstile.render(containerRef.current, {
                sitekey: siteKey,
                callback: (token) => onTokenRef.current(token),
                "expired-callback": () => onTokenRef.current(null),
                "error-callback": () => onTokenRef.current(null),
            });
        });

        return () => {
            cancelled = true;
            removeListener();
            if (widgetId) window.turnstile?.remove(widgetId);
        };
    }, [siteKey]);

    return <div ref={containerRef} />;
}
