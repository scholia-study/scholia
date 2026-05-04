import { createFileRoute } from "@tanstack/react-router";
import { WelcomePage } from "../modules/billing";

export const Route = createFileRoute("/membership/welcome")({
    component: WelcomePage,
});
