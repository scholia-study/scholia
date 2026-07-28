import {
    createFileRoute,
    Link,
    notFound,
    Outlet,
} from "@tanstack/react-router";
import { getMeQueryOptions } from "../api/auth/auth";
import { useAuth } from "../hooks/useAuth";

// Passes for any editor tool permission; each page still checks (via its
// API calls) the specific permission it needs.
const EDITOR_PERMISSIONS = ["resources_manage", "articles_review"];

export const Route = createFileRoute("/_auth/_editor")({
    beforeLoad: async ({ context }) => {
        const me = await context.queryClient.fetchQuery(getMeQueryOptions());
        const permissions = me?.data?.permissions ?? [];
        if (!EDITOR_PERMISSIONS.some((p) => permissions.includes(p))) {
            throw notFound();
        }
    },
    component: EditorLayout,
});

const EDITOR_TOOLS = [
    {
        to: "/editor/resource-submissions",
        label: "Source submissions",
        permission: "resources_manage",
    },
    {
        to: "/editor/article-reviews",
        label: "Article reviews",
        permission: "articles_review",
    },
] as const;

function EditorLayout() {
    const { hasPermission } = useAuth();
    return (
        <>
            <div className="shrink-0 flex items-center gap-1 px-8 h-9 bg-stone-50 border-b border-stone-200">
                <span className="text-[0.65rem] uppercase tracking-wide text-stone-400 mr-3">
                    Editor
                </span>
                {EDITOR_TOOLS.filter((t) => hasPermission(t.permission)).map(
                    (tool) => (
                        <Link
                            key={tool.to}
                            to={tool.to}
                            className="text-xs px-3 py-1 rounded transition-colors text-stone-500 no-underline hover:text-stone-900"
                            activeProps={{
                                className:
                                    "text-xs px-3 py-1 rounded bg-white border border-stone-200 text-stone-900 font-medium",
                            }}
                        >
                            {tool.label}
                        </Link>
                    ),
                )}
            </div>
            <Outlet />
        </>
    );
}
