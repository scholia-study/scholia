import { createFileRoute } from "@tanstack/react-router";
import { StaticMDRenderer } from "../components/StaticMDRenderer";
import privacyMd from "../content/privacy.md?raw";
import { SEO_COPY, seoHead } from "../modules/seo";

export const Route = createFileRoute("/privacy")({
    head: () =>
        seoHead({
            title: SEO_COPY.privacy.title,
            description: SEO_COPY.privacy.description,
            path: "/privacy",
        }),
    component: PrivacyPage,
});

function PrivacyPage() {
    return (
        <div className="min-h-full bg-white">
            <div className="max-w-3xl mx-auto px-6 md:px-8 py-16 md:py-24">
                <StaticMDRenderer source={privacyMd} />
            </div>
        </div>
    );
}
