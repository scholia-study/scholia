import { createFileRoute } from "@tanstack/react-router";
import { StaticMDRenderer } from "../components/StaticMDRenderer";
import termsMd from "../content/terms.md?raw";

export const Route = createFileRoute("/terms")({
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
