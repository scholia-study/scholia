import { defineConfig } from "orval";

export default defineConfig({
    prospero: {
        input: {
            target: "../openapi.json",
        },
        output: {
            mode: "tags-split",
            target: "src/api",
            schemas: "src/api/model",
            client: "react-query",
            override: {
                mutator: {
                    path: "./src/lib/fetcher.ts",
                    name: "customFetch",
                },
                query: {
                    useSuspenseQuery: true,
                },
                operations: {
                    get_node_page: {
                        query: {
                            useInfinite: true,
                            useSuspenseQuery: true,
                            useSuspenseInfiniteQuery: true,
                            useInfiniteQueryParam: "after",
                        },
                    },
                },
            },
        },
        hooks: {
            afterAllFilesWrite:
                "pnpx biome check --write --config-path=./biome.json ./src/api",
        },
    },
});
