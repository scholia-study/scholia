import {
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    FormControl,
    InputLabel,
    MenuItem,
    Select,
    TextField,
} from "@mui/material";
import { useState } from "react";
import { useDebouncedValue } from "../hooks/useDebouncedValue";
import toast from "react-hot-toast";
import type {
    PersonResponse,
    SourcePersonResponse,
    SourceResponse,
} from "../api/model";
import { useSearchPersons } from "../api/persons/persons";
import {
    useAddSourcePerson,
    useCreateSource,
    useSearchSources,
} from "../api/sources/sources";
import { PersonFormModal } from "./PersonFormModal";

interface SourceFormModalProps {
    open: boolean;
    onClose: () => void;
    onCreated: (source: SourceResponse) => void;
}

const SOURCE_TYPES = ["book", "article", "chapter", "journal", "web"] as const;
const PERSON_ROLES = [
    "author",
    "editor",
    "translator",
    "contributor",
] as const;

const REQUIRED_BY_TYPE: Record<string, readonly string[]> = {
    book: ["title", "publication_year", "publisher", "author"],
    chapter: ["title", "parent_source_id", "page_start"],
    article: ["title", "journal_name", "publication_year", "page_start"],
    journal: ["title", "publication_year", "publisher"],
    web: ["title", "url"],
};

