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
            prerender: {
                enabled: true,
                crawlLinks: true,
                autoStaticPathsDiscovery: true,
                filter: (page) => {
                    const p = page.path;
                    if (p.startsWith("/user")) return false;
                    if (p.startsWith("/admin")) return false;
                    return true;
                },
                // onSuccess: ({ page }) => {
                //     console.info(`🧼 Rendered ${page.path}`);
                // },
            },
            spa: {
                enabled: true,
                prerender: {
                    enabled: true,
                    // onSuccess: ({ page }) => {
                    //     console.info(`🖼️ Rendered ${page.path}!`);
                    // },
                },
            },
        }),
        viteReact(),
    ],
    clearScreen: false,
});

export default config;
