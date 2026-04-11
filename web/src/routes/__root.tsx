import type { QueryClient } from "@tanstack/react-query";
import {
    createRootRouteWithContext,
    HeadContent,
    Outlet,
    Scripts,
} from "@tanstack/react-router";
import { Toaster } from "react-hot-toast";
import { Navbar } from "../components/Navbar";
import { ScrollToTop } from "../components/ScrollToTop";
import { UserSubnav } from "../components/UserSubnav";
import appCss from "../styles.css?url";

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
            { title: "Scholia" },
        ],
        links: [{ rel: "stylesheet", href: appCss }],
    }),
    shellComponent: RootDocument,
    component: RootComponent,
    notFoundComponent: NotFound,
});

function RootComponent() {
    return (
        <>
            <Navbar />
            <UserSubnav />
            <main className="flex-1 overflow-y-auto">
                <Outlet />
            </main>
            <ScrollToTop />
            <Toaster position="bottom-right" />
        </>
    );
}

function NotFound() {
    return (
        <div className="flex items-center justify-center h-full">
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
            <body className="h-screen overflow-hidden flex flex-col bg-stone-50 text-stone-900">
                {children}
                <Scripts />
            </body>
        </html>
    );
}
