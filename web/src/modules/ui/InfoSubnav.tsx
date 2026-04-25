import { Link, useMatches } from "@tanstack/react-router";
import { INFO_LINKS } from "./InfoLinks";

const INFO_PATHS = INFO_LINKS.map((r) => r.to);

export function InfoSubnav() {
    const matches = useMatches();
    const isInfoRoute = matches.some((m) =>
        INFO_PATHS.some((p) => m.fullPath === p),
    );

    if (!isInfoRoute) return null;

    return (
        <nav className="flex flex-wrap shrink-0 min-h-10 items-center px-2 md:px-4 py-1 md:py-0 bg-white border-b border-stone-200 gap-0.5 md:gap-1">
            {INFO_LINKS.map((route) => (
                <Link
                    key={route.to}
                    to={route.to}
                    className="text-xs md:text-sm px-2 md:px-3 py-1 rounded transition-colors text-stone-500 hover:text-stone-900 hover:bg-stone-100 whitespace-nowrap"
                    activeProps={{
                        className:
                            "text-xs md:text-sm px-2 md:px-3 py-1 rounded transition-colors text-stone-900 bg-stone-100 font-medium whitespace-nowrap",
                    }}
                >
                    {route.label}
                </Link>
            ))}
        </nav>
    );
}
