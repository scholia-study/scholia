import DeleteOutlined from "@mui/icons-material/DeleteOutlined";
import EditOutlined from "@mui/icons-material/EditOutlined";
import {
    Chip,
    FormControl,
    IconButton,
    InputLabel,
    MenuItem,
    Paper,
    Select,
    TextField,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute, Link, redirect } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import toast from "react-hot-toast";
import { NoteFormModal } from "#/modules/forms";
import { getGetProfileQueryOptions } from "../api/auth/auth";
import type { NoteWithContextResponse } from "../api/model";
import {
    getListAllNotesQueryKey,
    useDeleteNote,
    useListAllNotes,
} from "../api/quotations/quotations";

export const Route = createFileRoute("/user/notes")({
    beforeLoad: async ({ context }) => {
        const data = await context.queryClient.fetchQuery(
            getGetProfileQueryOptions(),
        );
        if (!data?.data) {
            throw redirect({ to: "/login" });
        }
    },
    component: NotesPage,
});

function sentenceLabel(n: NoteWithContextResponse): string {
    const start = n.anchor_sentence_start_number;
    const end = n.anchor_sentence_end_number;
    if (end == null || end === start) return `Sentence ${start}`;
    return `Sentences ${start}–${end}`;
}

function NotesPage() {
    const queryClient = useQueryClient();
    const [bookFilter, setBookFilter] = useState<string>("");
    const [searchQuery, setSearchQuery] = useState<string>("");
    const [selectedTags, setSelectedTags] = useState<Set<string>>(new Set());
    const [editingNote, setEditingNote] =
        useState<NoteWithContextResponse | null>(null);

    const { data: allNotesData, isLoading } = useListAllNotes({});
    const allNotes = allNotesData?.data?.notes ?? [];

    const availableBooks = useMemo(() => {
        const map = new Map<string, string>();
        for (const n of allNotes) {
            if (!map.has(n.book_slug)) {
                map.set(n.book_slug, n.book_title);
            }
        }
        return [...map.entries()].sort((a, b) => a[1].localeCompare(b[1]));
    }, [allNotes]);

    // Notes filtered by book only (for deriving available tags)
    const bookFilteredNotes = useMemo(() => {
        if (!bookFilter) return allNotes;
        return allNotes.filter((n) => n.book_slug === bookFilter);
    }, [allNotes, bookFilter]);

    // Derive available tags from book-filtered notes
    const availableTags = useMemo(() => {
        const counts = new Map<string, number>();
        for (const n of bookFilteredNotes) {
            for (const t of n.tags) {
                counts.set(t.name, (counts.get(t.name) ?? 0) + 1);
            }
        }
        return [...counts.entries()].sort((a, b) => a[0].localeCompare(b[0]));
    }, [bookFilteredNotes]);

    const toggleTag = (tag: string) => {
        setSelectedTags((prev) => {
            const next = new Set(prev);
            if (next.has(tag)) {
                next.delete(tag);
            } else {
                next.add(tag);
            }
            return next;
        });
    };

    const notes = useMemo(() => {
        let filtered = bookFilteredNotes;
        if (selectedTags.size > 0) {
            filtered = filtered.filter((n) =>
                n.tags.some((t) => selectedTags.has(t.name)),
            );
        }
        if (searchQuery.trim()) {
            const q = searchQuery.toLowerCase();
            filtered = filtered.filter(
                (n) =>
                    n.body.toLowerCase().includes(q) ||
                    n.tags.some((t) => t.name.toLowerCase().includes(q)),
            );
        }
        return filtered;
    }, [bookFilteredNotes, selectedTags, searchQuery]);

    const deleteNoteMutation = useDeleteNote({
        mutation: {
            onSuccess: () => {
                toast.success("Note deleted");
                queryClient.invalidateQueries({
                    queryKey: getListAllNotesQueryKey(),
                });
            },
            onError: () => toast.error("Failed to delete note"),
        },
    });

    const handleDelete = (note: NoteWithContextResponse) => {
        if (window.confirm("Delete this note?")) {
            deleteNoteMutation.mutate({
                slug: note.book_slug,
                id: note.quotation_id,
                noteId: note.id,
            });
        }
    };

    return (
        <div className="w-full max-w-3xl mx-auto px-8 py-16">
            <div className="flex items-center justify-between mb-4">
                <h1 className="text-2xl font-bold text-stone-900">My Notes</h1>
                <FormControl size="small" sx={{ minWidth: 200 }}>
                    <InputLabel>Filter by book</InputLabel>
                    <Select
                        value={bookFilter}
                        label="Filter by book"
                        onChange={(e) => setBookFilter(e.target.value)}
                    >
                        <MenuItem value="">All books</MenuItem>
                        {availableBooks.map(([slug, title]) => (
                            <MenuItem key={slug} value={slug}>
                                {title}
                            </MenuItem>
                        ))}
                    </Select>
                </FormControl>
            </div>
            <div className="mb-4">
                <TextField
                    size="small"
                    fullWidth
                    placeholder="Search notes and tags..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                />
            </div>
            {availableTags.length > 0 && (
                <div className="flex flex-wrap gap-1.5 mb-6">
                    {availableTags.map(([name, count]) => (
                        <Chip
                            key={name}
                            label={`${name} (${count})`}
                            size="small"
                            variant={
                                selectedTags.has(name) ? "filled" : "outlined"
                            }
                            color={
                                selectedTags.has(name) ? "primary" : "default"
                            }
                            onClick={() => toggleTag(name)}
                            sx={{ cursor: "pointer" }}
                        />
                    ))}
                </div>
            )}

            {isLoading && <p className="text-sm text-stone-400">Loading...</p>}

            {!isLoading && notes.length === 0 && (
                <p className="text-sm text-stone-400">No notes yet.</p>
            )}

            <div className="space-y-2">
                {notes.map((note) => (
                    <NoteItem
                        key={note.id}
                        note={note}
                        onEdit={() => setEditingNote(note)}
                        onDelete={() => handleDelete(note)}
                    />
                ))}
            </div>

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
        </div>
    );
}

