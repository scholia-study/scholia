import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";

/**
 * Single source of truth for the site's "info" pages — surfaced in the
 * info subnav, the root About panel, and the footer.
 */
export const INFO_LINKS = [
    { to: "/about" as const, label: "About" },
    { to: "/contribute" as const, label: "Contribute" },
    { to: "/membership" as const, label: "Membership" },
    { to: "/licence" as const, label: "Licence" },
    { to: "/terms" as const, label: "Terms" },
    { to: "/privacy" as const, label: "Privacy" },
];

interface InfoLinksProps {
    className?: string;
    linkClassName?: string;
    /** Extra item rendered after the links, sharing the same flex row. */
    trailing?: ReactNode;
}

export function InfoLinks({
    className = "flex flex-wrap gap-x-4 gap-y-1 text-sm text-stone-500",
    linkClassName = "no-underline hover:underline",
    trailing,
}: InfoLinksProps) {
    return (
        <div className={className}>
            {INFO_LINKS.map((link) => (
                <Link key={link.to} to={link.to} className={linkClassName}>
                    {link.label}
                </Link>
            ))}
            {trailing}
        </div>
    );
}
