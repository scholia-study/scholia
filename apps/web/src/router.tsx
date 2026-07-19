import * as Sentry from "@sentry/tanstackstart-react";
import { QueryClient } from "@tanstack/react-query";
import { createRouter } from "@tanstack/react-router";
import { setupRouterSsrQueryIntegration } from "@tanstack/react-router-ssr-query";
import { DefaultCatchBoundary } from "./components/DefaultCatchBoundary";
import { NotFound } from "./components/NotFound";
import { routeTree } from "./routeTree.gen";

export function getRouter() {
    const queryClient = new QueryClient({
        defaultOptions: {
            queries: {
                staleTime: 5 * 60 * 1000,
                refetchOnWindowFocus: false,
            },
        },
    });

    const router = createRouter({
        routeTree,
        context: { queryClient },
        defaultPreload: "intent",
        defaultPreloadStaleTime: 0,
        defaultErrorComponent: DefaultCatchBoundary,
        defaultNotFoundComponent: () => <NotFound />,
    });
    setupRouterSsrQueryIntegration({
        router,
        queryClient,
    });

    if (!router.isServer) {
        Sentry.addIntegration(
            Sentry.tanstackRouterBrowserTracingIntegration(router),
        );
    }

    return router;
}

declare module "@tanstack/react-router" {
    interface Register {
        router: ReturnType<typeof getRouter>;
    }
}
