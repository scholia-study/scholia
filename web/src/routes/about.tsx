import { createFileRoute } from "@tanstack/react-router";
import aboutMd from "../content/about.md?raw";
import { StaticMDRenderer } from "../modules/ui";

export const Route = createFileRoute("/about")({
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
