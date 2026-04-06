import {
    Chip,
    FormControl,
    InputLabel,
    MenuItem,
    Select,
} from "@mui/material";
import { Link, createFileRoute, redirect } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { getGetProfileQueryOptions } from "../api/auth/auth";
import type { NoteWithContextResponse } from "../api/model";
import { useListAllNotes } from "../api/quotations/quotations";

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
    const [bookFilter, setBookFilter] = useState<string>("");

    // Fetch all notes (unfiltered) to derive available books
    const { data: allNotesData, isLoading } = useListAllNotes({});
    const allNotes = allNotesData?.data?.notes ?? [];

    // Derive book list from actual data
    const availableBooks = useMemo(() => {
        const map = new Map<string, string>();
        for (const n of allNotes) {
            if (!map.has(n.book_slug)) {
                map.set(n.book_slug, n.book_title);
            }
        }
        return [...map.entries()].sort((a, b) => a[1].localeCompare(b[1]));
    }, [allNotes]);

    // Apply client-side filter
    const notes = useMemo(() => {
        if (!bookFilter) return allNotes;
        return allNotes.filter((n) => n.book_slug === bookFilter);
    }, [allNotes, bookFilter]);

    return (
        <div className="max-w-3xl mx-auto px-8 py-16">
            <div className="flex items-center justify-between mb-8">
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

            {isLoading && (
                <p className="text-sm text-stone-400">Loading...</p>
            )}

            {!isLoading && notes.length === 0 && (
                <p className="text-sm text-stone-400">No notes yet.</p>
            )}

            <div className="space-y-2">
                {notes.map((note) => (
                    <NoteItem key={note.id} note={note} />
                ))}
            </div>
        </div>
    );
}

function NoteItem({ note }: { note: NoteWithContextResponse }) {
    const date = new Date(note.updated_at);
    const dateStr = date.toLocaleDateString(undefined, {
        month: "short",
        day: "numeric",
        year: "numeric",
    });

    return (
        <Link
            to="/books/$bookSlug/$nodeSlug"
            params={{
                bookSlug: note.book_slug,
                nodeSlug: note.node_slug,
            }}
            search={{
                s: note.anchor_sentence_end_number && note.anchor_sentence_end_number !== note.anchor_sentence_start_number
                    ? `${note.anchor_sentence_start_number}-${note.anchor_sentence_end_number}`
                    : String(note.anchor_sentence_start_number),
                r: "1",
                rv: "notes",
            }}
            className="block border border-stone-300 rounded p-3 hover:shadow-md transition-all"
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
    );
}
