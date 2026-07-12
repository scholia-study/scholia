import { createFileRoute } from "@tanstack/react-router";
import { StaticMDRenderer } from "../components/StaticMDRenderer";
import { CONTACT_EMAIL } from "../constants";
import licenceMd from "../content/licence.md?raw";
import { SEO_COPY, seoHead } from "../modules/seo";

export const Route = createFileRoute("/licence")({
    head: () =>
        seoHead({
            title: SEO_COPY.licence.title,
            description: SEO_COPY.licence.description,
            path: "/licence",
        }),
    component: LicencePage,
});

function LicencePage() {
    const source = licenceMd.replace("{{CONTACT_EMAIL}}", CONTACT_EMAIL);
    return (
        <div className="min-h-full bg-white">
            <div className="max-w-3xl mx-auto px-6 md:px-8 py-16 md:py-24">
                <StaticMDRenderer source={source} />
            </div>
        </div>
    );
}
