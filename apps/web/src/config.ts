/**
 * Frontend config — profile registry.
 *
 * Every deployment-environment-specific value lives here, indexed by
 * profile. The active profile is selected at runtime via
 * `window.__ENV__.APP_PROFILE`, which is set by `/config.js` (rendered
 * by nginx envsubst at pod startup in cluster deployments). For local
 * `pnpm dev`, no `__ENV__` is injected and the profile defaults to
 * `"local"`.
 *
 * One container image works for every environment — only the rendered
 * `/config.js` differs.
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
        STRIPE_PUBLISHABLE_KEY: _stripePubKeyTest,
    },
    "local-proxy": {
        // Same-origin API: the local proxy (apps/proxy) terminates :8000
        // and routes /api/* to Rust. Activated by /config.js served from
        // the proxy container with APP_PROFILE=local-proxy.
        PROFILE: "local-proxy",
        API_BASE_URL: "",
        STRIPE_PUBLISHABLE_KEY: _stripePubKeyTest,
    },
    dev: {
        PROFILE: "dev",
        API_BASE_URL: "",
        STRIPE_PUBLISHABLE_KEY: _stripePubKeyTest,
    },
    prod: {
        PROFILE: "prod",
        API_BASE_URL: "",
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
