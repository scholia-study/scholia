import { createFileRoute, Link } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useGetLibrary } from "../api/books/books";
import type {
    LibraryGroup,
    LibraryStats,
    LibraryVersion,
    LibraryWork,
} from "../api/model";
import { InfoLinks } from "../components/InfoLinks";

export const Route = createFileRoute("/")({
    component: IndexPage,
});

function IndexPage() {
    const { data, isLoading } = useGetLibrary();
    const library = data?.data;

    return (
        <div className="min-h-full bg-white flex justify-center">
            <div className="flex w-full max-w-[88rem]">
                <div className="flex-1 min-w-0 px-6 md:pl-20 md:pr-8 py-10 md:pt-24 md:pb-14">
                    <div className="max-w-3xl mx-auto">
                        <h1 className="sr-only">Library</h1>
                        {!isLoading &&
                            library &&
                            library.groups.length === 0 && (
                                <p className="text-sm text-stone-400">
                                    No books in the library yet.
                                </p>
                            )}
                        {!isLoading && library && library.groups.length > 0 && (
                            <div className="space-y-10">
                                {library.groups.map((group) => (
                                    <GroupSection
                                        key={group.id}
                                        group={group}
                                    />
                                ))}
                            </div>
                        )}

                        <div className="md:hidden mt-10">
                            <AboutPanel stats={library?.stats} />
                        </div>
                    </div>
                </div>

                <aside className="hidden md:block md:w-96 md:shrink-0 bg-stone-50">
                    <div className="sticky top-0 h-[calc(100vh-3rem)] overflow-y-auto px-6 pt-24 pb-6">
                        <AboutPanel stats={library?.stats} />
                    </div>
                </aside>
            </div>
        </div>
    );
}

/**
 * Renders one group: an author with their works, a compilation with its
 * children, or a singleton authorless work that has no listed children
 * (the heading itself is the entry).
 */
function GroupSection({ group }: { group: LibraryGroup }) {
    const accent = accentColorFor(group.primary_label, group.id);
    const isSelf = group.primary_kind === "self";
    const isSingleton = isSelf && group.books.length === 0;
    // Bible-shape: one compilation work in many translations.
    // Pills become the primary navigation, translation collapses to a
    // single subtle chooser (PLAN_BIG_BOOKS.md Q1/Q2/Q5).
    const isBibleShape = group.book_pills.length > 0;

    if (isBibleShape) {
        return <BibleShapeGroup group={group} accent={accent} />;
    }

    return (
        <section>
            <h2 className="text-sm font-semibold uppercase tracking-wider text-stone-700 pb-2">
                {isSelf && group.primary_slug ? (
                    <Link
                        to="/books/$bookSlug"
                        params={{
                            bookSlug: group.primary_slug.replace(
                                /^\/books\//,
                                "",
                            ),
                        }}
                        className="hover:underline"
                    >
                        {group.primary_label}
                    </Link>
                ) : (
                    group.primary_label
                )}
            </h2>
            <div
                className="h-0.5 rounded-full mb-4"
                style={{ backgroundColor: accent }}
            />
            {!isSingleton && (
                <div className="space-y-5">
                    {group.books.map((work) => (
                        <WorkCard
                            key={work.work_id}
                            work={work}
                            hideTitle={
                                isSelf && work.title === group.primary_label
                            }
                        />
                    ))}
                </div>
            )}
        </section>
    );
}

/** Storage key holding the user's preferred translation per Bible-shape group. */
function bibleTranslationStorageKey(groupId: string): string {
    return `bible-translation:${groupId}`;
}

/**
 * Reads the persisted translation slug from localStorage on mount only.
 * Initial state matches the SSR default to avoid hydration mismatch;
 * the real value is applied after hydration via useEffect.
 */
