import { createFileRoute } from "@tanstack/react-router";
import { JoinCollegiumPage } from "../modules/collegium";

export const Route = createFileRoute("/_auth/user/collegia/join/$token")({
    component: JoinGroupRoute,
});

function JoinGroupRoute() {
    const { token } = Route.useParams();
    return <JoinCollegiumPage token={token} />;
}
