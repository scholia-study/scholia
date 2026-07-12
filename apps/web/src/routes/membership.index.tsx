import { createFileRoute } from "@tanstack/react-router";
import { MembershipPage } from "../modules/billing";
import { SEO_COPY, seoHead } from "../modules/seo";

export const Route = createFileRoute("/membership/")({
    head: () =>
        seoHead({
            title: SEO_COPY.membership.title,
            description: SEO_COPY.membership.description,
            path: "/membership",
        }),
    component: MembershipPage,
});