function NoteItem({
    note,
    onEdit,
    onDelete,
}: {
    note: NoteWithContextResponse;
    onEdit: () => void;
    onDelete: () => void;
}) {
    const date = new Date(note.updated_at);
    const dateStr = date.toLocaleDateString(undefined, {
        month: "short",
        day: "numeric",
        year: "numeric",
    });

    return (
        <Paper
            elevation={0}
            sx={{
                border: "1px solid rgb(214 211 209)",
                p: 1.5,
                transition: "box-shadow 0.15s",
                "&:hover": {
                    boxShadow: 3,
                },
                "&:hover .note-actions": {
                    opacity: "1 !important",
                },
            }}
        >
            <div className="flex items-start gap-2">
                <Link
                    to="/books/$bookSlug/$nodeSlug"
                    params={{
                        bookSlug: note.book_slug,
                        nodeSlug: note.node_slug,
                    }}
                    search={{
                        s:
                            note.anchor_sentence_end_number &&
                            note.anchor_sentence_end_number !==
                                note.anchor_sentence_start_number
                                ? `${note.anchor_sentence_start_number}-${note.anchor_sentence_end_number}`
                                : String(note.anchor_sentence_start_number),
                        r: "1",
                        rv: "notes",
                    }}
                    className="flex-1 min-w-0"
                >
                    <p className="text-sm text-stone-700 whitespace-pre-wrap break-words line-clamp-3">
                        {note.body}
                    </p>
                    {note.tags.length > 0 && (
                        <div className="flex flex-wrap gap-1 mt-1.5">
                            {note.tags.map((tag) => (
                                <Chip
                                    key={tag.id}
                                    label={tag.name}
                                    size="small"
                                    variant="outlined"
                                    sx={{
                                        height: 20,
                                        fontSize: "0.65rem",
                                        borderColor: "rgb(214 211 209)",
                                        color: "rgb(120 113 108)",
                                    }}
                                />
                            ))}
                        </div>
                    )}
                    <div className="flex items-center mt-2 text-[10px] text-stone-400">
                        <span>{note.book_title}</span>
                        <span className="mx-1">&middot;</span>
                        <span>{note.node_label}</span>
                        <span className="mx-1">&middot;</span>
                        <span>{sentenceLabel(note)}</span>
                        <span className="ml-auto">{dateStr}</span>
                    </div>
                </Link>
                <div
                    className="note-actions"
                    style={{
                        opacity: 0,
                        transition: "opacity 0.15s",
                        display: "flex",
                        gap: 2,
                        flexShrink: 0,
                    }}
                >
                    <IconButton
                        size="small"
                        onClick={onEdit}
                        title="Edit note"
                        sx={{ p: 0.5 }}
                    >
                        <EditOutlined sx={{ fontSize: 16 }} />
                    </IconButton>
                    <IconButton
                        size="small"
                        onClick={onDelete}
                        title="Delete note"
                        sx={{ p: 0.5, color: "rgb(168 162 158)" }}
                    >
                        <DeleteOutlined sx={{ fontSize: 16 }} />
                    </IconButton>
                </div>
            </div>
        </Paper>
    );
}
