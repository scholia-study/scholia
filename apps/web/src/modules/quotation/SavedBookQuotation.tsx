import AddOutlined from "@mui/icons-material/AddOutlined";
import DeleteOutlined from "@mui/icons-material/DeleteOutlined";
import EditOutlined from "@mui/icons-material/EditOutlined";
import { Chip, IconButton, Paper, Tooltip } from "@mui/material";
import { Link } from "@tanstack/react-router";
import type {
    NoteWithContextResponse,
    UnifiedQuotationResponse,
} from "#/api/model";
import { TranslationBadge } from "#/modules/reader";
import { readerAnchorSearch, sentenceLabel } from "./anchorLink";

export type BookQuotation = Extract<
    UnifiedQuotationResponse,
    { source_type: "book" }
>;

/** Drama attribution: which character speaks the quoted line. Verbatim, so
 *  it keeps any stage parenthetical the curated speaker line carries — and
 *  stays un-uppercased, since that parenthetical is prose. */
function Speaker({ name }: { name?: string | null }) {
    if (!name) return null;
    return <span className="font-semibold text-stone-500">{name} </span>;
}

function formatDate(value: string) {
    return new Date(value).toLocaleDateString(undefined, {
        month: "short",
        day: "numeric",
        year: "numeric",
    });
}

export function SavedBookQuotation({
    quotation: q,
    notes,
    onUnsave,
    onAddNote,
    onEditNote,
    onDeleteNote,
}: {
    quotation: BookQuotation;
    notes: NoteWithContextResponse[];
    onUnsave: () => void;
    onAddNote: () => void;
    onEditNote: (note: NoteWithContextResponse) => void;
    onDeleteNote: (note: NoteWithContextResponse) => void;
}) {
    return (
        <div>
            <Paper
                elevation={0}
                sx={{
                    border: "1px solid rgb(214 211 209)",
                    borderLeft: "3px solid rgb(168 162 158)",
                    p: 1.5,
                    transition: "box-shadow 0.15s",
                    "&:hover": { boxShadow: 3 },
                    "&:hover .card-actions": { opacity: 1 },
                }}
            >
                <div className="flex items-start gap-1">
                    <Link
                        to="/books/$bookSlug/$nodeSlug"
                        params={{
                            bookSlug: q.book_slug,
                            nodeSlug: q.node_slug,
                        }}
                        search={readerAnchorSearch(q)}
                        className="flex-1 min-w-0"
                    >
                        <div className="text-xs text-stone-400 mb-1 flex items-center gap-1.5 flex-wrap">
                            <TranslationBadge
                                label={q.translation_label}
                                title={q.book_title}
                            />
                            <span>
                                {q.book_title} &middot; {q.node_label} &middot;{" "}
                                {sentenceLabel(q)}
                            </span>
                        </div>
                        {q.start_text_snippet && (
                            <p className="text-sm text-stone-700 truncate">
                                <Speaker name={q.start_speaker} />
                                &ldquo;{q.start_text_snippet}&rdquo;
                                {q.end_text_snippet && (
                                    <span className="text-stone-400">
                                        {" "}
                                        &hellip;{" "}
                                        <Speaker name={q.end_speaker} />
                                        &ldquo;{q.end_text_snippet}&rdquo;
                                    </span>
                                )}
                            </p>
                        )}
                    </Link>
                    <div className="relative shrink-0 self-center">
                        <div className="text-[10px] text-stone-300 text-right">
                            {formatDate(q.created_at)}
                        </div>
                        <div className="card-actions absolute inset-0 flex items-center justify-end gap-0.5 bg-white opacity-0 transition-opacity">
                            <Tooltip title="Add note">
                                <IconButton
                                    size="small"
                                    onClick={onAddNote}
                                    sx={{ p: 0.5, color: "rgb(168 162 158)" }}
                                >
                                    <AddOutlined sx={{ fontSize: 16 }} />
                                </IconButton>
                            </Tooltip>
                            <Tooltip title="Remove quotation">
                                <IconButton
                                    size="small"
                                    onClick={onUnsave}
                                    sx={{ p: 0.5, color: "rgb(168 162 158)" }}
                                >
                                    <DeleteOutlined sx={{ fontSize: 16 }} />
                                </IconButton>
                            </Tooltip>
                        </div>
                    </div>
                </div>
            </Paper>

            {notes.length > 0 && (
                <ul className="mt-1.5 ml-6 space-y-1.5">
                    {notes.map((note) => (
                        <NoteCard
                            key={note.id}
                            note={note}
                            onEdit={() => onEditNote(note)}
                            onDelete={() => onDeleteNote(note)}
                        />
                    ))}
                </ul>
            )}
        </div>
    );
}

function NoteCard({
    note,
    onEdit,
    onDelete,
}: {
    note: NoteWithContextResponse;
    onEdit: () => void;
    onDelete: () => void;
}) {
    return (
        <li>
            <Paper
                elevation={0}
                sx={{
                    border: "1px solid rgb(226 232 240)",
                    backgroundColor: "rgb(241 245 249)",
                    p: 1.25,
                    display: "flex",
                    alignItems: "flex-start",
                    gap: 1,
                    transition: "box-shadow 0.15s",
                    "&:hover": { boxShadow: 2 },
                    "&:hover .note-actions": { opacity: 1 },
                }}
            >
                <div className="flex-1 min-w-0">
                    <p className="text-sm text-stone-700 whitespace-pre-wrap break-words">
                        {note.body}
                    </p>
                    <div className="flex flex-wrap items-center gap-1 mt-1.5">
                        {note.tags.map((tag) => (
                            <Chip
                                key={tag.id}
                                label={tag.name}
                                size="small"
                                variant="outlined"
                                sx={{
                                    height: 20,
                                    fontSize: "0.65rem",
                                    backgroundColor: "rgb(255 255 255)",
                                    borderColor: "rgb(214 211 209)",
                                    color: "rgb(120 113 108)",
                                }}
                            />
                        ))}
                        <span className="text-[10px] text-stone-400">
                            {formatDate(note.updated_at)}
                        </span>
                    </div>
                </div>
                <div className="note-actions flex shrink-0 gap-0.5 opacity-0 transition-opacity">
                    <IconButton
                        size="small"
                        onClick={onEdit}
                        title="Edit note"
                        sx={{ p: 0.5 }}
                    >
                        <EditOutlined sx={{ fontSize: 15 }} />
                    </IconButton>
                    <IconButton
                        size="small"
                        onClick={onDelete}
                        title="Delete note"
                        sx={{ p: 0.5, color: "rgb(168 162 158)" }}
                    >
                        <DeleteOutlined sx={{ fontSize: 15 }} />
                    </IconButton>
                </div>
            </Paper>
        </li>
    );
}
