import { Link, useLocation } from "@tanstack/react-router";
import { INFO_LINKS, infoLinkPath } from "../components/InfoLinks";

const INFO_PATHS = INFO_LINKS.map(infoLinkPath);

export function InfoSubnav() {
    const pathname = useLocation({
        select: (location) => location.pathname,
    });
    const isInfoRoute = INFO_PATHS.includes(pathname.replace(/\/$/, ""));

    if (!isInfoRoute) return null;

    return (
        <nav className="flex flex-wrap shrink-0 min-h-10 items-center px-2 md:px-4 py-1 md:py-0 bg-white border-b border-stone-200 gap-0.5 md:gap-1">
            {INFO_LINKS.map((route) =>
                "href" in route ? (
                    <a
                        key={route.label}
                        href={route.href}
                        target="_blank"
                        rel="noreferrer"
                        className="text-xs md:text-sm px-2 md:px-3 py-1 rounded transition-colors text-stone-500 hover:text-stone-900 hover:bg-stone-100 whitespace-nowrap"
                    >
                        {route.label}
                    </a>
                ) : "params" in route ? (
                    <Link
                        key={route.label}
                        to={route.to}
                        params={route.params}
                        className="text-xs md:text-sm px-2 md:px-3 py-1 rounded transition-colors text-stone-500 hover:text-stone-900 hover:bg-stone-100 whitespace-nowrap"
                        activeProps={{
                            className:
                                "text-xs md:text-sm px-2 md:px-3 py-1 rounded transition-colors text-stone-900 bg-stone-100 font-medium whitespace-nowrap",
                        }}
                    >
                        {route.label}
                    </Link>
                ) : (
                    <Link
                        key={route.label}
                        to={route.to}
                        className="text-xs md:text-sm px-2 md:px-3 py-1 rounded transition-colors text-stone-500 hover:text-stone-900 hover:bg-stone-100 whitespace-nowrap"
                        activeProps={{
                            className:
                                "text-xs md:text-sm px-2 md:px-3 py-1 rounded transition-colors text-stone-900 bg-stone-100 font-medium whitespace-nowrap",
                        }}
                    >
                        {route.label}
                    </Link>
                ),
            )}
        </nav>
    );
}