export function SourceFormModal({
    open,
    onClose,
    onCreated,
}: SourceFormModalProps) {
    // Source fields
    const [sourceType, setSourceType] = useState<string>("book");
    const [title, setTitle] = useState("");
    const [publicationYear, setPublicationYear] = useState("");
    const [publisher, setPublisher] = useState("");
    const [isbn, setIsbn] = useState("");
    const [doi, setDoi] = useState("");
    const [edition, setEdition] = useState("");
    const [volume, setVolume] = useState("");
    const [journalName, setJournalName] = useState("");
    const [url, setUrl] = useState("");
    const [pageStart, setPageStart] = useState("");
    const [pageEnd, setPageEnd] = useState("");
    const [parentSourceId, setParentSourceId] = useState("");

    // Parent source search
    const [parentSearch, setParentSearch] = useState("");
    const debouncedParentSearch = useDebouncedValue(parentSearch);
    const { data: parentResults } = useSearchSources(
        { q: debouncedParentSearch },
        { query: { enabled: debouncedParentSearch.length >= 3 } },
    );

    // Person management
    const [persons, setPersons] = useState<SourcePersonResponse[]>([]);
    const [personSearch, setPersonSearch] = useState("");
    const debouncedPersonSearch = useDebouncedValue(personSearch);
    const [addPersonRole, setAddPersonRole] = useState<string>("author");
    const { data: personResults } = useSearchPersons(
        { q: debouncedPersonSearch },
        { query: { enabled: debouncedPersonSearch.length >= 3 } },
    );

    // Person creation modal
    const [personModalOpen, setPersonModalOpen] = useState(false);

    const createMutation = useCreateSource({
        mutation: {
            onSuccess: (response) => {
                if (!response?.data) return;
                const source = response.data;
                // If we have persons to link, link them now
                if (persons.length > 0) {
                    linkPersonsSequentially(source.id, persons, 0, source);
                } else {
                    toast.success(`Source "${source.title}" created`);
                    onCreated(source);
                    resetAndClose();
                }
            },
            onError: () => {
                toast.error("Failed to create source");
            },
        },
    });

    const addPersonMutation = useAddSourcePerson();

    const linkPersonsSequentially = (
        sourceId: string,
        personsToLink: SourcePersonResponse[],
        index: number,
        source: SourceResponse,
    ) => {
        if (index >= personsToLink.length) {
            toast.success(`Source "${source.title}" created`);
            // Return source with persons attached
            onCreated({ ...source, persons: personsToLink });
            resetAndClose();
            return;
        }
        const p = personsToLink[index];
        addPersonMutation.mutate(
            {
                id: sourceId,
                data: {
                    person_id: p.person_id,
                    role: p.role,
                    position: p.position,
                },
            },
            {
                onSuccess: () => {
                    linkPersonsSequentially(
                        sourceId,
                        personsToLink,
                        index + 1,
                        source,
                    );
                },
                onError: () => {
                    toast.error(`Failed to link person ${p.name}`);
                },
            },
        );
    };

    const resetAndClose = () => {
        setSourceType("book");
        setTitle("");
        setPublicationYear("");
        setPublisher("");
        setIsbn("");
        setDoi("");
        setEdition("");
        setVolume("");
        setJournalName("");
        setUrl("");
        setPageStart("");
        setPageEnd("");
        setParentSourceId("");
        setParentSearch("");
        setPersons([]);
        setPersonSearch("");
        onClose();
    };

    const isRequired = (field: string) =>
        REQUIRED_BY_TYPE[sourceType]?.includes(field) ?? false;

    const handleSubmit = () => {
        if (!title.trim()) {
            toast.error("Title is required");
            return;
        }

        const yearNum = publicationYear
            ? Number.parseInt(publicationYear, 10)
            : undefined;
        const hasValidYear = yearNum !== undefined && !Number.isNaN(yearNum);

        if (isRequired("publication_year") && !hasValidYear) {
            toast.error("Year is required");
            return;
        }
        if (isRequired("publisher") && !publisher.trim()) {
            toast.error("Publisher is required");
            return;
        }
        if (isRequired("journal_name") && !journalName.trim()) {
            toast.error("Journal name is required");
            return;
        }
        if (isRequired("url") && !url.trim()) {
            toast.error("URL is required");
            return;
        }
        if (isRequired("page_start") && !pageStart.trim()) {
            toast.error("Page start is required");
            return;
        }
        if (isRequired("parent_source_id") && !parentSourceId) {
            toast.error("Parent book is required");
            return;
        }
        if (
            isRequired("author") &&
            !persons.some((p) => p.role === "author")
        ) {
            toast.error("At least one author is required");
            return;
        }

        const isbnArr = isbn.trim()
            ? isbn.split(",").map((s) => s.trim())
            : undefined;

        createMutation.mutate({
            data: {
                source_type: sourceType,
                title: title.trim(),
                publication_year: yearNum && !Number.isNaN(yearNum) ? yearNum : undefined,
                publisher: publisher.trim() || undefined,
                isbn: isbnArr,
                doi: doi.trim() || undefined,
                edition: edition.trim() || undefined,
                volume: volume.trim() || undefined,
                journal_name: journalName.trim() || undefined,
                url: url.trim() || undefined,
                page_start: pageStart ? Number.parseInt(pageStart, 10) : undefined,
                page_end: pageEnd ? Number.parseInt(pageEnd, 10) : undefined,
                parent_source_id: parentSourceId || undefined,
            },
        });
    };

    const addPerson = (person: PersonResponse) => {
        const exists = persons.some(
            (p) => p.person_id === person.id && p.role === addPersonRole,
        );
        if (exists) return;

        setPersons([
            ...persons,
            {
                person_id: person.id,
                name: person.name,
                sort_name: person.sort_name,
                role: addPersonRole,
                position: persons.length,
            },
        ]);
        setPersonSearch("");
    };

    const removePerson = (personId: string, role: string) => {
        setPersons(
            persons.filter(
                (p) => !(p.person_id === personId && p.role === role),
            ),
        );
    };

    const handlePersonCreated = (person: PersonResponse) => {
        addPerson(person);
        setPersonModalOpen(false);
    };

    return (
        <>
            <Dialog
                open={open}
                onClose={resetAndClose}
                maxWidth="sm"
                fullWidth
            >
                <DialogTitle sx={{ fontSize: 16 }}>New Source</DialogTitle>
                <DialogContent
                    sx={{
                        display: "flex",
                        flexDirection: "column",
                        gap: 2,
                        pt: "8px !important",
                    }}
                >
                    <FormControl size="small">
                        <InputLabel>Type</InputLabel>
                        <Select
                            value={sourceType}
                            onChange={(e) => setSourceType(e.target.value)}
                            label="Type"
                        >
                            {SOURCE_TYPES.map((t) => (
                                <MenuItem key={t} value={t}>
                                    {t.charAt(0).toUpperCase() + t.slice(1)}
                                </MenuItem>
                            ))}
                        </Select>
                    </FormControl>

                    <TextField
                        label="Title"
                        value={title}
                        onChange={(e) => setTitle(e.target.value)}
                        size="small"
                        required
                    />

                    <div className="flex gap-2">
                        <TextField
                            label="Year"
                            value={publicationYear}
                            onChange={(e) => setPublicationYear(e.target.value)}
                            size="small"
                            type="number"
                            sx={{ flex: 1 }}
                            required={isRequired("publication_year")}
                        />
                        <TextField
                            label="Publisher"
                            value={publisher}
                            onChange={(e) => setPublisher(e.target.value)}
                            size="small"
                            sx={{ flex: 2 }}
                            required={isRequired("publisher")}
                        />
                    </div>

                    <div className="flex gap-2">
                        <TextField
                            label="Edition"
                            value={edition}
                            onChange={(e) => setEdition(e.target.value)}
                            size="small"
                            sx={{ flex: 1 }}
                        />
                        <TextField
                            label="Volume"
                            value={volume}
                            onChange={(e) => setVolume(e.target.value)}
                            size="small"
                            sx={{ flex: 1 }}
                        />
                    </div>

                    {(sourceType === "article" || sourceType === "journal") && (
                        <TextField
                            label="Journal Name"
                            value={journalName}
                            onChange={(e) => setJournalName(e.target.value)}
                            size="small"
                            required={isRequired("journal_name")}
                        />
                    )}

                    <TextField
                        label="ISBN (comma-separated)"
                        value={isbn}
                        onChange={(e) => setIsbn(e.target.value)}
                        size="small"
                    />

                    <div className="flex gap-2">
                        <TextField
                            label="DOI"
                            value={doi}
                            onChange={(e) => setDoi(e.target.value)}
                            size="small"
                            sx={{ flex: 1 }}
                        />
                        <TextField
                            label="URL"
                            value={url}
                            onChange={(e) => setUrl(e.target.value)}
                            size="small"
                            sx={{ flex: 1 }}
                            required={isRequired("url")}
                        />
                    </div>

                    {(sourceType === "article" || sourceType === "chapter") && (
                        <div className="flex gap-2">
                            <TextField
                                label="Page Start"
                                value={pageStart}
                                onChange={(e) => setPageStart(e.target.value)}
                                size="small"
                                type="number"
                                sx={{ flex: 1 }}
                                required={isRequired("page_start")}
                            />
                            <TextField
                                label="Page End"
                                value={pageEnd}
                                onChange={(e) => setPageEnd(e.target.value)}
                                size="small"
                                type="number"
                                sx={{ flex: 1 }}
                            />
                        </div>
                    )}

                    {sourceType === "chapter" && (
                        <div>
                            <TextField
                                label="Parent Book (search)"
                                value={parentSearch}
                                onChange={(e) => {
                                    setParentSearch(e.target.value);
                                    setParentSourceId("");
                                }}
                                size="small"
                                fullWidth
                                required={isRequired("parent_source_id")}
                                helperText={
                                    parentSourceId
                                        ? "Parent selected"
                                        : "Type to search for the parent book"
                                }
                            />
                            {Array.isArray(parentResults?.data) &&
                                parentResults.data.length > 0 &&
                                !parentSourceId && (
                                    <ul className="border border-stone-200 rounded mt-1 max-h-32 overflow-y-auto">
                                        {parentResults.data.map((s) => (
                                            <li key={s.id}>
                                                <button
                                                    type="button"
                                                    onClick={() => {
                                                        setParentSourceId(s.id);
                                                        setParentSearch(
                                                            s.title,
                                                        );
                                                    }}
                                                    className="w-full text-left px-2 py-1 text-xs hover:bg-stone-50"
                                                >
                                                    {s.title}
                                                    {s.publication_year
                                                        ? ` (${s.publication_year})`
                                                        : ""}
                                                </button>
                                            </li>
                                        ))}
                                    </ul>
                                )}
                        </div>
                    )}

                    {/* Persons section */}
                    <div className="border-t border-stone-200 pt-2 mt-1">
                        <div className="text-sm text-stone-600 mb-2 font-medium">
                            Contributors
                            {isRequired("author") && (
                                <span className="text-red-500 ml-0.5">*</span>
                            )}
                        </div>
                        {isRequired("author") &&
                            !persons.some((p) => p.role === "author") && (
                                <div className="text-xs text-stone-500 mb-2">
                                    At least one author is required.
                                </div>
                            )}
                        {persons.length > 0 && (
                            <ul className="space-y-1 mb-2">
                                {persons.map((p) => (
                                    <li
                                        key={`${p.person_id}-${p.role}`}
                                        className="flex items-center justify-between text-xs px-2 py-1 bg-stone-50 rounded"
                                    >
                                        <span>
                                            {p.name}{" "}
                                            <span className="text-stone-400">
                                                ({p.role})
                                            </span>
                                        </span>
                                        <button
                                            type="button"
                                            onClick={() =>
                                                removePerson(
                                                    p.person_id,
                                                    p.role,
                                                )
                                            }
                                            className="text-red-400 hover:text-red-600 ml-2"
                                        >
                                            &times;
                                        </button>
                                    </li>
                                ))}
                            </ul>
                        )}
                        <div className="flex gap-2 items-end">
                            <div className="flex-1 relative">
                                <TextField
                                    label="Search person"
                                    value={personSearch}
                                    onChange={(e) =>
                                        setPersonSearch(e.target.value)
                                    }
                                    size="small"
                                    fullWidth
                                />
                                {Array.isArray(personResults?.data) &&
                                    personResults.data.length > 0 &&
                                    personSearch.length >= 3 && (
                                        <ul className="absolute bottom-full left-0 z-10 w-full border border-stone-200 rounded bg-white mb-0.5 max-h-32 overflow-y-auto shadow-sm">
                                            {personResults.data.map((p) => (
                                                <li key={p.id}>
                                                    <button
                                                        type="button"
                                                        onClick={() =>
                                                            addPerson(p)
                                                        }
                                                        className="w-full text-left px-2 py-1 text-xs hover:bg-stone-50"
                                                    >
                                                        {p.name}
                                                        {p.sort_name
                                                            ? ` (${p.sort_name})`
                                                            : ""}
                                                    </button>
                                                </li>
                                            ))}
                                        </ul>
                                    )}
                            </div>
                            <FormControl size="small" sx={{ minWidth: 100 }}>
                                <InputLabel>Role</InputLabel>
                                <Select
                                    value={addPersonRole}
                                    onChange={(e) =>
                                        setAddPersonRole(e.target.value)
                                    }
                                    label="Role"
                                >
                                    {PERSON_ROLES.map((r) => (
                                        <MenuItem key={r} value={r}>
                                            {r}
                                        </MenuItem>
                                    ))}
                                </Select>
                            </FormControl>
                            <Button
                                size="small"
                                variant="outlined"
                                onClick={() => setPersonModalOpen(true)}
                                sx={{ whiteSpace: "nowrap" }}
                            >
                                New
                            </Button>
                        </div>
                    </div>
                </DialogContent>
                <DialogActions>
                    <Button onClick={resetAndClose} size="small">
                        Cancel
                    </Button>
                    <Button
                        onClick={handleSubmit}
                        variant="contained"
                        size="small"
                        disabled={createMutation.isPending}
                    >
                        Create
                    </Button>
                </DialogActions>
            </Dialog>

            <PersonFormModal
                open={personModalOpen}
                onClose={() => setPersonModalOpen(false)}
                onCreated={handlePersonCreated}
            />
        </>
    );
}
