import FeedbackOutlined from "@mui/icons-material/FeedbackOutlined";
import { Link } from "@tanstack/react-router";
import { useGetBookAbout } from "../../../api/books/books";
import type { SourcePersonResponse } from "../../../api/model";
import { useAuth } from "../../../hooks/useAuth";
import { useFeedback } from "../../feedback";

interface AboutThisTextViewProps {
    bookSlug: string;
    activeNodeSlug: string | undefined;
}

export function AboutThisTextView({
    bookSlug,
    activeNodeSlug,
}: AboutThisTextViewProps) {
    const { user } = useAuth();
    const { openModal } = useFeedback();
    const { data, isLoading, error } = useGetBookAbout(bookSlug, {
        node: activeNodeSlug,
    });

    if (isLoading) {
        return <div className="p-4 text-sm text-stone-400">Loading...</div>;
    }
    if (error || !data?.data) {
        return (
            <div className="p-4 text-sm text-stone-400">
                Couldn't load text info.
            </div>
        );
    }

    const { source, source_book, about_text } = data.data;
    const title = source.title_display ?? source.title;
    const byRole = groupByRole(source.persons);

    return (
        <div className="flex-1 overflow-y-auto px-3 py-3 space-y-4 text-sm">
            <div>
                <div className="text-stone-900 text-base leading-snug">
                    {title}
                </div>
                {byRole.author.length > 0 && (
                    <div className="text-stone-600 mt-0.5">
                        {byRole.author.map((p) => p.name).join(", ")}
                    </div>
                )}
            </div>

            {about_text && (
                <div>
                    <div className="text-stone-500 text-xs mb-1">
                        About this edition
                    </div>
                    <p className="text-stone-700 whitespace-pre-wrap leading-relaxed">
                        {about_text}
                    </p>
                </div>
            )}

            <dl className="space-y-2">
                {source.publication_year != null && (
                    <Field
                        label="Year"
                        value={String(source.publication_year)}
                    />
                )}
                {source.publisher && (
                    <Field label="Publisher" value={source.publisher} />
                )}
                {source.edition && (
                    <Field label="Edition" value={source.edition} />
                )}
                {source.volume && (
                    <Field label="Volume" value={source.volume} />
                )}
                {source.journal_name && (
                    <Field label="Journal" value={source.journal_name} />
                )}
                {source.isbn && source.isbn.length > 0 && (
                    <Field label="ISBN" value={source.isbn.join(", ")} />
                )}
                {source.doi && (
                    <Field
                        label="DOI"
                        value={
                            <a
                                href={`https://doi.org/${source.doi}`}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-stone-700 underline hover:text-stone-900"
                            >
                                {source.doi}
                            </a>
                        }
                    />
                )}
                {source.url && (
                    <Field
                        label="URL"
                        value={
                            <a
                                href={source.url}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-stone-700 underline hover:text-stone-900 break-all"
                            >
                                {source.url}
                            </a>
                        }
                    />
                )}
                {byRole.editor.length > 0 && (
                    <Field
                        label="Editor"
                        value={byRole.editor.map((p) => p.name).join(", ")}
                    />
                )}
                {byRole.translator.length > 0 && (
                    <Field
                        label="Translator"
                        value={byRole.translator.map((p) => p.name).join(", ")}
                    />
                )}
                {byRole.contributor.length > 0 && (
                    <Field
                        label="Contributor"
                        value={byRole.contributor.map((p) => p.name).join(", ")}
                    />
                )}
            </dl>

            {source_book && (
                <div className="pt-2 border-t border-stone-100">
                    <div className="text-stone-500 text-xs mb-1">
                        Translation of
                    </div>
                    <Link
                        to="/books/$bookSlug"
                        params={{ bookSlug: source_book.slug }}
                        className="text-stone-800 hover:underline"
                    >
                        {source_book.title}
                    </Link>
                    {source_book.author && (
                        <span className="text-stone-500">
                            {" — "}
                            {source_book.author}
                        </span>
                    )}
                </div>
            )}

            {source.parent && (
                <div className="pt-2 border-t border-stone-100">
                    <div className="text-stone-500 text-xs mb-1">In</div>
                    <div className="text-stone-800">{source.parent.title}</div>
                </div>
            )}

            <div className="pt-10 border-stone-100">
                <p className="text-stone-500 italic leading-relaxed mb-3">
                    Our texts get better through our community of readers.
                </p>
                <p className="text-stone-600 leading-relaxed mb-2">
                    Spotted a typo, a missing passage, or anything else off
                    about this text? Please tell us and we'll fix it.
                </p>
                <div className="text-center">
                    {user ? (
                        <button
                            type="button"
                            onClick={openModal}
                            className="inline-flex items-center gap-1.5 text-stone-700 hover:text-stone-900 hover:underline cursor-pointer"
                        >
                            <FeedbackOutlined fontSize="small" />
                            Report an issue
                        </button>
                    ) : (
                        <p className="text-stone-500">
                            <Link
                                to="/login"
                                className="text-stone-700 underline hover:text-stone-900"
                            >
                                Log in
                            </Link>{" "}
                            to send us a note.
                        </p>
                    )}
                </div>
            </div>
        </div>
    );
}

function Field({ label, value }: { label: string; value: React.ReactNode }) {
    return (
        <div className="flex gap-2">
            <dt className="text-stone-500 w-20 shrink-0">{label}</dt>
            <dd className="text-stone-800 min-w-0">{value}</dd>
        </div>
    );
}

function groupByRole(persons: SourcePersonResponse[]) {
    const groups: Record<
        "author" | "editor" | "translator" | "contributor",
        SourcePersonResponse[]
    > = { author: [], editor: [], translator: [], contributor: [] };
    for (const p of persons) {
        if (
            p.role === "author" ||
            p.role === "editor" ||
            p.role === "translator" ||
            p.role === "contributor"
        ) {
            groups[p.role].push(p);
        }
    }
    return groups;
}
