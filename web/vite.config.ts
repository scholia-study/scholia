import tailwindcss from "@tailwindcss/vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { nitro } from "nitro/vite";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";

const config = defineConfig({
    plugins: [
        nitro({
            devProxy: {
                "/api": {
                    target: "http://localhost:4000",
                    changeOrigin: true,
                },
            },
            preset: "static", // This ensures it outputs files for Nginx, not a Node server
            prerender: {
                crawlLinks: true, // Automatically finds links and generates HTML for them
                routes: ["/"], // Start crawling from the homepage
            },
            rollupConfig: { external: [/^@sentry\//] },
        }),
        tsconfigPaths({ projects: ["./tsconfig.json"] }),
        tailwindcss(),
        // 2. You can also enable prerendering inside the tanstackStart plugin
        tanstackStart({
            prerender: {
                enabled: true,
            },
        }),
        viteReact(),
    ],
    server: {
        proxy: {
            "/api": "http://localhost:4000",
        },
    },
    clearScreen: false,
});

export default config;
