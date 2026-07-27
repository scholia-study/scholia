import ExpandLessOutlined from "@mui/icons-material/ExpandLessOutlined";
import ExpandMoreOutlined from "@mui/icons-material/ExpandMoreOutlined";
import StarOutlined from "@mui/icons-material/StarOutlined";
import { useState } from "react";
import type { ResourceResponse } from "../../../api/model";

interface CommentaryEntryProps {
    resource: ResourceResponse;
    isEditor: boolean;
    /** Language of the book being read, for the cross-language badge. */
    bookLanguage?: string;
    onEdit: (resource: ResourceResponse) => void;
    onDelete: (resource: ResourceResponse) => void;
}

function scopeLabel(scope: string, isProjected: boolean): string | null {
    if (scope === "language") {
        return isProjected
            ? "about its language's translation layer"
            : "about this translation layer";
    }
    if (scope === "edition") {
        return isProjected
            ? "specific to its edition's text"
            : "specific to this edition's text";
    }
    return null;
}

export function CommentaryEntry({
    resource,
    isEditor,
    bookLanguage,
    onEdit,
    onDelete,
}: CommentaryEntryProps) {
    const [expanded, setExpanded] = useState(false);
    const source = resource.source;

    // Projected entries live on a sibling edition of this work. Language
    // difference is the prominent signal; same-language projection gets a
    // subtle edition note (ADR 0008).
    const crossLanguage =
        resource.is_projected &&
        !!resource.origin_language &&
        resource.origin_language !== bookLanguage;

    // Build citation line: "Author (Year), Title, pp. X-Y"
    const firstAuthor = source?.persons?.find((p) => p.role === "author");
    const authorName = firstAuthor?.name ?? source?.persons?.[0]?.name;
    const year = source?.publication_year;
    const citation = [authorName, year ? `(${year})` : null]
        .filter(Boolean)
        .join(" ");

    const pageRef = resource.source_page_start
        ? resource.source_page_end &&
          resource.source_page_end !== resource.source_page_start
            ? `pp. ${resource.source_page_start}\u2013${resource.source_page_end}`
            : `p. ${resource.source_page_start}`
        : (resource.source_location_freeform ?? null);

    return (
        <div className="border border-stone-200 rounded text-sm">
            {/* Collapsed header */}
            <button
                type="button"
                onClick={() => setExpanded(!expanded)}
                className="w-full text-left px-3 py-2 flex items-center gap-2 hover:bg-stone-50 transition-colors"
            >
                <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-1.5">
                        {resource.is_featured && (
                            <StarOutlined
                                sx={{ fontSize: 14, color: "rgb(202 138 4)" }}
                            />
                        )}
                        <span className="text-stone-800 truncate">
                            {citation || "Unknown source"}
                        </span>
                        {resource.review_status === "pending" && (
                            <span className="shrink-0 text-[10px] uppercase tracking-wide font-medium text-amber-700 bg-amber-50 rounded px-1 py-0.5">
                                Pending review
                            </span>
                        )}
                        {crossLanguage && (
                            <span
                                className="shrink-0 text-[10px] uppercase tracking-wide font-medium text-indigo-700 bg-indigo-50 rounded px-1 py-0.5"
                                title={`From the ${resource.origin_book_slug} edition`}
                            >
                                {resource.origin_language}
                            </span>
                        )}
                        {resource.is_projected && !crossLanguage && (
                            <span
                                className="shrink-0 text-[10px] uppercase tracking-wide text-stone-500 border border-stone-300 rounded px-1 py-0.5"
                                title={`From the ${resource.origin_book_slug} edition`}
                            >
                                other ed.
                            </span>
                        )}
                    </div>
                    <div className="text-xs text-stone-400 truncate mt-0.5">
                        {source?.title}
                        {pageRef ? `, ${pageRef}` : ""}
                    </div>
                </div>
                {expanded ? (
                    <ExpandLessOutlined
                        fontSize="small"
                        className="text-stone-400 shrink-0"
                    />
                ) : (
                    <ExpandMoreOutlined
                        fontSize="small"
                        className="text-stone-400 shrink-0"
                    />
                )}
            </button>

            {/* Expanded detail */}
            {expanded && (
                <div className="px-3 pb-3 pt-1 border-t border-stone-100 space-y-2">
                    {(resource.is_projected || resource.scope !== "work") && (
                        <p className="text-xs text-stone-500 bg-stone-50 rounded px-2 py-1">
                            {resource.is_projected && (
                                <>
                                    From the{" "}
                                    <span className="uppercase">
                                        {resource.origin_language}
                                    </span>{" "}
                                    edition{" "}
                                    <span className="text-stone-400">
                                        ({resource.origin_book_slug})
                                    </span>
                                    {resource.scope !== "work" ? " — " : ""}
                                </>
                            )}
                            {resource.scope !== "work" &&
                                (scopeLabel(
                                    resource.scope,
                                    resource.is_projected,
                                ) ??
                                    resource.scope)}
                        </p>
                    )}

                    {resource.quoted_text && (
                        <Field
                            label={
                                resource.verbatim_kind === "fragmentary"
                                    ? "Fragment"
                                    : "Quote"
                            }
                        >
                            <blockquote className="text-stone-700 italic text-xs border-l-2 border-stone-300 pl-2">
                                {resource.quoted_text}
                            </blockquote>
                        </Field>
                    )}

                    {resource.editor_note && (
                        <Field label="Editor Note">
                            <p className="text-stone-700 text-xs">
                                {resource.editor_note}
                            </p>
                        </Field>
                    )}

                    {source && (
                        <>
                            <Field label="Source">
                                <p className="text-stone-800 text-xs font-medium">
                                    {source.title}
                                </p>
                                {source.parent && (
                                    <p className="text-stone-500 text-xs mt-0.5">
                                        in{" "}
                                        <span className="italic">
                                            {source.parent.title}
                                        </span>
                                    </p>
                                )}
                            </Field>

                            {source.persons.length > 0 && (
                                <Field label="Contributors">
                                    <ul className="text-xs text-stone-700 space-y-0.5">
                                        {source.persons.map((p) => (
                                            <li
                                                key={`${p.person_id}-${p.role}`}
                                            >
                                                {p.name}
                                                <span className="text-stone-400 ml-1">
                                                    ({p.role})
                                                </span>
                                            </li>
                                        ))}
                                    </ul>
                                </Field>
                            )}

                            {source.publisher && (
                                <Field label="Publisher">
                                    <p className="text-stone-700 text-xs">
                                        {source.publisher}
                                        {source.publication_year
                                            ? `, ${source.publication_year}`
                                            : ""}
                                    </p>
                                </Field>
                            )}

                            {source.doi && (
                                <Field label="DOI">
                                    <a
                                        href={`https://doi.org/${source.doi}`}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        className="text-xs text-blue-600 hover:underline"
                                    >
                                        {source.doi}
                                    </a>
                                </Field>
                            )}

                            {source.url && (
                                <Field label="URL">
                                    <a
                                        href={source.url}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        className="text-xs text-blue-600 hover:underline truncate block"
                                    >
                                        {source.url}
                                    </a>
                                </Field>
                            )}
                        </>
                    )}

                    {pageRef && (
                        <Field label="Location in source">
                            <p className="text-stone-700 text-xs">{pageRef}</p>
                        </Field>
                    )}

                    <Field label="Sentences">
                        <p className="text-stone-700 text-xs">
                            {resource.is_projected &&
                            resource.projected_sentence_start_number != null ? (
                                <>
                                    {resource.projected_sentence_end_number !=
                                        null &&
                                    resource.projected_sentence_end_number !==
                                        resource.projected_sentence_start_number
                                        ? `${resource.projected_sentence_start_number}\u2013${resource.projected_sentence_end_number}`
                                        : resource.projected_sentence_start_number}{" "}
                                    ({resource.sentence_kind})
                                    <span className="text-stone-400">
                                        {" \u00b7 anchored at "}
                                        {resource.anchor_sentence_end_number !=
                                        null
                                            ? `${resource.anchor_sentence_start_number}\u2013${resource.anchor_sentence_end_number}`
                                            : resource.anchor_sentence_start_number}{" "}
                                        in the origin edition
                                    </span>
                                </>
                            ) : (
                                <>
                                    {resource.anchor_sentence_end_number != null
                                        ? `${resource.anchor_sentence_start_number}\u2013${resource.anchor_sentence_end_number}`
                                        : resource.anchor_sentence_start_number}{" "}
                                    ({resource.sentence_kind})
                                </>
                            )}
                        </p>
                    </Field>

                    {isEditor && !resource.is_projected && (
                        <div className="flex gap-2 pt-1">
                            <button
                                type="button"
                                onClick={() => onEdit(resource)}
                                className="text-xs text-stone-500 hover:text-stone-700 transition-colors"
                            >
                                Edit
                            </button>
                            <button
                                type="button"
                                onClick={() => onDelete(resource)}
                                className="text-xs text-red-400 hover:text-red-600 transition-colors"
                            >
                                Delete
                            </button>
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}

function Field({
    label,
    children,
}: {
    label: string;
    children: React.ReactNode;
}) {
    return (
        <div>
            <dt className="text-stone-400 text-xs mb-0.5">{label}</dt>
            <dd>{children}</dd>
        </div>
    );
}
