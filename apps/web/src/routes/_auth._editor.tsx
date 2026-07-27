import { createFileRoute, notFound, Outlet } from "@tanstack/react-router";
import { getMeQueryOptions } from "../api/auth/auth";

export const Route = createFileRoute("/_auth/_editor")({
    beforeLoad: async ({ context }) => {
        const me = await context.queryClient.fetchQuery(getMeQueryOptions());
        if (!me?.data?.permissions?.includes("resources_manage")) {
            throw notFound();
        }
    },
    component: EditorLayout,
});

function EditorLayout() {
    return <Outlet />;
}
