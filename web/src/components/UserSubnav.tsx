import { Link, useMatches } from "@tanstack/react-router";

const USER_ROUTES = [
    { to: "/user/profile" as const, label: "Profile" },
    { to: "/user/quotations" as const, label: "Quotations" },
    { to: "/user/notes" as const, label: "Notes" },
    { to: "/user/articles" as const, label: "Articles" },
];

export function UserSubnav() {
    const matches = useMatches();
    const isUserRoute = matches.some((m) => m.fullPath.startsWith("/user/"));

    if (!isUserRoute) return null;

    return (
        <nav className="hidden md:flex fixed top-12 left-0 right-0 z-40 h-10 items-center px-4 bg-white/80 backdrop-blur border-b border-stone-200 gap-1">
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
