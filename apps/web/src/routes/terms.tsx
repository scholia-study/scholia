import { createFileRoute } from "@tanstack/react-router";
import { StaticMDRenderer } from "../components/StaticMDRenderer";
import termsMd from "../content/terms.md?raw";
import { SEO_COPY, seoHead } from "../modules/seo";

export const Route = createFileRoute("/terms")({
    head: () =>
        seoHead({
            title: SEO_COPY.terms.title,
            description: SEO_COPY.terms.description,
            path: "/terms",
        }),
    component: TermsPage,
});

function TermsPage() {
    return (
        <div className="min-h-full bg-white">
            <div className="max-w-3xl mx-auto px-6 md:px-8 py-16 md:py-24">
                <StaticMDRenderer source={termsMd} />
            </div>
        </div>
    );
}
