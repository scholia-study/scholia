import { Link, useMatches } from "@tanstack/react-router";

const USER_ROUTES = [
    { to: "/user/quotations" as const, label: "My Quotations" },
    { to: "/user/notes" as const, label: "My Notes" },
    { to: "/user/articles" as const, label: "My Articles" },
    { to: "/user/sources" as const, label: "Sources" },
    { to: "/user/profile" as const, label: "Profile" },
];

export function UserSubnav() {
    const matches = useMatches();
    const isUserRoute = matches.some((m) => m.fullPath.startsWith("/user/"));

    if (!isUserRoute) return null;

    return (
        <nav className="flex flex-wrap shrink-0 min-h-10 items-center px-2 md:px-4 py-1 md:py-0 bg-white border-b border-stone-200 gap-0.5 md:gap-1">
            {USER_ROUTES.map((route) => (
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
