import tailwindcss from "@tailwindcss/vite";
import { devtools } from "@tanstack/devtools-vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";

const config = defineConfig({
    plugins: [
        devtools(),
        tsconfigPaths({ projects: ["./tsconfig.json"] }),
        tailwindcss(),
        tanstackStart({
            prerender: {
                enabled: true,
                crawlLinks: true,
                autoStaticPathsDiscovery: true,
                onSuccess: ({ page }) => {
                    console.info(`🧼 Rendered ${page.path}`);
                },
            },
            spa: {
                enabled: true,
                prerender: {
                    enabled: true,
                    crawlLinks: true,
                    onSuccess: ({ page }) => {
                        console.info(`🖼️ Rendered ${page.path}!`);
                    },
                },
            },
        }),
        viteReact(),
    ],
    clearScreen: false,
});

export default config;
