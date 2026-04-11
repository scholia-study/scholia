import { Link, useMatches } from "@tanstack/react-router";

const USER_ROUTES = [
    { to: "/user/quotations" as const, label: "My Quotations" },
    { to: "/user/notes" as const, label: "My Notes" },
    { to: "/user/articles" as const, label: "My Articles" },
    { to: "/user/profile" as const, label: "Profile" },
];

export function UserSubnav() {
    const matches = useMatches();
    const isUserRoute = matches.some((m) => m.fullPath.startsWith("/user/"));

    if (!isUserRoute) return null;

    return (
        <nav className="hidden md:flex shrink-0 h-10 items-center px-4 bg-white border-b border-stone-200 gap-1">
            {USER_ROUTES.map((route) => (
                <Link
                    key={route.to}
                    to={route.to}
                    className="text-sm px-3 py-1 rounded transition-colors text-stone-500 hover:text-stone-900 hover:bg-stone-100"
                    activeProps={{
                        className:
                            "text-sm px-3 py-1 rounded transition-colors text-stone-900 bg-stone-100 font-medium",
                    }}
                >
                    {route.label}
                </Link>
            ))}
        </nav>
    );
}
