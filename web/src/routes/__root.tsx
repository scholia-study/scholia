import type { QueryClient } from "@tanstack/react-query";
import {
    createRootRouteWithContext,
    HeadContent,
    Scripts,
} from "@tanstack/react-router";
import appCss from "../styles.css?url";
import "@fontsource/roboto/300.css";
import "@fontsource/roboto/400.css";
import "@fontsource/roboto/500.css";
import "@fontsource/roboto/700.css";
import "@fontsource/libre-baskerville/400.css";
import "@fontsource/libre-baskerville/500.css";
import "@fontsource/libre-baskerville/600.css";
import "@fontsource/libre-baskerville/700.css";
import "@fontsource/libre-baskerville/400-italic.css";
import "@fontsource/libre-baskerville/500-italic.css";
import "@fontsource/libre-baskerville/600-italic.css";
import "@fontsource/libre-baskerville/700-italic.css";

interface RouterContext {
    queryClient: QueryClient;
}

export const Route = createRootRouteWithContext<RouterContext>()({
    head: () => ({
        meta: [
            { charSet: "utf-8" },
            {
                name: "viewport",
                content: "width=device-width, initial-scale=1",
            },
            { title: "Prospero" },
        ],
        links: [{ rel: "stylesheet", href: appCss }],
    }),
    shellComponent: RootDocument,
    notFoundComponent: NotFound,
});

function NotFound() {
    return (
        <div className="flex items-center justify-center h-screen">
            <div className="text-center">
                <h1 className="text-2xl font-bold text-stone-900 mb-2">
                    Page not found
                </h1>
                <p className="text-stone-500">
                    The page you're looking for doesn't exist.
                </p>
            </div>
        </div>
    );
}

function RootDocument({ children }: { children: React.ReactNode }) {
    return (
        <html lang="en">
            <head>
                <HeadContent />
            </head>
            <body className="min-h-screen bg-stone-50 text-stone-900">
                {children}
                <Scripts />
            </body>
        </html>
    );
}
