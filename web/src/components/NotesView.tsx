import AddOutlined from "@mui/icons-material/AddOutlined";
import FavoriteBorderOutlined from "@mui/icons-material/FavoriteBorderOutlined";
import FavoriteOutlined from "@mui/icons-material/FavoriteOutlined";
import DeleteOutlined from "@mui/icons-material/DeleteOutlined";
import EditOutlined from "@mui/icons-material/EditOutlined";
import {
    Button,
    Chip,
    Dialog,
    DialogActions,
    DialogContent,
    DialogContentText,
    DialogTitle,
    IconButton,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import toast from "react-hot-toast";
import type {
    FootnoteSentenceResponse,
    NoteResponse,
    QuotationResponse,
    SentenceResponse,
} from "../api/model";
import {
    getListNotesQueryKey,
    getListQuotationsQueryKey,
    useCreateQuotation,
    useDeleteNote,
    useDeleteQuotation,
    useListNotes,
    useListQuotations,
} from "../api/quotations/quotations";
import { parseRangeKey } from "./BlockRenderer";
import { getSentenceRange } from "./CommentaryView";

interface NotesViewProps {
    bookSlug: string;
    activeNodeId: string | undefined;
    selectedSentence:
        | SentenceResponse
        | FootnoteSentenceResponse
        | (SentenceResponse | FootnoteSentenceResponse)[]
        | undefined;
    selectedSentenceId: string | undefined;
    onOpenNoteModal: (quotationId: string, note?: NoteResponse) => void;
}

function quotationLabel(q: QuotationResponse): string {
    const start = q.anchor_sentence_start_number;
    const end = q.anchor_sentence_end_number;
    if (end == null || end === start) return `Sentence ${start}`;
    return `Sentences ${start}–${end}`;
}

export function NotesView({
    bookSlug,
    activeNodeId,
    selectedSentence,
    selectedSentenceId,
    onOpenNoteModal,
}: NotesViewProps) {
    const queryClient = useQueryClient();

    // Derive range from selectedSentence if available, otherwise fall back to parsing selectedSentenceId
    const range = useMemo(() => {
        const fromSentence = getSentenceRange(selectedSentence);
        if (fromSentence) return fromSentence;
        if (!selectedSentenceId) return null;
        const parsed = parseRangeKey(selectedSentenceId);
        if (parsed) {
            return { start: parsed[0], end: parsed[1], kind: "body" as const };
        }
        const num = Number(selectedSentenceId);
        if (!Number.isNaN(num)) {
            return { start: num, end: num, kind: "body" as const };
        }
        return null;
    }, [selectedSentence, selectedSentenceId]);

    // Fetch all quotations for this node
    const { data: quotationsData } = useListQuotations(
        bookSlug,
        { node_id: activeNodeId ?? "" },
        { query: { enabled: !!activeNodeId } },
    );

    // Find exact-match quotation for current selection (for save/unsave/add actions)
    const exactQuotation = useMemo(() => {
        if (!range || !quotationsData?.data?.quotations) return undefined;
        return quotationsData.data.quotations.find((q) => {
            const startMatch =
                q.anchor_sentence_start_number === range.start;
            const endMatch =
                range.start === range.end
                    ? q.anchor_sentence_end_number == null ||
                      q.anchor_sentence_end_number === range.start
                    : q.anchor_sentence_end_number === range.end;
            const kindMatch = q.sentence_kind === range.kind;
            return startMatch && endMatch && kindMatch;
        });
    }, [range, quotationsData]);

    // Find all quotations whose range overlaps with the selected range
    const overlappingQuotations = useMemo(() => {
        if (!range || !quotationsData?.data?.quotations) return [];
        return quotationsData.data.quotations.filter((q) => {
            if (q.sentence_kind !== range.kind) return false;
            const qStart = q.anchor_sentence_start_number;
            const qEnd = q.anchor_sentence_end_number ?? qStart;
            return qStart <= range.end && qEnd >= range.start;
        });
    }, [range, quotationsData]);

    const createQuotation = useCreateQuotation({
        mutation: {
            onSuccess: () => {
                toast.success("Quotation saved");
                if (activeNodeId) {
                    queryClient.invalidateQueries({
                        queryKey: getListQuotationsQueryKey(bookSlug, {
                            node_id: activeNodeId,
                        }),
                    });
                }
            },
            onError: () => toast.error("Failed to save quotation"),
        },
    });

    const deleteQuotation = useDeleteQuotation({
        mutation: {
            onSuccess: () => {
                toast.success("Quotation removed");
                if (activeNodeId) {
                    queryClient.invalidateQueries({
                        queryKey: getListQuotationsQueryKey(bookSlug, {
                            node_id: activeNodeId,
                        }),
                    });
                }
            },
            onError: () => toast.error("Failed to remove quotation"),
        },
    });

    const handleSaveQuotation = () => {
        if (!range) return;
        createQuotation.mutate({
            slug: bookSlug,
            data: {
                sentence_start: range.start,
                sentence_end:
                    range.start !== range.end ? range.end : undefined,
                sentence_kind: range.kind,
            },
        });
    };

    const [unsaveTarget, setUnsaveTarget] = useState<QuotationResponse | null>(null);

    const confirmUnsave = () => {
        if (unsaveTarget) {
            deleteQuotation.mutate({ slug: bookSlug, id: unsaveTarget.id });
            setUnsaveTarget(null);
        }
    };

    const handleAddNote = () => {
        if (exactQuotation) {
            onOpenNoteModal(exactQuotation.id);
            return;
        }
        // Auto-save quotation first, then open modal
        if (!range) return;
        createQuotation.mutate(
            {
                slug: bookSlug,
                data: {
                    sentence_start: range.start,
                    sentence_end:
                        range.start !== range.end ? range.end : undefined,
                    sentence_kind: range.kind,
                },
            },
            {
                onSuccess: (result) => {
                    if (result.status === 200) {
                        onOpenNoteModal(result.data.quotation.id);
                    }
                },
            },
        );
    };

    if (!range) {
        return (
            <div className="flex-1 overflow-y-auto p-4">
                <p className="text-sm text-stone-400">
                    Select a sentence to view or add notes.
                </p>
            </div>
        );
    }

    const rangeLabel =
        range.start === range.end
            ? `Sentence ${range.start}`
            : `Sentences ${range.start}–${range.end}`;

    return (
        <div className="flex-1 overflow-y-auto flex flex-col">
            {/* Toolbar */}
            <div className="px-3 py-2 border-b border-stone-100 flex items-center gap-2 shrink-0">
                <span className="text-xs text-stone-500 flex-1">
                    {rangeLabel}
                    {range.kind === "footnote" && (
                        <span className="text-stone-400 ml-1">(fn)</span>
                    )}
                </span>
                {!exactQuotation && (
                    <button
                        onClick={handleSaveQuotation}
                        disabled={createQuotation.isPending}
                        className="text-xs px-2 py-1 rounded border border-stone-200 hover:bg-stone-100 text-stone-600 transition-colors disabled:opacity-50 flex items-center gap-1"
                    >
                        <FavoriteBorderOutlined sx={{ fontSize: 14 }} />
                        Save
                    </button>
                )}
                {exactQuotation && (
                    <>
                        <IconButton
                            size="small"
                            onClick={handleAddNote}
                            title="Add note"
                        >
                            <AddOutlined fontSize="small" />
                        </IconButton>
                        <IconButton
                            size="small"
                            onClick={() => setUnsaveTarget(exactQuotation)}
                            title="Unsave quotation"
                            sx={{ color: "rgb(168 162 158)" }}
                        >
                            <FavoriteOutlined fontSize="small" />
                        </IconButton>
                    </>
                )}
            </div>

            {/* Notes list — grouped by overlapping quotation */}
            <div className="flex-1 overflow-y-auto p-2 space-y-2">
                {overlappingQuotations.length === 0 && !exactQuotation && (
                    <div className="p-3 text-center">
                        <p className="text-sm text-stone-400 mb-2">
                            Not saved yet.
                        </p>
                        <button
                            onClick={handleAddNote}
                            disabled={createQuotation.isPending}
                            className="text-xs px-3 py-1.5 rounded bg-stone-100 hover:bg-stone-200 text-stone-600 transition-colors disabled:opacity-50"
                        >
                            Add Note
                        </button>
                    </div>
                )}

                {overlappingQuotations.map((q) => (
                    <QuotationNotesGroup
                        key={q.id}
                        quotation={q}
                        bookSlug={bookSlug}
                        activeNodeId={activeNodeId}
                        isExact={q.id === exactQuotation?.id}
                        showLabel={overlappingQuotations.length > 1}
                        onOpenNoteModal={onOpenNoteModal}
                    />
                ))}

                {exactQuotation &&
                    !overlappingQuotations.some(
                        (q) => q.id === exactQuotation.id,
                    ) && (
                        <QuotationNotesGroup
                            key={exactQuotation.id}
                            quotation={exactQuotation}
                            bookSlug={bookSlug}
                            activeNodeId={activeNodeId}
                            isExact
                            showLabel={overlappingQuotations.length > 0}
                            onOpenNoteModal={onOpenNoteModal}
                        />
                    )}
            </div>

            {/* Unsave confirmation dialog */}
            <Dialog
                open={unsaveTarget != null}
                onClose={() => setUnsaveTarget(null)}
            >
                <DialogTitle sx={{ fontSize: "0.95rem" }}>
                    Remove saved quotation?
                </DialogTitle>
                <DialogContent>
                    <DialogContentText sx={{ fontSize: "0.875rem" }}>
                        {unsaveTarget && unsaveTarget.note_count > 0
                            ? `This will permanently delete ${unsaveTarget.note_count} note${unsaveTarget.note_count > 1 ? "s" : ""} attached to this quotation.`
                            : "This will remove the saved quotation."}
                    </DialogContentText>
                </DialogContent>
                <DialogActions sx={{ px: 3, pb: 2 }}>
                    <Button
                        onClick={() => setUnsaveTarget(null)}
                        size="small"
                    >
                        Cancel
                    </Button>
                    <Button
                        onClick={confirmUnsave}
                        size="small"
                        color="error"
                        variant="contained"
                    >
                        Remove
                    </Button>
                </DialogActions>
            </Dialog>
        </div>
    );
}

function QuotationNotesGroup({
    quotation,
    bookSlug,
    activeNodeId,
    isExact,
    showLabel,
    onOpenNoteModal,
}: {
    quotation: QuotationResponse;
    bookSlug: string;
    activeNodeId: string | undefined;
    isExact: boolean;
    showLabel: boolean;
    onOpenNoteModal: (quotationId: string, note?: NoteResponse) => void;
}) {
    const queryClient = useQueryClient();
    const { data: notesData, isLoading } = useListNotes(
        bookSlug,
        quotation.id,
    );
    const notes = notesData?.data?.notes ?? [];

    const deleteNoteMutation = useDeleteNote({
        mutation: {
            onSuccess: () => {
                toast.success("Note deleted");
                queryClient.invalidateQueries({
                    queryKey: getListNotesQueryKey(bookSlug, quotation.id),
                });
                if (activeNodeId) {
                    queryClient.invalidateQueries({
                        queryKey: getListQuotationsQueryKey(bookSlug, {
                            node_id: activeNodeId,
                        }),
                    });
                }
            },
            onError: () => toast.error("Failed to delete note"),
        },
    });

    const handleDeleteNote = (note: NoteResponse) => {
        if (window.confirm("Delete this note?")) {
            deleteNoteMutation.mutate({
                slug: bookSlug,
                id: quotation.id,
                noteId: note.id,
            });
        }
    };

    if (isLoading) {
        return <p className="text-sm text-stone-400 p-2">Loading...</p>;
    }

    if (notes.length === 0 && !isExact) {
        return null;
    }

    return (
        <div>
            {showLabel && (
                <div className="flex items-center gap-1 mb-1 px-0.5">
                    <span className="text-[11px] font-medium text-stone-400">
                        {quotationLabel(quotation)}
                    </span>
                    {isExact && (
                        <span className="text-[10px] text-stone-300">
                            (exact)
                        </span>
                    )}
                </div>
            )}
            {notes.length === 0 && isExact && (
                <p className="text-sm text-stone-400 p-2">
                    No notes yet. Click + to add one.
                </p>
            )}
            <div className="space-y-1.5">
                {notes.map((note) => (
                    <NoteCard
                        key={note.id}
                        note={note}
                        sentenceLabel={quotationLabel(quotation)}
                        onEdit={() => onOpenNoteModal(quotation.id, note)}
                        onDelete={() => handleDeleteNote(note)}
                    />
                ))}
            </div>
        </div>
    );
}

function NoteCard({
    note,
    sentenceLabel,
    onEdit,
    onDelete,
}: {
    note: NoteResponse;
    sentenceLabel: string;
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
        <div className="border border-stone-300 rounded p-2.5 hover:shadow-md transition-all group">
            <p className="text-sm text-stone-700 whitespace-pre-wrap break-words">
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
            <div className="flex items-center mt-1.5">
                <span className="text-[10px] text-stone-400">
                    {sentenceLabel}
                </span>
                <span className="text-[10px] text-stone-400 mx-auto">
                    {dateStr}
                </span>
                <div className="opacity-0 group-hover:opacity-100 transition-opacity flex gap-0.5">
                    <IconButton
                        size="small"
                        onClick={onEdit}
                        title="Edit note"
                        sx={{ p: 0.25 }}
                    >
                        <EditOutlined sx={{ fontSize: 14 }} />
                    </IconButton>
                    <IconButton
                        size="small"
                        onClick={onDelete}
                        title="Delete note"
                        sx={{ p: 0.25, color: "rgb(168 162 158)" }}
                    >
                        <DeleteOutlined sx={{ fontSize: 14 }} />
                    </IconButton>
                </div>
            </div>
        </div>
    );
}