function useBibleTranslation(
    groupId: string,
    versions: LibraryVersion[],
): [string, (slug: string) => void] {
    const fallback = useMemo(
        () =>
            // WEB is the v1 default (PLAN_BIG_BOOKS.md Q2). The publisher
            // field carries the short label; we ship "WEB" / "KJV" today.
            versions.find((v) => v.publisher === "WEB")?.book_slug ??
            versions[0]?.book_slug ??
            "",
        [versions],
    );
    const [slug, setSlug] = useState(fallback);

    useEffect(() => {
        if (typeof window === "undefined") return;
        try {
            const stored = window.localStorage.getItem(
                bibleTranslationStorageKey(groupId),
            );
            if (stored && versions.some((v) => v.book_slug === stored)) {
                setSlug(stored);
            }
        } catch {
            // localStorage may throw under privacy modes; just fall back.
        }
    }, [groupId, versions]);

    const setAndPersist = useCallback(
        (next: string) => {
            setSlug(next);
            try {
                window.localStorage.setItem(
                    bibleTranslationStorageKey(groupId),
                    next,
                );
            } catch {
                // ignore
            }
        },
        [groupId],
    );

    return [slug, setAndPersist];
}

function BibleShapeGroup({
    group,
    accent,
}: {
    group: LibraryGroup;
    accent: string;
}) {
    // Versions live on the (single) work for Bible-shape groups; the
    // group is guaranteed to have one work by the backend's pill-eligibility
    // rule, so we read versions off it. Empty fallback keeps types happy.
    const versions = group.books[0]?.versions ?? [];
    const [activeSlug, setActiveSlug] = useBibleTranslation(group.id, versions);

    return (
        <section>
            <div className="flex items-baseline justify-between gap-4 pb-2">
                <h2 className="text-sm font-semibold uppercase tracking-wider text-stone-700">
                    {group.primary_label}
                </h2>
                <div className="text-xs text-stone-400 flex flex-wrap gap-x-2">
                    {versions.map((v) => {
                        const isActive = v.book_slug === activeSlug;
                        const label = v.publisher ?? v.language.toUpperCase();
                        return (
                            <button
                                type="button"
                                key={v.book_slug}
                                onClick={() => setActiveSlug(v.book_slug)}
                                className={
                                    isActive
                                        ? "text-stone-700 underline underline-offset-2"
                                        : "hover:text-stone-700"
                                }
                            >
                                {label}
                            </button>
                        );
                    })}
                </div>
            </div>
            <div
                className="h-0.5 rounded-full mb-4"
                style={{ backgroundColor: accent }}
            />
            <div className="flex flex-wrap gap-1.5">
                {group.book_pills.map((p) => (
                    <Link
                        key={p.node_slug}
                        to="/books/$bookSlug"
                        params={{ bookSlug: activeSlug }}
                        hash={p.node_slug}
                        className="text-xs px-2 py-0.5 rounded border border-stone-300 text-stone-700 hover:border-stone-500 hover:text-stone-900 transition-colors"
                    >
                        {p.label}
                    </Link>
                ))}
            </div>
        </section>
    );
}

const ACCENT_PALETTE = [
    "#b45309", // amber-700
    "#047857", // emerald-700
    "#1d4ed8", // blue-700
    "#b91c1c", // red-700
    "#6d28d9", // violet-700
    "#0369a1", // sky-700
    "#a16207", // yellow-700
    "#be185d", // pink-700
];

/** Manual per-group accent overrides, keyed by exact display label. */
const ACCENT_OVERRIDES: Record<string, string> = {
    "Immanuel Kant": "#4169e1", // royal blue
    "The Bible": "#4b0082", // royal purple (indigo)
};

function accentColorFor(label: string, id: string): string {
    const override = ACCENT_OVERRIDES[label];
    if (override) return override;
    let hash = 0;
    for (let i = 0; i < id.length; i++) {
        hash = (hash * 31 + id.charCodeAt(i)) | 0;
    }
    return ACCENT_PALETTE[Math.abs(hash) % ACCENT_PALETTE.length];
}

function WorkCard({
    work,
    hideTitle = false,
}: {
    work: LibraryWork;
    /**
     * Suppress title and metadata. Used for SelfNamed groups where the
     * group heading already shows the work's title (e.g. "The Bible").
     * The version pills are then the only useful row.
     */
    hideTitle?: boolean;
}) {
    const versionLabels = useMemo(
        () => labelVersions(work.versions),
        [work.versions],
    );

    return (
        <article>
            {!hideTitle && (
                <>
                    <h3 className="text-base font-medium text-stone-900 font-serif">
                        {work.title}
                    </h3>
                    <p className="text-xs text-stone-400 mt-0.5">
                        {work.publication_year ?? "Undated"}
                        {work.co_authors.length > 0 && (
                            <> · with {work.co_authors.join(", ")}</>
                        )}
                        {work.editor_names && work.editor_names.length > 0 && (
                            <> · edited by {work.editor_names.join(", ")}</>
                        )}
                    </p>
                </>
            )}
            <div
                className={`flex flex-wrap gap-1.5 ${hideTitle ? "" : "mt-2"}`}
            >
                {work.versions.map((v, i) => (
                    <VersionPill
                        key={`${v.book_slug}::${v.node_slug ?? ""}`}
                        version={v}
                        label={versionLabels[i] ?? v.language.toUpperCase()}
                    />
                ))}
            </div>
        </article>
    );
}

