/**
 * Frontend config — profile registry.
 *
 * Every deployment-environment-specific value lives here, indexed by
 * profile. The active profile is selected at runtime via
 * `window.__ENV__.APP_PROFILE`, which is set by an inline <script> in
 * the SSR HTML head — see `src/routes/__root.tsx`. The Node SSR
 * container reads `APP_PROFILE` from its env at render time. For local
 * `pnpm dev`, no `APP_PROFILE` is set and the profile defaults to
 * `"local"`.
 *
 * One container image works for every environment — only the
 * `APP_PROFILE` env var on the web Deployment differs.
 *
 * ⚠️ DO NOT PUT SENSITIVE INFORMATION HERE. This file ships to the
 * browser. Stripe publishable keys are public by design; secret keys
 * stay server-side.
 */

type Profile = "local" | "local-proxy" | "dev" | "prod";

declare global {
    interface Window {
        __ENV__?: {
            APP_PROFILE: Profile;
        };
    }
}

const getActiveProfile = (): Profile => {
    if (typeof window === "undefined") {
        // Prerender / SSR context. The build hits the local API at
        // localhost:4000 to render book/chapter pages, so the "local"
        // profile is the right default here.
        return "local";
    }
    return window.__ENV__?.APP_PROFILE ?? "local";
};

interface EnvConfig {
    PROFILE: Profile;
    /** Base URL for API calls. Empty string = same-origin (cluster). */
    API_BASE_URL: string;
    /** Public origin for canonical URLs, Open Graph tags and JSON-LD. */
    SITE_ORIGIN: string;
    STRIPE_PUBLISHABLE_KEY: string;
    SENTRY_DSN: string;
    POSTHOG_TOKEN: string;
    POSTHOG_HOST: string;
    /**
     * Cloudflare Turnstile sitekey (public) for the registration bot
     * gate. Empty = widget hidden and no token sent; must match the
     * TURNSTILE_SECRET_KEY configured on the API for that environment.
     * Cloudflare's test pair for local runs: sitekey
     * 1x00000000000000000000AA / secret 1x0000000000000000000000000000000AA.
     */
    TURNSTILE_SITE_KEY: string;
}

const _sentryDsnDev =
    "https://pWnckLLN4SR5ErwyAiJ3Kb2T@s2610462.eu-central-1a.betterstackdata.com/2610462";
const _sentryDsnProd =
    "https://NvCH5CMCY7FBFhw2QTntdLrZ@s2637365.eu-central-1a.betterstackdata.com/2637365";

const _posthogToken = "phc_xxtCK6dsz7BBKjzFf4BGvbFpNE7MuVytwiiWfvTenQ3q";
const _posthogHost = "https://eu.i.posthog.com";

const _stripePubKeyTest =
    "pk_test_51TSz7zPDKNSxTB0E4aksjZoEVrCnhH5z6o78uTWhfwlCEqj2jmpBZd6B0miol0lM6xNQh1PVF68Sg3JMEtAuElkW00tReLfYms";
const _turnstileSiteKey = "0x4AAAAAAEDtKau9vgXX0_D9";

const _stripePubKeyLive =
    "pk_live_51TSz7zPDKNSxTB0ElTBxJBmkq0TiidSoEwnhtK7a9oyUsxQ1A72Lw345ieMwykOTr8CHd8BVmSHR0WKYPOOFp9Dk00xGvys1oh";

const envConfigs = {
    local: {
        PROFILE: "local",
        API_BASE_URL: "http://localhost:4000",
        SITE_ORIGIN: "http://localhost:3000",
        STRIPE_PUBLISHABLE_KEY: _stripePubKeyTest,
        SENTRY_DSN: "",
        POSTHOG_TOKEN: "",
        POSTHOG_HOST: _posthogHost,
        TURNSTILE_SITE_KEY: "",
    },
    "local-proxy": {
        // Same-origin API: the local proxy (apps/proxy) terminates :8000
        // and routes /api/* to Rust. Activated by running the web dev
        // server with APP_PROFILE=local-proxy (see `pnpm dev:all` in the
        // root package.json), which makes __root.tsx inject this profile
        // into the rendered HTML.
        PROFILE: "local-proxy",
        API_BASE_URL: "",
        SITE_ORIGIN: "http://localhost:8000",
        STRIPE_PUBLISHABLE_KEY: _stripePubKeyTest,
        SENTRY_DSN: "",
        POSTHOG_TOKEN: "",
        POSTHOG_HOST: _posthogHost,
        TURNSTILE_SITE_KEY: "",
    },
    dev: {
        PROFILE: "dev",
        API_BASE_URL: "",
        SITE_ORIGIN: "https://dev.scholia.study",
        STRIPE_PUBLISHABLE_KEY: _stripePubKeyTest,
        SENTRY_DSN: _sentryDsnDev,
        POSTHOG_TOKEN: _posthogToken,
        POSTHOG_HOST: _posthogHost,
        TURNSTILE_SITE_KEY: _turnstileSiteKey,
    },
    prod: {
        PROFILE: "prod",
        API_BASE_URL: "",
        SITE_ORIGIN: "https://scholia.study",
        STRIPE_PUBLISHABLE_KEY: _stripePubKeyLive,
        SENTRY_DSN: _sentryDsnProd,
        POSTHOG_TOKEN: _posthogToken,
        POSTHOG_HOST: _posthogHost,
        TURNSTILE_SITE_KEY: _turnstileSiteKey,
    },
} as const satisfies Record<Profile, EnvConfig>;

const activeProfile = getActiveProfile();
const activeEnvConfig = envConfigs[activeProfile];

if (activeProfile === "dev" && typeof window !== "undefined") {
    console.info("[dev profile]", activeEnvConfig);
}

const config = {
    ...activeEnvConfig,
} as const;

export default config;

/**
 * Public site origin for canonical URLs, Open Graph tags and JSON-LD.
 *
 * A function rather than a constant: the module-level `config` freezes
 * the profile at import time, which on the server is always "local"
 * (see getActiveProfile). SEO head() calls run at render time, where
 * `process.env.APP_PROFILE` reflects the actual deployment — mirror the
 * runtime resolution `__root.tsx` uses for `window.__ENV__` injection.
 */
export function getSiteOrigin(): string {
    const profile: Profile =
        typeof window !== "undefined"
            ? (window.__ENV__?.APP_PROFILE ?? "local")
            : ((process.env.APP_PROFILE as Profile | undefined) ?? "local");
    return envConfigs[profile].SITE_ORIGIN;
}
