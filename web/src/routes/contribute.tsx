import { createFileRoute } from "@tanstack/react-router";
import { CONTACT_EMAIL } from "../constants";
import { StaticMDRenderer } from "../components/StaticMDRenderer";
import contributeMd from "../content/contribute.md?raw";

export const Route = createFileRoute("/contribute")({
    component: ContributePage,
});

function ContributePage() {
    const source = contributeMd.replace("{{CONTACT_EMAIL}}", CONTACT_EMAIL);
    return (
        <div className="min-h-full bg-white">
            <div className="max-w-3xl mx-auto px-6 md:px-8 py-16 md:py-24">
                <StaticMDRenderer source={source} />
            </div>
        </div>
    );
}
