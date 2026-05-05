import { ThemeProvider } from "@mui/material/styles";
import type { QueryClient } from "@tanstack/react-query";
import {
    createRootRouteWithContext,
    HeadContent,
    Outlet,
    Scripts,
    useLocation,
} from "@tanstack/react-router";
import { Toaster } from "react-hot-toast";
import { Footer } from "../layout/Footer";
import { InfoSubnav } from "../layout/InfoSubnav";
import { Navbar } from "../layout/Navbar";
import { ScrollToTop } from "../layout/ScrollToTop";
import { UserSubnav } from "../layout/UserSubnav";
import { FeedbackModal, FeedbackProvider } from "../modules/feedback";
import appCss from "../styles.css?url";
import { theme } from "../theme";

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
        scripts: [{ src: "/config.js" }],
    }),
    shellComponent: RootDocument,
    component: RootComponent,
    notFoundComponent: NotFound,
});

/** Match the reader route shape: /books/<bookSlug>/<nodeSlug>. */
const READER_PATH = /^\/books\/[^/]+\/[^/]+/;

function RootComponent() {
    const { pathname } = useLocation();
    const isReader = READER_PATH.test(pathname);
    // Footer lives inside <main> and scrolls with the content.
    // Suppressed on the root library page (which has its own info links)
    // and on the reader route, which fills the viewport with internally
    // scrolling panels and has nowhere to put a non-overlapping footer.
    const showFooter = pathname !== "/" && !isReader;

    return (
        <ThemeProvider theme={theme}>
            <FeedbackProvider>
                <Navbar />
                <UserSubnav />
                <InfoSubnav />
                <main className="flex-1 overflow-y-auto">
                    <div
                        className={`${isReader ? "h-full" : "min-h-full"} flex flex-col`}
                    >
                        <div className="flex-1 min-h-0 flex flex-col">
                            <Outlet />
                        </div>
                        {showFooter && <Footer />}
                    </div>
                </main>
                <ScrollToTop />
                <FeedbackModal />
                <Toaster position="bottom-right" />
            </FeedbackProvider>
        </ThemeProvider>
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
            <body className="antialiased h-screen overflow-hidden flex flex-col bg-stone-50 text-stone-900">
                {children}
                <Scripts />
            </body>
        </html>
    );
}
