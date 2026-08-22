import { createFileRoute } from "@tanstack/react-router";
import { CollegiumPage } from "../modules/collegium";

// Access is enforced by the API: private groups 404 for non-members.
export const Route = createFileRoute("/_auth/user/collegia/$slug")({
    component: GroupRoute,
});

function GroupRoute() {
    const { slug } = Route.useParams();
    return <CollegiumPage slug={slug} />;
}
