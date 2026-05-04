import { createFileRoute } from "@tanstack/react-router";
import { MembershipPage } from "../modules/billing";

export const Route = createFileRoute("/membership/")({
    component: MembershipPage,
});
