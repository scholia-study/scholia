/**
 * Every human-readable SEO string — titles, meta descriptions, shared
 * taglines — lives here so copy is reviewed and edited in one place.
 * Routes interpolate their dynamic values through the template
 * functions; no bare SEO prose belongs in route files.
 */

export const SITE_NAME = "Scholia";

/** "X | Scholia" — the standard title tail. */
export function siteTitle(prefix?: string): string {
    return prefix ? `${prefix} | ${SITE_NAME}` : SITE_NAME;
}

/** Shared tail for book and chapter meta descriptions. */
const STUDY_TAGLINE =
    "Study on Scholia with notes, translations and references.";

/** " by John Milton", or "" — the Bible has no single author. */
const byAuthor = (author: string) => (author ? ` by ${author}` : "");

export const SEO_COPY = {
    library: {
        title: "Scholia — read classic texts with notes and translations",
        description:
            "A hermeneutical workspace for deep study: the Bible, Kant, Shakespeare, " +
            "Milton and more, structured to the sentence and linked across " +
            "translations for careful study, quotation and citation.",
    },
    reader: {
        title: (nodeRef: string, bookTitle: string) =>
            siteTitle(`${nodeRef} — ${bookTitle}`),
        /** Fallback when the node has no content excerpt. */
        description: (nodeRef: string, bookTitle: string, author: string) =>
            `${nodeRef} of ${bookTitle}${byAuthor(author)}. ${STUDY_TAGLINE}`,
        /** Preferred: the node's actual opening text. */
        descriptionFromExcerpt: (nodeRef: string, excerpt: string) =>
            `${nodeRef} — ${excerpt}`,
    },
    bookToc: {
        title: (bookTitle: string) => siteTitle(bookTitle),
        description: (bookTitle: string, year: number | null, author: string) =>
            `${bookTitle}${year ? ` (${year})` : ""}${byAuthor(author)} — ` +
            `full table of contents. ${STUDY_TAGLINE}`,
    },
    articles: {
        title: siteTitle("Articles"),
        description:
            "Essays and commentary written on Scholia, grounded in " +
            "sentence-level quotations from primary sources.",
    },
    article: {
        title: (articleTitle: string) => siteTitle(articleTitle),
    },
    profile: {
        title: (displayName: string, handle: string) =>
            siteTitle(`${displayName} (@${handle})`),
        /** Fallback when the user has no bio. */
        description: (displayName: string) =>
            `Articles and notes by ${displayName} on Scholia.`,
    },
    about: {
        title: siteTitle("About"),
        description:
            "What Scholia is and why it exists: a hermeneutical " +
            "workspace for the deep study of literary, philosophical " +
            "and sacred texts.",
    },
    contribute: {
        title: siteTitle("Contribute"),
        description:
            "How to help Scholia grow: suggest texts, report issues " +
            "and contribute to the library.",
    },
    membership: {
        title: siteTitle("Membership"),
        description:
            "Support Scholia and unlock the full workspace: " +
            "quotations, notes and article writing across the library.",
    },
    licence: {
        title: siteTitle("Licence"),
        description:
            "Licensing of the texts hosted on Scholia and of " +
            "user-contributed content.",
    },
    privacy: {
        title: siteTitle("Privacy policy"),
        description: "How Scholia handles your data.",
    },
    terms: {
        title: siteTitle("Terms of service"),
        description: "The terms governing the use of Scholia.",
    },
    auth: {
        login: siteTitle("Log in"),
        register: siteTitle("Create account"),
        forgotPassword: siteTitle("Forgot password"),
        resetPassword: siteTitle("Reset password"),
        verifyEmail: siteTitle("Verify email"),
    },
} as const;
