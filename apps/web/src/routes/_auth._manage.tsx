import MenuOutlined from "@mui/icons-material/MenuOutlined";
import { Drawer, IconButton } from "@mui/material";
import {
    createFileRoute,
    Link,
    notFound,
    Outlet,
} from "@tanstack/react-router";
import { useState } from "react";
import { getMeQueryOptions } from "../api/auth/auth";
import { useAuth } from "../hooks/useAuth";

/**
 * The Bridge: one umbrella for every elevated-permission tool.
 * Any elevated permission opens the shell (obscurity-404 otherwise);
 * each section still enforces its own permission, and the API returns
 * 403 on its own account regardless.
 */
export const BRIDGE_SECTIONS = [
    {
        to: "/manage/feedback",
        label: "Feedback",
        description: "Reader-submitted feedback queue",
        permission: "admin_panel",
    },
    {
        to: "/manage/users",
        label: "Users",
        description: "Accounts and role grants",
        permission: "admin_panel",
    },
    {
        to: "/manage/topics",
        label: "Topics",
        description: "Article topic vocabulary",
        permission: "admin_panel",
    },
    {
        to: "/manage/series",
        label: "Series",
        description: "Curated, ordered article collections",
        permission: "series_manage",
    },
    {
        to: "/manage/resource-submissions",
        label: "Resource submissions",
        description:
            "Proposed passage resources — verbatim, paraphrase, allusion",
        permission: "resources_manage",
    },
    {
        to: "/manage/article-reviews",
        label: "Article reviews",
        description: "Publication review queue",
        permission: "articles_review",
    },
] as const;

const BRIDGE_PERMISSIONS = [
    ...new Set(BRIDGE_SECTIONS.map((s) => s.permission)),
];

const DRAWER_WIDTH = 208;

export const Route = createFileRoute("/_auth/_manage")({
    beforeLoad: async ({ context }) => {
        const me = await context.queryClient.fetchQuery(getMeQueryOptions());
        const permissions = me?.data?.permissions ?? [];
        if (!BRIDGE_PERMISSIONS.some((p) => permissions.includes(p))) {
            throw notFound();
        }
    },
    component: ManageLayout,
});

function ManageLayout() {
    const { hasPermission } = useAuth();
    const [mobileOpen, setMobileOpen] = useState(false);

    const nav = (
        <nav className="py-4 sticky top-0">
            <Link
                to="/manage"
                onClick={() => setMobileOpen(false)}
                className="block px-4 mb-3 text-[0.65rem] uppercase tracking-wide text-stone-400 no-underline hover:text-stone-600"
            >
                The Bridge
            </Link>
            {BRIDGE_SECTIONS.filter((s) => hasPermission(s.permission)).map(
                (section) => (
                    <Link
                        key={section.to}
                        to={section.to}
                        onClick={() => setMobileOpen(false)}
                        className="block px-4 py-1.5 text-sm text-stone-500 no-underline transition-colors hover:text-stone-900 hover:bg-stone-100"
                        activeProps={{
                            className:
                                "block px-4 py-1.5 text-sm no-underline bg-white text-stone-900 font-medium border-r-2 border-stone-700",
                        }}
                    >
                        {section.label}
                    </Link>
                ),
            )}
        </nav>
    );

    return (
        <div className="flex flex-1 min-h-0">
            {/* Always-open docked drawer at md+ (Tailwind md = 768px —
                same breakpoint as the mobile header below, NOT MUI's 900px
                md; mixing the two leaves a navless dead zone in between).
                The paper is put back in
                flow (instead of MUI's fixed) and stretches with this flex
                row, so it spans the full page height — exactly the visible
                area on short pages (no spurious scrollbar), the whole
                scrolled length on long ones. The nav inside is sticky so
                the links stay in view while the page scrolls. */}
            <Drawer
                variant="permanent"
                open
                className="hidden md:block"
                sx={{
                    width: DRAWER_WIDTH,
                    flexShrink: 0,
                }}
                slotProps={{
                    paper: {
                        sx: {
                            position: "relative",
                            width: DRAWER_WIDTH,
                            height: "100%",
                            bgcolor: "#fafaf9",
                            borderRight: "1px solid #e7e5e4",
                        },
                    },
                }}
            >
                {nav}
            </Drawer>

            {/* Mobile: toggled temporary drawer */}
            <Drawer
                variant="temporary"
                open={mobileOpen}
                onClose={() => setMobileOpen(false)}
                className="md:hidden"
                slotProps={{
                    paper: { sx: { width: DRAWER_WIDTH, bgcolor: "#fafaf9" } },
                }}
            >
                {nav}
            </Drawer>

            <div className="flex-1 min-w-0">
                <div className="md:hidden sticky top-0 z-10 flex items-center gap-2 px-4 h-9 bg-stone-50 border-b border-stone-200">
                    <IconButton
                        size="small"
                        onClick={() => setMobileOpen(true)}
                        aria-label="Open The Bridge navigation"
                    >
                        <MenuOutlined sx={{ fontSize: 18 }} />
                    </IconButton>
                    <span className="text-[0.65rem] uppercase tracking-wide text-stone-400">
                        The Bridge
                    </span>
                </div>
                <Outlet />
            </div>
        </div>
    );
}
