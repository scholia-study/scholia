import { Button, Popover, TextField } from "@mui/material";
import { useState } from "react";
import type { SourceSearchResponse } from "../../../api/model";
import { useSearchSources } from "../../../api/sources/sources";
import { useDebouncedValue } from "../../../hooks/useDebouncedValue";
import { SourceFormModal } from "../../source";

export interface CitationEntry {
    sourceId: string;
    sourceLabel: string;
    pages: string;
    year?: string;
    authorLastName?: string;
}

interface CitationPopoverProps {
    anchorEl: HTMLElement | null;
    onClose: () => void;
    onConfirm: (entries: CitationEntry[]) => void;
    initialEntries?: CitationEntry[];
}

function CitationEntryRow({
    entry,
    onChange,
    onRemove,
    showRemove,
}: {
    entry: CitationEntry;
    onChange: (updated: CitationEntry) => void;
    onRemove: () => void;
    showRemove: boolean;
}) {
    const [search, setSearch] = useState(entry.sourceLabel);
    const debouncedSearch = useDebouncedValue(search);
    const { data: results } = useSearchSources(
        { q: debouncedSearch },
        { query: { enabled: debouncedSearch.length >= 3 && !entry.sourceId } },
    );
    const [showResults, setShowResults] = useState(false);

    const selectSource = (source: SourceSearchResponse) => {
        const personNames = source.persons?.map((p) => p.name).join(", ");
        const label = personNames
            ? `${personNames} — ${source.title}`
            : source.title;
        // Extract last name of first author for citation display
        const firstAuthor = source.persons?.[0]?.name ?? "";
        const lastNameParts = firstAuthor.split(/\s+/);
        const authorLastName =
            source.persons && source.persons.length > 2
                ? `${lastNameParts[lastNameParts.length - 1]} et al.`
                : source.persons && source.persons.length === 2
                  ? `${lastNameParts[lastNameParts.length - 1]} and ${source.persons[1].name.split(/\s+/).pop()}`
                  : lastNameParts[lastNameParts.length - 1] || "Unknown";
        const year = source.publication_year
            ? String(source.publication_year)
            : "n.d.";
        onChange({
            ...entry,
            sourceId: source.id,
            sourceLabel: label,
            year,
            authorLastName,
        });
        setSearch(label);
        setShowResults(false);
    };

    const clearSource = () => {
        onChange({ ...entry, sourceId: "", sourceLabel: "" });
        setSearch("");
    };

    return (
        <div className="flex gap-2 items-start">
            <div className="flex-1 relative">
                {entry.sourceId ? (
                    <div className="flex items-center gap-1 text-xs bg-stone-100 rounded px-2 py-1.5">
                        <span className="flex-1 truncate">
                            {entry.sourceLabel}
                        </span>
                        <button
                            type="button"
                            onClick={clearSource}
                            className="text-stone-400 hover:text-stone-600 shrink-0"
                        >
                            x
                        </button>
                    </div>
                ) : (
                    <>
                        <TextField
                            size="small"
                            placeholder="Search source..."
                            value={search}
                            onChange={(e) => {
                                setSearch(e.target.value);
                                setShowResults(true);
                            }}
                            onFocus={() => setShowResults(true)}
                            fullWidth
                            slotProps={{
                                input: { style: { fontSize: "0.75rem" } },
                            }}
                        />
                        {showResults &&
                            Array.isArray(results?.data) &&
                            results.data.length > 0 &&
                            search.length >= 3 && (
                                <ul className="absolute z-10 w-full border border-stone-200 rounded bg-white mt-0.5 max-h-32 overflow-y-auto shadow-sm">
                                    {results.data.map((s) => (
                                        <li key={s.id}>
                                            <button
                                                type="button"
                                                className="w-full text-left px-2 py-1 text-xs hover:bg-stone-100"
                                                onClick={() => selectSource(s)}
                                            >
                                                <span className="font-medium">
                                                    {s.title}
                                                </span>
                                                {s.publication_year && (
                                                    <span className="text-stone-400 ml-1">
                                                        ({s.publication_year})
                                                    </span>
                                                )}
                                                {s.persons &&
                                                    s.persons.length > 0 && (
                                                        <span className="text-stone-400 ml-1">
                                                            —{" "}
                                                            {s.persons
                                                                .map(
                                                                    (p) =>
                                                                        p.name,
                                                                )
                                                                .join(", ")}
                                                        </span>
                                                    )}
                                            </button>
                                        </li>
                                    ))}
                                </ul>
                            )}
                    </>
                )}
            </div>
            <TextField
                size="small"
                placeholder="Pages"
                value={entry.pages}
                onChange={(e) => onChange({ ...entry, pages: e.target.value })}
                sx={{ width: 80 }}
                slotProps={{
                    input: { style: { fontSize: "0.75rem" } },
                }}
            />
            {showRemove && (
                <button
                    type="button"
                    onClick={onRemove}
                    className="text-stone-400 hover:text-red-500 text-xs mt-1.5"
                >
                    x
                </button>
            )}
        </div>
    );
}

