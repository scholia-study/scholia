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
            // index.mjs) plus client assets. nginx will sit in front of it
            // and cache the rendered HTML (see PLAN_3_TIER.md).
        }),
        viteReact(),
    ],
    clearScreen: false,
});

export default config;
