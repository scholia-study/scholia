import {
    Chip,
    FormControl,
    FormControlLabel,
    InputLabel,
    MenuItem,
    Select,
    Switch,
    TextField,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import toast from "react-hot-toast";
import {
    ArticleQuotationDetailModal,
    type BookQuotation,
    NoteFormModal,
    SavedArticleQuotation,
    SavedBookQuotation,
    sentenceLabel,
    useUnsaveQuotation,
} from "#/modules/quotation";
import { TranslationBadge } from "#/modules/reader";
import {
    getListArticleQuotationsQueryKey,
    useDeleteArticleQuotation,
} from "../api/article-quotations/article-quotations";
import type {
    NoteWithContextResponse,
    UnifiedQuotationResponse,
} from "../api/model";
import {
    getListAllNotesQueryKey,
    getListAllQuotationsQueryKey,
    useDeleteNote,
    useListAllNotes,
    useListAllQuotations,
} from "../api/quotations/quotations";

/** The left rail's key for the article-quotation group. Book slugs can
 *  never collide with it: slugs come from imported books, and no corpus
 *  is named with a leading underscore. */
const ARTICLES_SOURCE = "__articles__";

type SortKey = "quoted" | "annotated";

type QuotationsSearch = {
    source?: string;
    tags?: string[];
    q?: string;
    notes?: boolean;
    sort?: SortKey;
};

export const Route = createFileRoute("/_auth/user/quotations")({
    component: QuotationsPage,
    validateSearch: (search: Record<string, unknown>): QuotationsSearch => {
        const tags = Array.isArray(search.tags)
            ? search.tags.filter((t): t is string => typeof t === "string")
            : typeof search.tags === "string"
              ? [search.tags]
              : undefined;
        return {
            source:
                typeof search.source === "string" ? search.source : undefined,
            tags: tags?.length ? tags : undefined,
            q: typeof search.q === "string" && search.q ? search.q : undefined,
            notes: search.notes === true ? true : undefined,
            sort: search.sort === "annotated" ? "annotated" : undefined,
        };
    },
});

interface SourceEntry {
    key: string;
    title: string;
    translationLabel?: string | null;
    count: number;
}

function QuotationsPage() {
    const queryClient = useQueryClient();
    const {
        source,
        tags,
        q: searchQuery,
        notes: notesOnly,
        sort,
    } = Route.useSearch();
    const navigate = Route.useNavigate();

    const selectedTags = useMemo(() => new Set(tags ?? []), [tags]);

    const [editingNote, setEditingNote] =
        useState<NoteWithContextResponse | null>(null);
    const [addingNoteTo, setAddingNoteTo] = useState<BookQuotation | null>(
        null,
    );
    const [selectedArticleQuotation, setSelectedArticleQuotation] = useState<
        string | null
    >(null);

    const { data: quotationsData, isLoading: quotationsLoading } =
        useListAllQuotations({});
    const { data: notesData, isLoading: notesLoading } = useListAllNotes({});

    const allQuotations = quotationsData?.data?.quotations ?? [];
    const allNotes = useMemo(() => notesData?.data?.notes ?? [], [notesData]);
    const quotationLimits = quotationsData?.data?.limits;
    const noteLimits = notesData?.data?.limits;

    const notesByQuotation = useMemo(() => {
        const map = new Map<string, NoteWithContextResponse[]>();
        for (const n of allNotes) {
            const list = map.get(n.quotation_id);
            if (list) list.push(n);
            else map.set(n.quotation_id, [n]);
        }
        // Oldest first within a quotation: annotation reads as a thread.
        for (const list of map.values()) {
            list.sort((a, b) => a.created_at.localeCompare(b.created_at));
        }
        return map;
    }, [allNotes]);

    const sources = useMemo(() => {
        const books = new Map<string, SourceEntry>();
        let articleCount = 0;
        for (const quotation of allQuotations) {
            if (quotation.source_type === "article") {
                articleCount += 1;
                continue;
            }
            const existing = books.get(quotation.book_slug);
            if (existing) {
                existing.count += 1;
            } else {
                books.set(quotation.book_slug, {
                    key: quotation.book_slug,
                    title: quotation.book_title,
                    translationLabel: quotation.translation_label,
                    count: 1,
                });
            }
        }
        const entries = [...books.values()].sort((a, b) =>
            a.title.localeCompare(b.title),
        );
        if (articleCount > 0) {
            entries.push({
                key: ARTICLES_SOURCE,
                title: "Articles",
                count: articleCount,
            });
        }
        return entries;
    }, [allQuotations]);

    const sourceFiltered = useMemo(() => {
        if (!source) return allQuotations;
        if (source === ARTICLES_SOURCE) {
            return allQuotations.filter((q) => q.source_type === "article");
        }
        return allQuotations.filter(
            (q) => q.source_type === "book" && q.book_slug === source,
        );
    }, [allQuotations, source]);

    const availableTags = useMemo(() => {
        const visible = new Set(sourceFiltered.map((q) => q.id));
        const counts = new Map<string, number>();
        for (const note of allNotes) {
            if (!visible.has(note.quotation_id)) continue;
            for (const tag of note.tags) {
                counts.set(tag.name, (counts.get(tag.name) ?? 0) + 1);
            }
        }
        return [...counts.entries()].sort((a, b) => a[0].localeCompare(b[0]));
    }, [sourceFiltered, allNotes]);

    const visibleQuotations = useMemo(() => {
        const matchesTags = (quotationId: string) =>
            selectedTags.size === 0 ||
            (notesByQuotation.get(quotationId) ?? []).some((n) =>
                n.tags.some((t) => selectedTags.has(t.name)),
            );

        const needle = searchQuery?.trim().toLowerCase();
        const matchesSearch = (quotation: UnifiedQuotationResponse) => {
            if (!needle) return true;
            const haystack: string[] =
                quotation.source_type === "book"
                    ? [
                          quotation.book_title,
                          quotation.node_label,
                          quotation.start_text_snippet ?? "",
                          quotation.end_text_snippet ?? "",
                      ]
                    : [
                          quotation.article_title,
                          quotation.author_display_name,
                          quotation.text_snippet,
                      ];
            for (const note of notesByQuotation.get(quotation.id) ?? []) {
                haystack.push(note.body, ...note.tags.map((t) => t.name));
            }
            return haystack.some((h) => h.toLowerCase().includes(needle));
        };

        const filtered = sourceFiltered.filter(
            (quotation) =>
                (!notesOnly ||
                    (notesByQuotation.get(quotation.id) ?? []).length > 0) &&
                matchesTags(quotation.id) &&
                matchesSearch(quotation),
        );

        if (sort !== "annotated") return filtered;

        // Most recently annotated first; an un-annotated quotation falls
        // back to when it was saved, so it never floats above live work.
        const activity = (quotation: UnifiedQuotationResponse) => {
            const quotationNotes = notesByQuotation.get(quotation.id) ?? [];
            return quotationNotes.reduce(
                (latest, n) => (n.updated_at > latest ? n.updated_at : latest),
                quotation.created_at,
            );
        };
        return [...filtered].sort((a, b) =>
            activity(b).localeCompare(activity(a)),
        );
    }, [
        sourceFiltered,
        notesByQuotation,
        selectedTags,
        searchQuery,
        notesOnly,
        sort,
    ]);

    const updateSearch = (
        patch: Partial<QuotationsSearch>,
        replace = false,
    ) => {
        navigate({
            search: (prev) => ({ ...prev, ...patch }),
            startTransition: true,
            replace,
        });
    };

    const selectSource = (key: string | undefined) => {
        // Tags are derived per source, so a tag selection cannot survive
        // the switch — it would silently filter everything away.
        updateSearch({ source: key, tags: undefined });
    };

    const toggleTag = (name: string) => {
        const next = new Set(selectedTags);
        if (next.has(name)) next.delete(name);
        else next.add(name);
        updateSearch({ tags: next.size ? [...next] : undefined });
    };

    const { requestUnsave, UnsaveDialog } = useUnsaveQuotation({});

    const deleteArticleQuotation = useDeleteArticleQuotation();
    const handleDeleteArticleQuotation = async (id: string) => {
        await deleteArticleQuotation.mutateAsync({ id });
        queryClient.invalidateQueries({
            queryKey: getListAllQuotationsQueryKey(),
        });
        queryClient.invalidateQueries({
            queryKey: getListArticleQuotationsQueryKey(),
        });
    };

    const deleteNoteMutation = useDeleteNote({
        mutation: {
            onSuccess: () => {
                toast.success("Note deleted");
                queryClient.invalidateQueries({
                    queryKey: getListAllNotesQueryKey(),
                });
                queryClient.invalidateQueries({
                    queryKey: getListAllQuotationsQueryKey(),
                });
            },
            onError: () => toast.error("Failed to delete note"),
        },
    });

    const handleDeleteNote = (note: NoteWithContextResponse) => {
        if (window.confirm("Delete this note?")) {
            deleteNoteMutation.mutate({
                slug: note.book_slug,
                id: note.quotation_id,
                noteId: note.id,
            });
        }
    };

    const isLoading = quotationsLoading || notesLoading;
    const isFiltered = Boolean(
        source || selectedTags.size || searchQuery || notesOnly,
    );

    return (
        // w-full matters: this sits in a flex column, where mx-auto
        // cancels the default stretch and the box would otherwise
        // shrink-to-fit its content — the grid must hold its width
        // whether the list is full, loading, or empty.
        <div className="w-full max-w-3xl lg:max-w-7xl mx-auto px-8 py-16">
            <h1 className="text-2xl font-bold text-stone-900 mb-6">
                Quotations &amp; Notes
            </h1>

            <div className="lg:grid lg:grid-cols-[17rem_minmax(0,1fr)_14rem] lg:gap-10">
                <aside className="hidden lg:block">
                    <div className="sticky top-8 max-h-[calc(100vh-6rem)] overflow-y-auto">
                        <SourceSidebar
                            sources={sources}
                            selected={source}
                            onSelect={selectSource}
                        />
                    </div>
                </aside>

                <div className="min-w-0">
                    <div className="flex flex-wrap items-center gap-3 mb-4">
                        <TextField
                            size="small"
                            placeholder="Search quotations, notes and tags..."
                            value={searchQuery ?? ""}
                            onChange={(e) =>
                                // Replace, not push: a pushed entry per
                                // keystroke would bury the page in history.
                                updateSearch(
                                    { q: e.target.value || undefined },
                                    true,
                                )
                            }
                            sx={{ flex: "1 1 240px" }}
                        />
                        <FormControl size="small" sx={{ minWidth: 170 }}>
                            <InputLabel>Sort</InputLabel>
                            <Select
                                value={sort ?? "quoted"}
                                label="Sort"
                                onChange={(e) =>
                                    updateSearch({
                                        sort:
                                            e.target.value === "annotated"
                                                ? "annotated"
                                                : undefined,
                                    })
                                }
                            >
                                <MenuItem value="quoted">
                                    Recently quoted
                                </MenuItem>
                                <MenuItem value="annotated">
                                    Recently annotated
                                </MenuItem>
                            </Select>
                        </FormControl>
                        <FormControlLabel
                            control={
                                <Switch
                                    size="small"
                                    checked={Boolean(notesOnly)}
                                    onChange={(e) =>
                                        updateSearch({
                                            notes: e.target.checked
                                                ? true
                                                : undefined,
                                        })
                                    }
                                />
                            }
                            label={
                                <span className="text-sm text-stone-500">
                                    With notes
                                </span>
                            }
                        />
                    </div>

                    {/* Small screens: the rails collapse above the list */}
                    <div className="lg:hidden">
                        <SourceChips
                            sources={sources}
                            selected={source}
                            onSelect={selectSource}
                        />
                        <TagChips
                            tags={availableTags}
                            selected={selectedTags}
                            onToggle={toggleTag}
                        />
                    </div>

                    {isLoading && (
                        <p className="text-sm text-stone-400">Loading...</p>
                    )}

                    {!isLoading && visibleQuotations.length === 0 && (
                        <p className="text-sm text-stone-400">
                            {isFiltered ? (
                                <>
                                    Nothing matches this filter.{" "}
                                    <button
                                        type="button"
                                        onClick={() =>
                                            navigate({ search: () => ({}) })
                                        }
                                        className="text-stone-600 underline hover:text-stone-900 cursor-pointer"
                                    >
                                        Clear filters
                                    </button>
                                </>
                            ) : (
                                "No saved quotations yet."
                            )}
                        </p>
                    )}

                    <div className="space-y-4">
                        {visibleQuotations.map((quotation) =>
                            quotation.source_type === "book" ? (
                                <SavedBookQuotation
                                    key={quotation.id}
                                    quotation={quotation}
                                    notes={
                                        notesByQuotation.get(quotation.id) ?? []
                                    }
                                    onUnsave={() => requestUnsave(quotation)}
                                    onAddNote={() => setAddingNoteTo(quotation)}
                                    onEditNote={setEditingNote}
                                    onDeleteNote={handleDeleteNote}
                                />
                            ) : (
                                <SavedArticleQuotation
                                    key={quotation.id}
                                    quotation={quotation}
                                    onViewFull={() =>
                                        setSelectedArticleQuotation(
                                            quotation.id,
                                        )
                                    }
                                    onDelete={() =>
                                        handleDeleteArticleQuotation(
                                            quotation.id,
                                        )
                                    }
                                />
                            ),
                        )}
                    </div>
                </div>

                <aside className="hidden lg:block">
                    <div className="sticky top-8 max-h-[calc(100vh-6rem)] overflow-y-auto">
                        <TagSidebar
                            tags={availableTags}
                            selected={selectedTags}
                            onToggle={toggleTag}
                        />
                        <Usage
                            quotationLimits={quotationLimits}
                            noteLimits={noteLimits}
                        />
                    </div>
                </aside>
            </div>

            {UnsaveDialog}

            {addingNoteTo && (
                <NoteFormModal
                    open
                    onClose={() => setAddingNoteTo(null)}
                    bookSlug={addingNoteTo.book_slug}
                    quotationId={addingNoteTo.id}
                    mode="create"
                    sentenceContext={`${addingNoteTo.node_label} · ${sentenceLabel(addingNoteTo)}`}
                />
            )}

            {editingNote && (
                <NoteFormModal
                    key={editingNote.id}
                    open
                    onClose={() => {
                        setEditingNote(null);
                        queryClient.invalidateQueries({
                            queryKey: getListAllNotesQueryKey(),
                        });
                    }}
                    bookSlug={editingNote.book_slug}
                    quotationId={editingNote.quotation_id}
                    mode="edit"
                    initialData={{
                        id: editingNote.id,
                        body: editingNote.body,
                        tags: editingNote.tags,
                        created_at: editingNote.created_at,
                        updated_at: editingNote.updated_at,
                    }}
                    sentenceContext={`${editingNote.node_label} · ${sentenceLabel(editingNote)}`}
                />
            )}

            {selectedArticleQuotation && (
                <ArticleQuotationDetailModal
                    id={selectedArticleQuotation}
                    onClose={() => setSelectedArticleQuotation(null)}
                />
            )}
        </div>
    );
}

const railItemClass = (active: boolean) =>
    `block w-full text-left px-2 py-1 rounded text-sm cursor-pointer ${
        active
            ? "text-stone-900 bg-stone-200 font-medium"
            : "text-stone-500 hover:text-stone-900 hover:bg-stone-100"
    }`;

function SourceSidebar({
    sources,
    selected,
    onSelect,
}: {
    sources: SourceEntry[];
    selected?: string;
    onSelect: (key: string | undefined) => void;
}) {
    return (
        <div>
            <h2 className="text-xs uppercase tracking-wide text-stone-400 mb-2">
                Sources
            </h2>
            <ul className="space-y-0.5">
                <li>
                    <button
                        type="button"
                        className={railItemClass(!selected)}
                        onClick={() => onSelect(undefined)}
                    >
                        All
                    </button>
                </li>
                {sources.map((s) => (
                    <li key={s.key}>
                        <button
                            type="button"
                            className={railItemClass(selected === s.key)}
                            onClick={() =>
                                onSelect(selected === s.key ? undefined : s.key)
                            }
                        >
                            <span className="flex items-center gap-1.5">
                                {s.key !== ARTICLES_SOURCE && (
                                    <TranslationBadge
                                        label={s.translationLabel}
                                        title={s.title}
                                    />
                                )}
                                <span className="flex-1 min-w-0 truncate">
                                    {s.title}
                                </span>
                                <span className="text-xs text-stone-400">
                                    {s.count}
                                </span>
                            </span>
                        </button>
                    </li>
                ))}
            </ul>
        </div>
    );
}

function TagSidebar({
    tags,
    selected,
    onToggle,
}: {
    tags: [string, number][];
    selected: Set<string>;
    onToggle: (name: string) => void;
}) {
    if (tags.length === 0) return null;

    return (
        <div className="mb-6">
            <h2 className="text-xs uppercase tracking-wide text-stone-400 mb-2">
                Tags
            </h2>
            <ul className="space-y-0.5">
                {tags.map(([name, count]) => (
                    <li key={name}>
                        <button
                            type="button"
                            className={railItemClass(selected.has(name))}
                            onClick={() => onToggle(name)}
                        >
                            <span className="flex items-center gap-1.5">
                                <span className="flex-1 min-w-0 truncate">
                                    {name}
                                </span>
                                <span className="text-xs text-stone-400">
                                    {count}
                                </span>
                            </span>
                        </button>
                    </li>
                ))}
            </ul>
        </div>
    );
}

function SourceChips({
    sources,
    selected,
    onSelect,
}: {
    sources: SourceEntry[];
    selected?: string;
    onSelect: (key: string | undefined) => void;
}) {
    if (sources.length === 0) return null;

    return (
        <div className="flex flex-wrap gap-1.5 mb-3">
            <Chip
                label="All"
                size="small"
                variant={!selected ? "filled" : "outlined"}
                onClick={() => onSelect(undefined)}
                sx={{ fontSize: "0.75rem" }}
            />
            {sources.map((s) => (
                <Chip
                    key={s.key}
                    label={`${s.title} (${s.count})`}
                    size="small"
                    color={selected === s.key ? "primary" : "default"}
                    variant={selected === s.key ? "filled" : "outlined"}
                    onClick={() =>
                        onSelect(selected === s.key ? undefined : s.key)
                    }
                    sx={{ fontSize: "0.75rem" }}
                />
            ))}
        </div>
    );
}

function TagChips({
    tags,
    selected,
    onToggle,
}: {
    tags: [string, number][];
    selected: Set<string>;
    onToggle: (name: string) => void;
}) {
    if (tags.length === 0) return null;

    return (
        <div className="flex flex-wrap gap-1.5 mb-6">
            {tags.map(([name, count]) => (
                <Chip
                    key={name}
                    label={`${name} (${count})`}
                    size="small"
                    color={selected.has(name) ? "primary" : "default"}
                    variant={selected.has(name) ? "filled" : "outlined"}
                    onClick={() => onToggle(name)}
                    sx={{ fontSize: "0.75rem" }}
                />
            ))}
        </div>
    );
}

function Usage({
    quotationLimits,
    noteLimits,
}: {
    quotationLimits?: { current: number; max: number };
    noteLimits?: { current: number; max: number };
}) {
    // Only metered plans see a counter; on an unmetered one it is noise.
    const showQuotations = quotationLimits ? quotationLimits.max <= 50 : false;
    const showNotes = noteLimits ? noteLimits.max <= 50 : false;
    if (!showQuotations && !showNotes) return null;

    return (
        <div className="text-xs text-stone-400 space-y-0.5">
            {showQuotations && quotationLimits && (
                <div>
                    {quotationLimits.current}/{quotationLimits.max} quotations
                </div>
            )}
            {showNotes && noteLimits && (
                <div>
                    {noteLimits.current}/{noteLimits.max} notes
                </div>
            )}
        </div>
    );
}
