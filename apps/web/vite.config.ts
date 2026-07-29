import { sentryTanstackStart } from "@sentry/tanstackstart-react/vite";
import tailwindcss from "@tailwindcss/vite";
import { devtools } from "@tanstack/devtools-vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const config = defineConfig({
    resolve: {
        tsconfigPaths: true,
    },
    plugins: [
        devtools(),
        tailwindcss(),
        tanstackStart({
            // SSR runtime mode: no prerender pass, no SPA shell. The build
            // produces a Nitro Node server (default output: .output/server/
            // index.mjs) plus client assets. The nginx proxy in apps/proxy/
            // sits in front of it and caches the rendered HTML.
        }),
        viteReact(),
        sentryTanstackStart({
            sentryUrl: "https://eu-central-1a-sourcemaps.betterstackdata.com",
            org: "572091",
            project: "2637365", // prod source.
            authToken: process.env.SENTRY_AUTH_TOKEN,
            tunnelRoute: true,
            release: { name: process.env.SENTRY_RELEASE }, // Build identity from CI (main-<sha7>).
            sourcemaps: {
                // Never ship map files in the image/public assets — they're
                // uploaded, then removed.
                filesToDeleteAfterUpload: "./dist/**/*.map",
            },
        }),
    ],
    clearScreen: false,
});

export default config;
