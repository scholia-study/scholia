import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { getGetProfileQueryOptions } from "../api/auth/auth";

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