export function CitationPopover({
    anchorEl,
    onClose,
    onConfirm,
    initialEntries,
}: CitationPopoverProps) {
    const [entries, setEntries] = useState<CitationEntry[]>(
        initialEntries?.length
            ? initialEntries
            : [{ sourceId: "", sourceLabel: "", pages: "" }],
    );
    const [sourceModalOpen, setSourceModalOpen] = useState(false);

    const updateEntry = (idx: number, updated: CitationEntry) => {
        setEntries((prev) => prev.map((e, i) => (i === idx ? updated : e)));
    };

    const removeEntry = (idx: number) => {
        setEntries((prev) => prev.filter((_, i) => i !== idx));
    };

    const addEntry = () => {
        setEntries((prev) => [
            ...prev,
            { sourceId: "", sourceLabel: "", pages: "" },
        ]);
    };

    const handleConfirm = () => {
        const valid = entries.filter((e) => e.sourceId);
        if (valid.length > 0) {
            onConfirm(valid);
        }
    };

    const hasValidEntry = entries.some((e) => e.sourceId);

    return (
        <>
            <Popover
                open={!!anchorEl}
                anchorEl={anchorEl}
                onClose={onClose}
                anchorOrigin={{ vertical: "bottom", horizontal: "left" }}
                transformOrigin={{ vertical: "top", horizontal: "left" }}
                slotProps={{
                    paper: {
                        sx: {
                            p: 2,
                            width: 400,
                            maxWidth: "90vw",
                            overflow: "visible",
                        },
                    },
                }}
            >
                <div className="space-y-2">
                    <div className="text-xs font-medium text-stone-500 mb-1">
                        Citation
                    </div>
                    {entries.map((entry, idx) => (
                        <CitationEntryRow
                            key={`${idx}-${entry.sourceId}`}
                            entry={entry}
                            onChange={(updated) => updateEntry(idx, updated)}
                            onRemove={() => removeEntry(idx)}
                            showRemove={entries.length > 1}
                        />
                    ))}
                    <div className="flex justify-between items-center pt-1">
                        <div className="flex gap-2">
                            <button
                                type="button"
                                onClick={addEntry}
                                className="text-xs text-stone-500 hover:text-stone-700"
                            >
                                + Add source
                            </button>
                            <button
                                type="button"
                                onClick={() => setSourceModalOpen(true)}
                                className="text-xs text-blue-500 hover:text-blue-700"
                            >
                                Create new source
                            </button>
                        </div>
                        <div className="flex gap-1">
                            <Button
                                size="small"
                                onClick={onClose}
                                sx={{ fontSize: "0.7rem" }}
                            >
                                Cancel
                            </Button>
                            <Button
                                size="small"
                                variant="contained"
                                onClick={handleConfirm}
                                disabled={!hasValidEntry}
                                sx={{ fontSize: "0.7rem" }}
                            >
                                Insert
                            </Button>
                        </div>
                    </div>
                </div>
            </Popover>
            {sourceModalOpen && (
                <SourceFormModal
                    open={sourceModalOpen}
                    onClose={() => setSourceModalOpen(false)}
                    onCreated={(source) => {
                        setSourceModalOpen(false);
                        const emptyIdx = entries.findIndex((e) => !e.sourceId);
                        const personNames = source.persons
                            ?.map((p) => p.name)
                            .join(", ");
                        const label = personNames
                            ? `${personNames} — ${source.title}`
                            : source.title;
                        const firstAuthor = source.persons?.[0]?.name ?? "";
                        const lastNameParts = firstAuthor.split(/\s+/);
                        const authorLastName =
                            source.persons && source.persons.length > 2
                                ? `${lastNameParts[lastNameParts.length - 1]} et al.`
                                : source.persons && source.persons.length === 2
                                  ? `${lastNameParts[lastNameParts.length - 1]} and ${source.persons[1].name.split(/\s+/).pop()}`
                                  : lastNameParts[lastNameParts.length - 1] ||
                                    "Unknown";
                        const year = source.publication_year
                            ? String(source.publication_year)
                            : "n.d.";
                        const newEntry: CitationEntry = {
                            sourceId: source.id,
                            sourceLabel: label,
                            pages: "",
                            year,
                            authorLastName,
                        };
                        if (emptyIdx >= 0) {
                            updateEntry(emptyIdx, newEntry);
                        } else {
                            setEntries((prev) => [...prev, newEntry]);
                        }
                    }}
                />
            )}
        </>
    );
}
