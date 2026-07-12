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
}

const _stripePubKeyTest =
    "pk_test_51TSz7zPDKNSxTB0E4aksjZoEVrCnhH5z6o78uTWhfwlCEqj2jmpBZd6B0miol0lM6xNQh1PVF68Sg3JMEtAuElkW00tReLfYms";
// TODO: replace with the real live key before flipping prod to live mode.
const _stripePubKeyLive = _stripePubKeyTest;

const envConfigs = {
    local: {
        PROFILE: "local",
        API_BASE_URL: "http://localhost:4000",
        SITE_ORIGIN: "http://localhost:3000",
        STRIPE_PUBLISHABLE_KEY: _stripePubKeyTest,
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
    },
    dev: {
        PROFILE: "dev",
        API_BASE_URL: "",
        SITE_ORIGIN: "https://dev.scholia.study",
        STRIPE_PUBLISHABLE_KEY: _stripePubKeyTest,
    },
    prod: {
        PROFILE: "prod",
        API_BASE_URL: "",
        SITE_ORIGIN: "https://scholia.study",
        STRIPE_PUBLISHABLE_KEY: _stripePubKeyLive,
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
