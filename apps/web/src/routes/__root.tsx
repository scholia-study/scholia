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
import { getMeQueryKey, getMeQueryOptions } from "../api/auth/auth";
import { AuthProvider } from "../hooks/useAuth";
import { Footer } from "../layout/Footer";
import { InfoSubnav } from "../layout/InfoSubnav";
import { Navbar } from "../layout/Navbar";
import { ScrollToTop } from "../layout/ScrollToTop";
import { UserSubnav } from "../layout/UserSubnav";
import { FeedbackModal, FeedbackProvider } from "../modules/feedback";
import {
    READER_DISPLAY_CSS,
    READER_DISPLAY_INIT_SCRIPT,
    ReaderPreferencesProvider,
} from "../modules/reader";
import appCss from "../styles.css?url";
import { theme } from "../theme";

interface RouterContext {
    queryClient: QueryClient;
}

export const Route = createRootRouteWithContext<RouterContext>()({
    loader: ({ context: { queryClient } }) => {
        // Root loaders re-run on every navigation (defaultStaleTime 0). A
        // logged-out /api/auth/me is a 401 → React Query error state, which is
        // always "stale", so an unconditional prefetch re-fires it on each
        // navigation (e.g. every tour step). Skip when a logged-out result is
        // already cached; the AuthProvider observer and login/logout
        // invalidation keep auth current after the first fetch.
        const cached = queryClient.getQueryState(getMeQueryKey());
        if (cached?.status !== "error") {
            return queryClient.prefetchQuery(
                getMeQueryOptions({ query: { retry: false } }), // Speed up auth
            );
        }
    },
    head: () => {
        // Runtime profile injection. The Node SSR reads APP_PROFILE from
        // its container env at render time and writes it inline; the
        // browser executes the script before the bundle (it's in <head>
        // and synchronous) so `apps/web/src/config.ts` sees window.__ENV__
        // when it evaluates.
        //
        // If head() runs on the client too, preserve whatever the server
        // already injected — re-reading process.env in the browser would
        // give undefined and clobber "prod" back to "local".
        const profile =
            typeof window !== "undefined"
                ? (window.__ENV__?.APP_PROFILE ?? "local")
                : (process.env.APP_PROFILE ?? "local");
        return {
            meta: [
                { charSet: "utf-8" },
                {
                    name: "viewport",
                    content: "width=device-width, initial-scale=1",
                },
                { title: "Scholia" },
            ],
            links: [
                { rel: "stylesheet", href: appCss },
                { rel: "icon", href: "/favicon.ico" },
                {
                    rel: "manifest",
                    href: "/manifest.json",
                    crossOrigin: "use-credentials",
                },
            ],
            scripts: [
                {
                    children: `window.__ENV__ = { APP_PROFILE: ${JSON.stringify(profile)} };`,
                },
                // Apply saved reader display prefs before first paint (no flash).
                { children: READER_DISPLAY_INIT_SCRIPT },
            ],
        };
    },
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
            <AuthProvider>
                <FeedbackProvider>
                    <ReaderPreferencesProvider>
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
                    </ReaderPreferencesProvider>
                </FeedbackProvider>
            </AuthProvider>
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
        <html lang="en" className="h-[100dvh] overflow-hidden">
            <head>
                <HeadContent />
                {/* Critical CSS: size/space the reading column before first paint
                    so a hard refresh doesn't reflow/scroll-shift (see ReaderPreferences). */}
                <style
                    // biome-ignore lint/security/noDangerouslySetInnerHtml: trusted, build-time constant
                    dangerouslySetInnerHTML={{ __html: READER_DISPLAY_CSS }}
                />
            </head>
            <body className="antialiased h-[100dvh] overflow-hidden flex flex-col bg-stone-50 text-stone-900">
                {children}
                <Scripts />
            </body>
        </html>
    );
}
