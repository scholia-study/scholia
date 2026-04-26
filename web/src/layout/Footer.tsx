import { InfoLinks } from "../components/InfoLinks";

export function Footer() {
    return (
        <footer className="bg-stone-50 px-6 py-6">
            <div className="max-w-6xl mx-auto flex justify-center">
                <InfoLinks />
            </div>
        </footer>
    );
}
