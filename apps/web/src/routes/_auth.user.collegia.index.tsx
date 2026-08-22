import { createFileRoute } from "@tanstack/react-router";
import { CollegiaIndexPage } from "../modules/collegium";

export const Route = createFileRoute("/_auth/user/collegia/")({
    component: CollegiaIndexPage,
});