function VersionPill({
    version,
    label,
}: {
    version: LibraryVersion;
    label: string;
}) {
    const className = `text-xs px-2 py-0.5 rounded border transition-colors ${
        version.is_original
            ? "border-stone-800 text-stone-900 hover:bg-stone-900 hover:text-white"
            : "border-stone-300 text-stone-600 hover:border-stone-500 hover:text-stone-900"
    }`;
    if (version.node_slug) {
        // Shape-3 nested anchor: deep-link into the host book at the
        // toc node slug.
        return (
            <a
                href={`/books/${version.book_slug}/${version.node_slug}`}
                className={className}
            >
                {label}
            </a>
        );
    }
    return (
        <Link
            to="/books/$bookSlug"
            params={{ bookSlug: version.book_slug }}
            className={className}
        >
            {label}
        </Link>
    );
}

function AboutPanel({ stats }: { stats: LibraryStats | undefined }) {
    const pClasses = "text-sm text-stone-600 leading-relaxed";
    return (
        <div className="md:border-0 border border-stone-200 md:p-0 p-5 md:bg-transparent bg-stone-100">
            <h2 className="text-base font-semibold text-stone-600 mb-2 uppercase">
                A living library for scholars
            </h2>
            <p className={pClasses}>
                Scholia is a reading and annotation library for philosophical
                and literary texts. Every work is structured down to the
                sentence, linked across translations, and open for quotation,
                notes, and citation in your own writing.
            </p>
            <br />
            <p className={pClasses}>
                Inspired by marginal notes that ancient and medieval scholars
                wrote alongside classical texts, Scholia aims to be a digital
                sanctuary for careful study, developing original insights, and
                building collaborative commentary.
            </p>
            {stats && (
                <p className="text-xs text-stone-400 mt-4 pt-4 border-t border-stone-200">
                    {formatStats(stats)}
                </p>
            )}
            <InfoLinks className="text-sm mt-6 md:mt-16 flex flex-wrap gap-x-4 gap-y-1 text-stone-500" />
        </div>
    );
}

function plural(n: number, singular: string, pluralForm: string) {
    return `${n} ${n === 1 ? singular : pluralForm}`;
}

function formatStats(stats: LibraryStats) {
    return [
        plural(stats.works, "work", "works"),
        plural(stats.authors, "author", "authors"),
        plural(stats.languages, "language", "languages"),
    ].join(" · ");
}

/**
 * Generate the pill label for each version in a work.
 * Same-language versions get disambiguated by translator → publisher → year.
 */
function labelVersions(versions: LibraryVersion[]): string[] {
    const counts = new Map<string, number>();
    for (const v of versions) {
        counts.set(v.language, (counts.get(v.language) ?? 0) + 1);
    }
    // When every version shares the language, the language code carries no
    // information; the publisher / translator alone is the useful pill.
    // (e.g. KJV vs WEB — both English, "EN · KJV" would be noise.)
    const allSameLanguage = counts.size === 1;
    return versions.map((v) => {
        const code = v.language.toUpperCase();
        const ambiguous = (counts.get(v.language) ?? 0) > 1;
        if (!ambiguous) return code;
        const prefix = allSameLanguage ? "" : `${code} · `;
        if (v.translator_names.length > 0) {
            return `${prefix}${v.translator_names.map(lastName).join(" & ")}`;
        }
        if (v.publisher) return `${prefix}${v.publisher}`;
        if (v.publication_year) return `${prefix}${v.publication_year}`;
        return code;
    });
}

function lastName(fullName: string): string {
    const parts = fullName.trim().split(/\s+/);
    return parts[parts.length - 1] ?? fullName;
}
