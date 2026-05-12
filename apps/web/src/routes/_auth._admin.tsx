import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { getMeQueryOptions } from "../api/auth/auth";

/**
 * Pathless layout for admin-only routes. Nested under `_auth`, so the
 * authentication check is inherited from the parent layout and only the
 * permission check runs here.
 *
 * Non-admins are redirected to `/` rather than shown a 403, so the
 * existence of admin routes isn't signalled to non-admin clients.
 */
export const Route = createFileRoute("/_auth/_admin")({
    beforeLoad: async ({ context }) => {
        const me = await context.queryClient.fetchQuery(getMeQueryOptions());
        if (!me?.data?.permissions?.includes("admin_panel")) {
            throw redirect({ to: "/" });
        }
    },
    component: AdminLayout,
});

function AdminLayout() {
    return <Outlet />;
}
