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
    ],
    clearScreen: false,
});

export default config;
