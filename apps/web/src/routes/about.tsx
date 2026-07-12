import { createFileRoute } from "@tanstack/react-router";
import { StaticMDRenderer } from "../components/StaticMDRenderer";
import aboutMd from "../content/about.md?raw";
import { SEO_COPY, seoHead } from "../modules/seo";

export const Route = createFileRoute("/about")({
    head: () =>
        seoHead({
            title: SEO_COPY.about.title,
            description: SEO_COPY.about.description,
            path: "/about",
        }),
    component: AboutPage,
});

function AboutPage() {
    return (
        <div className="min-h-full bg-white">
            <div className="max-w-3xl mx-auto px-6 md:px-8 py-16 md:py-24">
                <StaticMDRenderer source={aboutMd} />
            </div>
        </div>
    );
}
