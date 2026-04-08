import {
    Button,
    Checkbox,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    FormControl,
    FormControlLabel,
    InputLabel,
    MenuItem,
    Select,
    TextField,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useDebouncedValue } from "../hooks/useDebouncedValue";
import toast from "react-hot-toast";
import type { ResourceResponse, SourceResponse } from "../api/model";
import {
    getListResourcesQueryKey,
    useCreateResource,
    useUpdateResource,
} from "../api/resources/resources";
import { useSearchSources } from "../api/sources/sources";
import { SourceFormModal } from "./SourceFormModal";

const RESOURCE_TYPES = ["verbatim", "paraphrase", "allusion"] as const;
const VERBATIM_KINDS = ["entirety", "fragmentary"] as const;

interface ResourceFormModalProps {
    open: boolean;
    onClose: () => void;
    bookSlug: string;
    mode: "create" | "edit";
    initialData?: ResourceResponse;
    defaultType?: "verbatim" | "paraphrase" | "allusion";
    defaultSentenceStart?: number;
    defaultSentenceEnd?: number;
    defaultSentenceKind?: string;
    isAdmin?: boolean;
}

export function ResourceFormModal({
    open,
    onClose,
    bookSlug,
    mode,
    initialData,
    defaultType,
    defaultSentenceStart,
    defaultSentenceEnd,
    defaultSentenceKind,
    isAdmin,
}: ResourceFormModalProps) {
    const isEdit = mode === "edit" && initialData;

    // Form state
    const [resourceType, setResourceType] = useState(
        isEdit ? initialData.resource_type : (defaultType ?? "verbatim"),
    );
    const [verbatimKind, setVerbatimKind] = useState(
        isEdit ? (initialData.verbatim_kind ?? "entirety") : "entirety",
    );
    const [sentenceStart, setSentenceStart] = useState(
        String(isEdit ? initialData.anchor_sentence_start_number : (defaultSentenceStart ?? "")),
    );
    const [sentenceEnd, setSentenceEnd] = useState(
        isEdit && initialData.anchor_sentence_end_number != null
            ? String(initialData.anchor_sentence_end_number)
            : defaultSentenceEnd != null ? String(defaultSentenceEnd) : "",
    );
    const [sentenceKind, setSentenceKind] = useState(
        isEdit ? initialData.sentence_kind : (defaultSentenceKind ?? "body"),
    );

    // Source
    const [sourceId, setSourceId] = useState(isEdit ? (initialData.source?.id ?? "") : "");
    const [sourceLabel, setSourceLabel] = useState(
        isEdit ? (initialData.source?.title ?? "") : "",
    );
    const [sourceSearch, setSourceSearch] = useState("");
    const debouncedSourceSearch = useDebouncedValue(sourceSearch);
    const { data: sourceResults } = useSearchSources(
        { q: debouncedSourceSearch },
        { query: { enabled: debouncedSourceSearch.length >= 3 } },
    );

    // Source location
    const [sourcePageStart, setSourcePageStart] = useState(
        isEdit && initialData.source_page_start != null
            ? String(initialData.source_page_start)
            : "",
    );
    const [sourcePageEnd, setSourcePageEnd] = useState(
        isEdit && initialData.source_page_end != null
            ? String(initialData.source_page_end)
            : "",
    );
    const [sourceLocationFreeform, setSourceLocationFreeform] = useState(
        isEdit ? (initialData.source_location_freeform ?? "") : "",
    );
    const [useFreeformLocation, setUseFreeformLocation] = useState(
        isEdit ? !!initialData.source_location_freeform : false,
    );

    // Content
    const [quotedText, setQuotedText] = useState(isEdit ? (initialData.quoted_text ?? "") : "");
    const [editorNote, setEditorNote] = useState(isEdit ? (initialData.editor_note ?? "") : "");
    const [isFeatured, setIsFeatured] = useState(isEdit ? initialData.is_featured : false);
    const [adminNotes, setAdminNotes] = useState(isEdit ? (initialData.admin_notes ?? "") : "");

    // Source creation modal
    const [sourceModalOpen, setSourceModalOpen] = useState(false);

    const queryClient = useQueryClient();

    const createMutation = useCreateResource({
        mutation: {
            onSuccess: () => {
                toast.success("Resource created");
                invalidateAndClose();
            },
            onError: () => toast.error("Failed to create resource"),
        },
    });

    const updateMutation = useUpdateResource({
        mutation: {
            onSuccess: () => {
                toast.success("Resource updated");
                invalidateAndClose();
            },
            onError: () => toast.error("Failed to update resource"),
        },
    });

    const invalidateAndClose = () => {
        const start = Number.parseInt(sentenceStart, 10);
        const end = sentenceEnd ? Number.parseInt(sentenceEnd, 10) : start;
        if (!Number.isNaN(start)) {
            queryClient.invalidateQueries({
                queryKey: getListResourcesQueryKey(bookSlug, {
                    start,
                    end,
                    kind: sentenceKind,
                }),
            });
        }
        onClose();
    };

    const handleSubmit = () => {
        const start = Number.parseInt(sentenceStart, 10);
        if (Number.isNaN(start)) {
            toast.error("Sentence start is required");
            return;
        }

        const end = sentenceEnd ? Number.parseInt(sentenceEnd, 10) : undefined;
        if (end != null && end - start + 1 > 15) {
            toast.error("Range cannot exceed 15 sentences");
            return;
        }

        const payload = {
            resource_type: resourceType,
            verbatim_kind: resourceType === "verbatim" ? verbatimKind : undefined,
            sentence_start: start,
            sentence_end: end,
            sentence_kind: sentenceKind,
            source_id: sourceId || undefined,
            source_page_start: !useFreeformLocation && sourcePageStart
                ? Number.parseInt(sourcePageStart, 10)
                : undefined,
            source_page_end: !useFreeformLocation && sourcePageEnd
                ? Number.parseInt(sourcePageEnd, 10)
                : undefined,
            source_location_freeform: useFreeformLocation && sourceLocationFreeform.trim()
                ? sourceLocationFreeform.trim()
                : undefined,
            quoted_text: quotedText.trim() || undefined,
            editor_note: editorNote.trim() || undefined,
            is_featured: isFeatured,
            admin_notes: adminNotes.trim() || undefined,
        };

        if (isEdit) {
            updateMutation.mutate({
                slug: bookSlug,
                id: initialData.id,
                data: payload,
            });
        } else {
            createMutation.mutate({
                slug: bookSlug,
                data: payload,
            });
        }
    };

    const handleSourceCreated = (source: SourceResponse) => {
        setSourceId(source.id);
        setSourceLabel(source.title);
        setSourceSearch("");
        setSourceModalOpen(false);
    };

    return (
        <>
            <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
                <DialogTitle sx={{ fontSize: 16 }}>
                    {isEdit ? "Edit Resource" : "New Resource"}
                </DialogTitle>
                <DialogContent
                    sx={{
                        display: "flex",
                        flexDirection: "column",
                        gap: 2,
                        pt: "8px !important",
                    }}
                >
                    {/* Type */}
                    <div className="flex gap-2">
                        <FormControl size="small" sx={{ flex: 1 }}>
                            <InputLabel>Type</InputLabel>
                            <Select
                                value={resourceType}
                                onChange={(e) => setResourceType(e.target.value)}
                                label="Type"
                            >
                                {RESOURCE_TYPES.map((t) => (
                                    <MenuItem key={t} value={t}>
                                        {t.charAt(0).toUpperCase() + t.slice(1)}
                                    </MenuItem>
                                ))}
                            </Select>
                        </FormControl>

                        {resourceType === "verbatim" && (
                            <FormControl size="small" sx={{ flex: 1 }}>
                                <InputLabel>Kind</InputLabel>
                                <Select
                                    value={verbatimKind}
                                    onChange={(e) =>
                                        setVerbatimKind(e.target.value)
                                    }
                                    label="Kind"
                                >
                                    {VERBATIM_KINDS.map((k) => (
                                        <MenuItem key={k} value={k}>
                                            {k.charAt(0).toUpperCase() + k.slice(1)}
                                        </MenuItem>
                                    ))}
                                </Select>
                            </FormControl>
                        )}
                    </div>

                    {/* Sentence anchor */}
                    <div>
                        <div className="flex gap-2">
                            <TextField
                                label="Sentence Start"
                                value={sentenceStart}
                                onChange={(e) => setSentenceStart(e.target.value)}
                                size="small"
                                type="number"
                                required
                                sx={{ flex: 1 }}
                            />
                            <TextField
                                label="Sentence End"
                                value={sentenceEnd}
                                onChange={(e) => setSentenceEnd(e.target.value)}
                                size="small"
                                type="number"
                                placeholder="Same as start"
                                sx={{ flex: 1 }}
                            />
                            <FormControl size="small" sx={{ minWidth: 100 }}>
                                <InputLabel>Kind</InputLabel>
                                <Select
                                    value={sentenceKind}
                                    onChange={(e) =>
                                        setSentenceKind(e.target.value)
                                    }
                                    label="Kind"
                                >
                                    <MenuItem value="body">Body</MenuItem>
                                    <MenuItem value="footnote">Footnote</MenuItem>
                                </Select>
                            </FormControl>
                        </div>
                        <p className="text-xs text-stone-400 mt-1">
                            {sentenceEnd
                                ? `Range: sentences ${sentenceStart}\u2013${sentenceEnd} (${sentenceKind})`
                                : `Single sentence: ${sentenceStart} (${sentenceKind})`}
                        </p>
                    </div>

                    {/* Source selector */}
                    <div className="border-t border-stone-200 pt-2">
                        <div className="text-sm text-stone-600 mb-1.5 font-medium">
                            Source
                        </div>
                        {sourceId ? (
                            <div className="flex items-center justify-between px-2 py-1.5 bg-stone-50 rounded text-xs">
                                <span className="text-stone-800">
                                    {sourceLabel}
                                </span>
                                <button
                                    type="button"
                                    onClick={() => {
                                        setSourceId("");
                                        setSourceLabel("");
                                    }}
                                    className="text-stone-400 hover:text-stone-600 ml-2"
                                >
                                    &times;
                                </button>
                            </div>
                        ) : (
                            <div className="relative">
                                <div className="flex gap-2">
                                    <TextField
                                        label="Search source"
                                        value={sourceSearch}
                                        onChange={(e) =>
                                            setSourceSearch(e.target.value)
                                        }
                                        size="small"
                                        fullWidth
                                    />
                                    <Button
                                        size="small"
                                        variant="outlined"
                                        onClick={() =>
                                            setSourceModalOpen(true)
                                        }
                                        sx={{ whiteSpace: "nowrap" }}
                                    >
                                        New
                                    </Button>
                                </div>
                                {Array.isArray(sourceResults?.data) &&
                                    sourceResults.data.length > 0 &&
                                    sourceSearch.length >= 3 && (
                                        <ul className="absolute z-10 w-full border border-stone-200 rounded bg-white mt-0.5 max-h-32 overflow-y-auto shadow-sm">
                                            {sourceResults.data.map((s) => (
                                                <li key={s.id}>
                                                    <button
                                                        type="button"
                                                        onClick={() => {
                                                            setSourceId(s.id);
                                                            setSourceLabel(
                                                                s.title,
                                                            );
                                                            setSourceSearch("");
                                                        }}
                                                        className="w-full text-left px-2 py-1 text-xs hover:bg-stone-50"
                                                    >
                                                        {s.title}
                                                        {s.publication_year
                                                            ? ` (${s.publication_year})`
                                                            : ""}
                                                        {s.persons.length > 0
                                                            ? ` \u2014 ${s.persons.map((p) => p.name).join(", ")}`
                                                            : ""}
                                                    </button>
                                                </li>
                                            ))}
                                        </ul>
                                    )}
                            </div>
                        )}
                    </div>

                    {/* Source location */}
                    {sourceId && (
                        <div>
                            <FormControlLabel
                                control={
                                    <Checkbox
                                        size="small"
                                        checked={useFreeformLocation}
                                        onChange={(e) =>
                                            setUseFreeformLocation(
                                                e.target.checked,
                                            )
                                        }
                                    />
                                }
                                label="Freeform location"
                                slotProps={{ typography: { fontSize: 12 } }}
                            />
                            {useFreeformLocation ? (
                                <TextField
                                    label="Location"
                                    value={sourceLocationFreeform}
                                    onChange={(e) =>
                                        setSourceLocationFreeform(
                                            e.target.value,
                                        )
                                    }
                                    size="small"
                                    fullWidth
                                    placeholder='e.g. "ch. 3", "lines 200-210"'
                                />
                            ) : (
                                <div className="flex gap-2">
                                    <TextField
                                        label="Page Start"
                                        value={sourcePageStart}
                                        onChange={(e) =>
                                            setSourcePageStart(e.target.value)
                                        }
                                        size="small"
                                        type="number"
                                        sx={{ flex: 1 }}
                                    />
                                    <TextField
                                        label="Page End"
                                        value={sourcePageEnd}
                                        onChange={(e) =>
                                            setSourcePageEnd(e.target.value)
                                        }
                                        size="small"
                                        type="number"
                                        sx={{ flex: 1 }}
                                    />
                                </div>
                            )}
                        </div>
                    )}

                    {/* Content fields */}
                    <div className="border-t border-stone-200 pt-2">
                        {resourceType === "verbatim" && (
                            <TextField
                                label="Quoted Text"
                                value={quotedText}
                                onChange={(e) => setQuotedText(e.target.value)}
                                size="small"
                                multiline
                                rows={3}
                                fullWidth
                                sx={{ mb: 2 }}
                            />
                        )}

                        {resourceType === "paraphrase" && (
                            <TextField
                                label="Paraphrased Content"
                                value={quotedText}
                                onChange={(e) => setQuotedText(e.target.value)}
                                size="small"
                                multiline
                                rows={3}
                                fullWidth
                                sx={{ mb: 2 }}
                            />
                        )}

                        <TextField
                            label="Editor Note"
                            value={editorNote}
                            onChange={(e) => setEditorNote(e.target.value)}
                            size="small"
                            multiline
                            rows={2}
                            fullWidth
                        />
                    </div>

                    <div className="flex items-center gap-4">
                        <FormControlLabel
                            control={
                                <Checkbox
                                    size="small"
                                    checked={isFeatured}
                                    onChange={(e) =>
                                        setIsFeatured(e.target.checked)
                                    }
                                />
                            }
                            label="Featured"
                            slotProps={{ typography: { fontSize: 12 } }}
                        />
                    </div>

                    {isAdmin && (
                        <TextField
                            label="Admin Notes"
                            value={adminNotes}
                            onChange={(e) => setAdminNotes(e.target.value)}
                            size="small"
                            multiline
                            rows={2}
                            fullWidth
                        />
                    )}
                </DialogContent>
                <DialogActions>
                    <Button onClick={onClose} size="small">
                        Cancel
                    </Button>
                    <Button
                        onClick={handleSubmit}
                        variant="contained"
                        size="small"
                        disabled={
                            createMutation.isPending ||
                            updateMutation.isPending
                        }
                    >
                        {isEdit ? "Update" : "Create"}
                    </Button>
                </DialogActions>
            </Dialog>

            <SourceFormModal
                open={sourceModalOpen}
                onClose={() => setSourceModalOpen(false)}
                onCreated={handleSourceCreated}
            />
        </>
    );
}
