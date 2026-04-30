import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { getGetProfileQueryOptions } from "../api/auth/auth";

/**
 * Pathless layout that gates all child routes behind authentication.
 * Children opt in by living under the `_auth.*` filename prefix.
 *
 * The route guard runs once for the whole subtree — child routes do
 * not duplicate the profile fetch / redirect.
 */
export const Route = createFileRoute("/_auth")({
    beforeLoad: async ({ context }) => {
        const data = await context.queryClient.fetchQuery(
            getGetProfileQueryOptions(),
        );
        if (!data?.data) {
            throw redirect({ to: "/login" });
        }
    },
    component: AuthLayout,
});

function AuthLayout() {
    return <Outlet />;
}
