import { createFileRoute } from "@tanstack/react-router";
import { CONTACT_EMAIL } from "../constants";
import licenceMd from "../content/licence.md?raw";
import { StaticMDRenderer } from "../modules/ui/StaticMDRenderer";

export const Route = createFileRoute("/licence")({
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
