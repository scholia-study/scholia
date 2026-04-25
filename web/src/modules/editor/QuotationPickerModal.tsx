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
    TextField,
} from "@mui/material";
import { useMemo, useState } from "react";
import type { UnifiedQuotationResponse } from "../../api/model";
import { useListAllQuotations } from "../../api/quotations/quotations";

export type QuotationPickerResult =
    | {
          source_type: "book";
          book: string;
          node: string;
          start: number;
          end?: number;
          kind: string;
          mode: "source" | "translation" | "source+translation";
          layout:
              | "stacked"
              | "side-by-side-source-left"
              | "side-by-side-source-right";
      }
    | {
          source_type: "article";
          id: string;
      };

type BookDisplayMode = "source" | "translation" | "source+translation";
type BookLayout =
    | "stacked"
    | "side-by-side-source-left"
    | "side-by-side-source-right";

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
    const [search, setSearch] = useState<string>("");
    const [selected, setSelected] = useState<UnifiedQuotationResponse | null>(
        null,
    );
    const [mode, setMode] = useState<BookDisplayMode>("translation");
    const [layout, setLayout] = useState<BookLayout>("stacked");

    const { data: quotationsData, isLoading } = useListAllQuotations(
        {},
        { query: { enabled: open } },
    );
    const allQuotations = useMemo(
        () => quotationsData?.data?.quotations ?? [],
        [quotationsData],
    );

    const availableBooks = useMemo(() => {
        const map = new Map<string, string>();
        for (const q of allQuotations) {
            if (q.source_type === "book" && !map.has(q.book_slug)) {
                map.set(q.book_slug, q.book_title);
            }
        }
        return [...map.entries()].sort((a, b) => a[1].localeCompare(b[1]));
    }, [allQuotations]);

    const filtered = useMemo(() => {
        const q = search.trim().toLowerCase();
        return allQuotations.filter((item) => {
            if (bookFilter) {
                if (item.source_type !== "book") return false;
                if (item.book_slug !== bookFilter) return false;
            }
            if (!q) return true;
            if (item.source_type === "book") {
                const hay = [
                    item.book_title,
                    item.node_label,
                    item.start_text_snippet ?? "",
                    item.end_text_snippet ?? "",
                ]
                    .join(" ")
                    .toLowerCase();
                return hay.includes(q);
            }
            const hay = [
                item.article_title,
                item.author_display_name,
                item.text_snippet,
            ]
                .join(" ")
                .toLowerCase();
            return hay.includes(q);
        });
    }, [allQuotations, bookFilter, search]);

    const handleConfirm = () => {
        if (!selected) return;
        if (selected.source_type === "book") {
            onSelect({
                source_type: "book",
                book: selected.book_slug,
                node: selected.node_slug,
                start: selected.anchor_sentence_start_number,
                end: selected.anchor_sentence_end_number ?? undefined,
                kind: selected.sentence_kind,
                mode,
                layout,
            });
        } else {
            onSelect({ source_type: "article", id: selected.id });
        }
        handleClose();
    };

    const handleClose = () => {
        setSelected(null);
        setSearch("");
        setMode("translation");
        setLayout("stacked");
        onClose();
    };

    return (
        <Dialog open={open} onClose={handleClose} maxWidth="md" fullWidth>
            <DialogTitle>Insert Quotation</DialogTitle>
            <DialogContent>
                <div className="flex items-center gap-3 mb-4 mt-1">
                    <TextField
                        size="small"
                        placeholder="Search quotations..."
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                        sx={{ flex: 1 }}
                    />
                    <FormControl size="small" sx={{ minWidth: 200 }}>
                        <InputLabel>Filter by book</InputLabel>
                        <Select
                            value={bookFilter}
                            label="Filter by book"
                            onChange={(e) => setBookFilter(e.target.value)}
                        >
                            <MenuItem value="">All sources</MenuItem>
                            {availableBooks.map(([slug, title]) => (
                                <MenuItem key={slug} value={slug}>
                                    {title}
                                </MenuItem>
                            ))}
                        </Select>
                    </FormControl>
                </div>

                {isLoading && (
                    <p className="text-sm text-stone-400">
                        Loading quotations...
                    </p>
                )}

                {!isLoading && filtered.length === 0 && (
                    <p className="text-sm text-stone-400">
                        No saved quotations match.
                    </p>
                )}

                <div className="space-y-1.5 max-h-64 overflow-y-auto mb-4">
                    {filtered.map((q) =>
                        q.source_type === "book" ? (
                            <QuotationRow
                                key={`book-${q.id}`}
                                isSelected={
                                    selected?.source_type === "book" &&
                                    selected.id === q.id
                                }
                                onClick={() => setSelected(q)}
                                badge="Book"
                                badgeColor="rgb(168 162 158)"
                                header={`${q.book_title} · ${q.node_label} · ${
                                    q.anchor_sentence_end_number &&
                                    q.anchor_sentence_end_number !==
                                        q.anchor_sentence_start_number
                                        ? `Sentences ${q.anchor_sentence_start_number}\u2013${q.anchor_sentence_end_number}`
                                        : `Sentence ${q.anchor_sentence_start_number}`
                                }`}
                                snippet={q.start_text_snippet ?? ""}
                            />
                        ) : (
                            <QuotationRow
                                key={`article-${q.id}`}
                                isSelected={
                                    selected?.source_type === "article" &&
                                    selected.id === q.id
                                }
                                onClick={() => setSelected(q)}
                                badge="Article"
                                badgeColor="rgb(180 83 9)"
                                header={`${q.article_title} · ${q.author_display_name}`}
                                snippet={q.text_snippet}
                            />
                        ),
                    )}
                </div>

                {selected?.source_type === "book" && (
                    <div className="border-t border-stone-200 pt-4 flex gap-6">
                        <FormControl>
                            <FormLabel sx={{ fontSize: "0.75rem" }}>
                                Display mode
                            </FormLabel>
                            <RadioGroup
                                value={mode}
                                onChange={(e) =>
                                    setMode(e.target.value as BookDisplayMode)
                                }
                                row
                            >
                                <FormControlLabel
                                    value="source"
                                    control={<Radio size="small" />}
                                    label="Source"
                                    slotProps={{
                                        typography: { fontSize: "0.8rem" },
                                    }}
                                />
                                <FormControlLabel
                                    value="translation"
                                    control={<Radio size="small" />}
                                    label="Translation"
                                    slotProps={{
                                        typography: { fontSize: "0.8rem" },
                                    }}
                                />
                                <FormControlLabel
                                    value="source+translation"
                                    control={<Radio size="small" />}
                                    label="Both"
                                    slotProps={{
                                        typography: { fontSize: "0.8rem" },
                                    }}
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
                                    setLayout(e.target.value as BookLayout)
                                }
                                row
                            >
                                <FormControlLabel
                                    value="stacked"
                                    control={<Radio size="small" />}
                                    label="Stacked"
                                    slotProps={{
                                        typography: { fontSize: "0.8rem" },
                                    }}
                                />
                                <FormControlLabel
                                    value="side-by-side-source-left"
                                    control={<Radio size="small" />}
                                    label="Side-by-side (source left)"
                                    slotProps={{
                                        typography: { fontSize: "0.8rem" },
                                    }}
                                />
                                <FormControlLabel
                                    value="side-by-side-source-right"
                                    control={<Radio size="small" />}
                                    label="Side-by-side (source right)"
                                    slotProps={{
                                        typography: { fontSize: "0.8rem" },
                                    }}
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

interface QuotationRowProps {
    isSelected: boolean;
    onClick: () => void;
    badge: string;
    badgeColor: string;
    header: string;
    snippet: string;
}

function QuotationRow({
    isSelected,
    onClick,
    badge,
    badgeColor,
    header,
    snippet,
}: QuotationRowProps) {
    return (
        <Paper
            elevation={0}
            onClick={onClick}
            sx={{
                p: 1.5,
                cursor: "pointer",
                border: isSelected
                    ? "2px solid rgb(59 130 246)"
                    : "1px solid rgb(214 211 209)",
                transition: "border-color 0.15s",
                "&:hover": {
                    borderColor: "rgb(168 162 158)",
                },
            }}
        >
            <div className="flex items-center gap-2 mb-0.5">
                <span
                    className="text-[10px] uppercase tracking-wide px-1.5 py-0.5 rounded text-white"
                    style={{ backgroundColor: badgeColor }}
                >
                    {badge}
                </span>
                <span className="text-xs text-stone-400 truncate">
                    {header}
                </span>
            </div>
            {snippet && (
                <p className="text-sm text-stone-600 truncate">
                    &ldquo;{snippet}&rdquo;
                </p>
            )}
        </Paper>
    );
}
