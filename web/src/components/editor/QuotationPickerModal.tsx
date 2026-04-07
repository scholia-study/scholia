import {
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    FormControl,
    FormControlLabel,
    FormLabel,
    InputLabel,
    MenuItem,
    Paper,
    Radio,
    RadioGroup,
    Select,
} from "@mui/material";
import { useMemo, useState } from "react";
import type { QuotationWithContextResponse } from "../../api/model";
import { useListAllQuotations } from "../../api/quotations/quotations";

export interface QuotationPickerResult {
    book: string;
    node: string;
    start: number;
    end?: number;
    kind: string;
    mode: "source" | "translation" | "source+translation";
    layout: "stacked" | "side-by-side-source-left" | "side-by-side-source-right";
}

interface QuotationPickerModalProps {
    open: boolean;
    onClose: () => void;
    onSelect: (result: QuotationPickerResult) => void;
}

export function QuotationPickerModal({
    open,
    onClose,
    onSelect,
}: QuotationPickerModalProps) {
    const [bookFilter, setBookFilter] = useState<string>("");
    const [selected, setSelected] = useState<QuotationWithContextResponse | null>(null);
    const [mode, setMode] = useState<QuotationPickerResult["mode"]>("translation");
    const [layout, setLayout] = useState<QuotationPickerResult["layout"]>("stacked");

    const { data: quotationsData, isLoading } = useListAllQuotations(
        {},
        { query: { enabled: open } },
    );
    const allQuotations = quotationsData?.data?.quotations ?? [];

    const availableBooks = useMemo(() => {
        const map = new Map<string, string>();
        for (const q of allQuotations) {
            if (!map.has(q.book_slug)) {
                map.set(q.book_slug, q.book_title);
            }
        }
        return [...map.entries()].sort((a, b) => a[1].localeCompare(b[1]));
    }, [allQuotations]);

    const filtered = useMemo(() => {
        if (!bookFilter) return allQuotations;
        return allQuotations.filter((q) => q.book_slug === bookFilter);
    }, [allQuotations, bookFilter]);

    const handleConfirm = () => {
        if (!selected) return;
        onSelect({
            book: selected.book_slug,
            node: selected.node_slug,
            start: selected.anchor_sentence_start_number,
            end: selected.anchor_sentence_end_number ?? undefined,
            kind: selected.sentence_kind,
            mode,
            layout,
        });
        handleClose();
    };

    const handleClose = () => {
        setSelected(null);
        setMode("translation");
        setLayout("stacked");
        onClose();
    };

    return (
        <Dialog open={open} onClose={handleClose} maxWidth="md" fullWidth>
            <DialogTitle>Insert Quotation</DialogTitle>
            <DialogContent>
                <div className="flex items-center gap-3 mb-4 mt-1">
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
                    <p className="text-sm text-stone-400">Loading quotations...</p>
                )}

                {!isLoading && filtered.length === 0 && (
                    <p className="text-sm text-stone-400">
                        No saved quotations. Save quotations from the reader first.
                    </p>
                )}

                <div className="space-y-1.5 max-h-64 overflow-y-auto mb-4">
                    {filtered.map((q) => (
                        <Paper
                            key={q.id}
                            elevation={0}
                            onClick={() => setSelected(q)}
                            sx={{
                                p: 1.5,
                                cursor: "pointer",
                                border:
                                    selected?.id === q.id
                                        ? "2px solid rgb(59 130 246)"
                                        : "1px solid rgb(214 211 209)",
                                transition: "border-color 0.15s",
                                "&:hover": {
                                    borderColor: "rgb(168 162 158)",
                                },
                            }}
                        >
                            <div className="text-xs text-stone-400 mb-0.5">
                                {q.book_title} &middot; {q.node_label} &middot;{" "}
                                {q.anchor_sentence_end_number &&
                                q.anchor_sentence_end_number !== q.anchor_sentence_start_number
                                    ? `Sentences ${q.anchor_sentence_start_number}\u2013${q.anchor_sentence_end_number}`
                                    : `Sentence ${q.anchor_sentence_start_number}`}
                            </div>
                            {q.start_text_snippet && (
                                <p className="text-sm text-stone-600 truncate">
                                    &ldquo;{q.start_text_snippet}&rdquo;
                                </p>
                            )}
                        </Paper>
                    ))}
                </div>

                {selected && (
                    <div className="border-t border-stone-200 pt-4 flex gap-6">
                        <FormControl>
                            <FormLabel sx={{ fontSize: "0.75rem" }}>
                                Display mode
                            </FormLabel>
                            <RadioGroup
                                value={mode}
                                onChange={(e) =>
                                    setMode(
                                        e.target.value as QuotationPickerResult["mode"],
                                    )
                                }
                                row
                            >
                                <FormControlLabel
                                    value="source"
                                    control={<Radio size="small" />}
                                    label="Source"
                                    slotProps={{ typography: { fontSize: "0.8rem" } }}
                                />
                                <FormControlLabel
                                    value="translation"
                                    control={<Radio size="small" />}
                                    label="Translation"
                                    slotProps={{ typography: { fontSize: "0.8rem" } }}
                                />
                                <FormControlLabel
                                    value="source+translation"
                                    control={<Radio size="small" />}
                                    label="Both"
                                    slotProps={{ typography: { fontSize: "0.8rem" } }}
                                />
                            </RadioGroup>
                        </FormControl>

                        <FormControl>
                            <FormLabel sx={{ fontSize: "0.75rem" }}>
                                Layout
                            </FormLabel>
                            <RadioGroup
                                value={layout}
                                onChange={(e) =>
                                    setLayout(
                                        e.target.value as QuotationPickerResult["layout"],
                                    )
                                }
                                row
                            >
                                <FormControlLabel
                                    value="stacked"
                                    control={<Radio size="small" />}
                                    label="Stacked"
                                    slotProps={{ typography: { fontSize: "0.8rem" } }}
                                />
                                <FormControlLabel
                                    value="side-by-side-source-left"
                                    control={<Radio size="small" />}
                                    label="Side-by-side (source left)"
                                    slotProps={{ typography: { fontSize: "0.8rem" } }}
                                />
                                <FormControlLabel
                                    value="side-by-side-source-right"
                                    control={<Radio size="small" />}
                                    label="Side-by-side (source right)"
                                    slotProps={{ typography: { fontSize: "0.8rem" } }}
                                />
                            </RadioGroup>
                        </FormControl>
                    </div>
                )}
            </DialogContent>
            <DialogActions>
                <Button onClick={handleClose}>Cancel</Button>
                <Button
                    onClick={handleConfirm}
                    variant="contained"
                    disabled={!selected}
                >
                    Insert
                </Button>
            </DialogActions>
        </Dialog>
    );
}
