import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { BLOG_SERIES_SLUG } from "../modules/series";

/**
 * Single source of truth for the site's "info" pages — surfaced in the
 * info subnav, the root About panel, and the footer.
 */
export const INFO_LINKS = [
    { to: "/about" as const, label: "About" },
    {
        to: "/articles/series/$slug" as const,
        params: { slug: BLOG_SERIES_SLUG },
        label: "Blog",
    },
    { to: "/contribute" as const, label: "Contribute" },
    { to: "/membership" as const, label: "Membership" },
    { to: "/licence" as const, label: "Licence" },
    { to: "/terms" as const, label: "Terms" },
    { to: "/privacy" as const, label: "Privacy" },
];

type InfoLink = (typeof INFO_LINKS)[number];

/** Concrete pathname of a link — `$param` segments substituted. Used by
 * the subnav to match the current location (route-pattern matching would
 * light up every series page, not just the blog's). */
export function infoLinkPath(link: InfoLink): string {
    if (!("params" in link) || !link.params) return link.to;
    return Object.entries(link.params).reduce(
        (path, [key, value]) => path.replace(`$${key}`, value),
        link.to as string,
    );
}

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
            {INFO_LINKS.map((link) =>
                "params" in link ? (
                    <Link
                        key={link.label}
                        to={link.to}
                        params={link.params}
                        className={linkClassName}
                    >
                        {link.label}
                    </Link>
                ) : (
                    <Link
                        key={link.label}
                        to={link.to}
                        className={linkClassName}
                    >
                        {link.label}
                    </Link>
                ),
            )}
            {trailing}
        </div>
    );
}
