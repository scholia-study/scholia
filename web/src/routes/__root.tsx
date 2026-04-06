import type { QueryClient } from "@tanstack/react-query";
import {
    Outlet,
    createRootRouteWithContext,
    HeadContent,
    Scripts,
    useMatches,
} from "@tanstack/react-router";
import { Toaster } from "react-hot-toast";
import appCss from "../styles.css?url";
import { Navbar } from "../components/Navbar";
import { UserSubnav } from "../components/UserSubnav";

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
    component: RootComponent,
    notFoundComponent: NotFound,
});

function RootComponent() {
    const matches = useMatches();
    const isUserRoute = matches.some((m) => m.fullPath.startsWith("/user/"));

    return (
        <>
            <Navbar />
            <UserSubnav />
            <main className={`min-h-screen pt-12 ${isUserRoute ? "md:pt-[5.5rem]" : ""}`}>
                <Outlet />
            </main>
            <Toaster position="bottom-right" />
        </>
    );
}

function NotFound() {
    return (
        <div className="flex items-center justify-center h-[calc(100vh-3rem)]">
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
            <body className="min-h-screen bg-stone-50  text-stone-900">
                {children}
                <Scripts />
            </body>
        </html>
    );
}
