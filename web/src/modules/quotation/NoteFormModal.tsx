import {
    Autocomplete,
    Button,
    Chip,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    TextField,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import toast from "react-hot-toast";
import type { NoteResponse } from "#/api/model";
import { FetchError } from "../../api/fetcher";
import {
    getListNotesQueryKey,
    useCreateNote,
    useUpdateNote,
} from "../../api/quotations/quotations";
import { useListTags } from "../../api/tags/tags";
import { invalidateAllNodeQuotations } from "./hooks/invalidateQuotations";

interface NoteFormModalProps {
    open: boolean;
    onClose: () => void;
    bookSlug: string;
    quotationId: string;
    mode: "create" | "edit";
    initialData?: NoteResponse;
    sentenceContext?: string;
}

export function NoteFormModal({
    open,
    onClose,
    bookSlug,
    quotationId,
    mode,
    initialData,
    sentenceContext,
}: NoteFormModalProps) {
    const queryClient = useQueryClient();

    const [body, setBody] = useState(initialData?.body ?? "");
    const [selectedTags, setSelectedTags] = useState<string[]>(
        initialData?.tags.map((t) => t.name) ?? [],
    );
    const [tagInput, setTagInput] = useState("");

    const { data: tagsData } = useListTags();
    const existingTags = tagsData?.data?.tags?.map((t) => t.name) ?? [];

    const createNote = useCreateNote({
        mutation: {
            onSuccess: () => {
                toast.success("Note created");
                queryClient.invalidateQueries({
                    queryKey: getListNotesQueryKey(bookSlug, quotationId),
                });
                invalidateAllNodeQuotations(queryClient);
                onClose();
            },
            onError: (err: unknown) => {
                const message =
                    err instanceof FetchError && err.message
                        ? err.message
                        : "Failed to create note";
                toast.error(message);
            },
        },
    });

    const updateNote = useUpdateNote({
        mutation: {
            onSuccess: () => {
                toast.success("Note updated");
                queryClient.invalidateQueries({
                    queryKey: getListNotesQueryKey(bookSlug, quotationId),
                });
                onClose();
            },
            onError: (err: unknown) => {
                const message =
                    err instanceof FetchError && err.message
                        ? err.message
                        : "Failed to update note";
                toast.error(message);
            },
        },
    });

    const handleSubmit = () => {
        if (!body.trim()) return;

        if (mode === "create") {
            createNote.mutate({
                slug: bookSlug,
                id: quotationId,
                data: { body: body.trim(), tags: selectedTags },
            });
        } else if (initialData) {
            updateNote.mutate({
                slug: bookSlug,
                id: quotationId,
                noteId: initialData.id,
                data: { body: body.trim(), tags: selectedTags },
            });
        }
    };

    const isPending = createNote.isPending || updateNote.isPending;

    return (
        <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
            <DialogTitle sx={{ fontSize: "0.95rem", pb: 0.5 }}>
                {mode === "create" ? "Add Note" : "Edit Note"}
            </DialogTitle>
            <DialogContent>
                {sentenceContext && (
                    <div className="text-xs text-stone-400 mb-3 mt-1 italic border-l-2 border-stone-200 pl-2">
                        {sentenceContext}
                    </div>
                )}
                <TextField
                    autoFocus
                    multiline
                    minRows={4}
                    maxRows={12}
                    fullWidth
                    placeholder="Write your note..."
                    value={body}
                    onChange={(e) => setBody(e.target.value)}
                    variant="outlined"
                    size="small"
                    sx={{ mt: 1 }}
                />
                <Autocomplete
                    multiple
                    freeSolo
                    options={existingTags.filter(
                        (t) => !selectedTags.includes(t),
                    )}
                    value={selectedTags}
                    onChange={(_e, newValue) => setSelectedTags(newValue)}
                    inputValue={tagInput}
                    onInputChange={(_e, newInputValue) =>
                        setTagInput(newInputValue)
                    }
                    renderTags={(value, getTagProps) =>
                        value.map((tag, index) => {
                            const { key, ...rest } = getTagProps({ index });
                            return (
                                <Chip
                                    key={key}
                                    label={tag}
                                    size="small"
                                    variant="outlined"
                                    {...rest}
                                />
                            );
                        })
                    }
                    renderInput={(params) => (
                        <TextField
                            {...params}
                            variant="outlined"
                            size="small"
                            placeholder="Add tags (press Enter to add)..."
                            sx={{ mt: 1.5 }}
                        />
                    )}
                    sx={{ mt: 0 }}
                />
            </DialogContent>
            <DialogActions sx={{ px: 3, pb: 2 }}>
                <Button onClick={onClose} size="small">
                    Cancel
                </Button>
                <Button
                    onClick={handleSubmit}
                    variant="contained"
                    size="small"
                    disabled={!body.trim() || isPending}
                >
                    {mode === "create" ? "Save" : "Update"}
                </Button>
            </DialogActions>
        </Dialog>
    );
}
