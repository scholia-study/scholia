import { createFileRoute, Link } from "@tanstack/react-router";
import { useMemo } from "react";
import { useGetLibrary } from "../api/books/books";
import type {
    LibraryAuthor,
    LibraryStats,
    LibraryVersion,
    LibraryWork,
} from "../api/model";

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
                    {isLoading && <LibrarySkeleton />}
                    {!isLoading && library && library.authors.length === 0 && (
                        <p className="text-sm text-stone-400">
                            No books in the library yet.
                        </p>
                    )}
                    {!isLoading && library && library.authors.length > 0 && (
                        <div className="space-y-10">
                            {library.authors.map((author) => (
                                <AuthorSection
                                    key={author.id}
                                    author={author}
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

function AuthorSection({ author }: { author: LibraryAuthor }) {
    const accent = accentColorFor(author.name, author.id);
    return (
        <section>
            <h2 className="text-sm font-semibold uppercase tracking-wider text-stone-700 pb-2">
                {author.name}
            </h2>
            <div
                className="h-0.5 rounded-full mb-4"
                style={{ backgroundColor: accent }}
            />
            <div className="space-y-5">
                {author.books.map((work) => (
                    <WorkCard key={work.work_id} work={work} />
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

/** Manual per-author accent overrides, keyed by exact display name. */
const ACCENT_OVERRIDES: Record<string, string> = {
    "Immanuel Kant": "#4169e1", // royal blue
};

function accentColorFor(name: string, id: string): string {
    const override = ACCENT_OVERRIDES[name];
    if (override) return override;
    let hash = 0;
    for (let i = 0; i < id.length; i++) {
        hash = (hash * 31 + id.charCodeAt(i)) | 0;
    }
    return ACCENT_PALETTE[Math.abs(hash) % ACCENT_PALETTE.length];
}

function WorkCard({ work }: { work: LibraryWork }) {
    const versionLabels = useMemo(
        () => labelVersions(work.versions),
        [work.versions],
    );

    return (
        <article>
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
            <div className="flex flex-wrap gap-1.5 mt-2">
                {work.versions.map((v, i) => (
                    <Link
                        key={v.book_slug}
                        to="/books/$bookSlug"
                        params={{ bookSlug: v.book_slug }}
                        className={`text-xs px-2 py-0.5 rounded border transition-colors ${
                            v.is_original
                                ? "border-stone-800 text-stone-900 hover:bg-stone-900 hover:text-white"
                                : "border-stone-300 text-stone-600 hover:border-stone-500 hover:text-stone-900"
                        }`}
                    >
                        {versionLabels[i]}
                    </Link>
                ))}
            </div>
        </article>
    );
}

function AboutPanel({ stats }: { stats: LibraryStats | undefined }) {
    return (
        <div className="md:border-0 border border-stone-200 md:p-0 p-5 md:bg-transparent bg-stone-100">
            <h2 className="text-base font-semibold text-stone-900 mb-2">
                A scholarly library
            </h2>
            <p className="text-sm text-stone-600 leading-relaxed">
                Scholia is a reading and annotation library for philosophical
                and literary texts. Every work is structured down to the
                sentence, linked across translations, and open for quotation,
                notes, and citation in your own writing.
            </p>
            {stats && (
                <p className="text-xs text-stone-400 mt-4 pt-4 border-t border-stone-200">
                    {formatStats(stats)}
                </p>
            )}
        </div>
    );
}

function LibrarySkeleton() {
    return (
        <div className="space-y-10 animate-pulse">
            {[0, 1, 2].map((i) => (
                <div key={i}>
                    <div className="h-3 w-32 bg-stone-200 rounded mb-4" />
                    <div className="space-y-5">
                        {[0, 1].map((j) => (
                            <div key={j}>
                                <div className="h-4 w-2/3 bg-stone-200 rounded mb-2" />
                                <div className="h-3 w-24 bg-stone-100 rounded mb-2" />
                                <div className="flex gap-1.5">
                                    <div className="h-5 w-10 bg-stone-100 rounded" />
                                    <div className="h-5 w-10 bg-stone-100 rounded" />
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
            ))}
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
    return versions.map((v) => {
        const code = v.language.toUpperCase();
        const ambiguous = (counts.get(v.language) ?? 0) > 1;
        if (!ambiguous) return code;
        if (v.translator_names.length > 0) {
            return `${code} · ${v.translator_names.map(lastName).join(" & ")}`;
        }
        if (v.publisher) return `${code} · ${v.publisher}`;
        if (v.publication_year) return `${code} · ${v.publication_year}`;
        return code;
    });
}

function lastName(fullName: string): string {
    const parts = fullName.trim().split(/\s+/);
    return parts[parts.length - 1] ?? fullName;
}
