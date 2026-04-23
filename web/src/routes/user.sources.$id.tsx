import ArrowBackOutlined from "@mui/icons-material/ArrowBackOutlined";
import DeleteOutlined from "@mui/icons-material/DeleteOutlined";
import LockOutlined from "@mui/icons-material/LockOutlined";
import SaveOutlined from "@mui/icons-material/SaveOutlined";
import {
    Button,
    Checkbox,
    Chip,
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
import {
    Link,
    createFileRoute,
    redirect,
    useNavigate,
} from "@tanstack/react-router";
import { useEffect, useState } from "react";
import toast from "react-hot-toast";
import { getGetProfileQueryOptions } from "../api/auth/auth";
import { FetchError } from "../api/fetcher";
import type {
    PersonResponse,
    ReferenceCheckResponse,
    SourcePersonResponse,
    SourceResponse,
} from "../api/model";
import {
    useSearchPersons,
    useUpdatePerson,
} from "../api/persons/persons";
import {
    getCheckSourceReferencesQueryKey,
    getBrowseSourcesQueryKey,
    getGetSourceQueryKey,
    useAddSourcePerson,
    useCheckSourceReferences,
    useDeleteSource,
    useGetSource,
    useRemoveSourcePerson,
    useUpdateSource,
} from "../api/sources/sources";
import { PersonFormModal } from "../components/PersonFormModal";
import { useAuth } from "../hooks/useAuth";
import { useDebouncedValue } from "../hooks/useDebouncedValue";

export const Route = createFileRoute("/user/sources/$id")({
    beforeLoad: async ({ context }) => {
        const data = await context.queryClient.fetchQuery(
            getGetProfileQueryOptions(),
        );
        if (!data?.data) {
            throw redirect({ to: "/login" });
        }
    },
    component: SourceDetailPage,
});

const PERSON_ROLES = ["author", "editor", "translator", "contributor"] as const;

function SourceDetailPage() {
    const { id } = Route.useParams();
    const navigate = useNavigate();
    const queryClient = useQueryClient();
    const { user, hasPermission } = useAuth();
    const isEditor = hasPermission("resources_manage");

    const { data: sourceData, isLoading } = useGetSource(id);
    const source = sourceData?.data;
    const { data: refsData } = useCheckSourceReferences(id);
    const refsRaw = refsData?.data;
    const refs: ReferenceCheckResponse | undefined =
        refsRaw && typeof refsRaw === "object" && "total" in refsRaw
            ? refsRaw
            : undefined;

    if (isLoading || !source) {
        return (
            <div className="max-w-3xl mx-auto px-8 py-16">
                <p className="text-sm text-stone-400">Loading...</p>
            </div>
        );
    }

    const isOwner = !!user && source.created_by === user.id;
    const canEdit = isEditor || (isOwner && !source.protected);

    return (
        <DetailContent
            source={source}
            refs={refs}
            canEdit={canEdit}
            isEditor={isEditor}
            isOwner={isOwner}
            onDeleted={() => {
                queryClient.invalidateQueries({
                    queryKey: getBrowseSourcesQueryKey(),
                });
                navigate({ to: "/user/sources" });
            }}
            onChanged={() => {
                queryClient.invalidateQueries({
                    queryKey: getGetSourceQueryKey(id),
                });
                queryClient.invalidateQueries({
                    queryKey: getCheckSourceReferencesQueryKey(id),
                });
                queryClient.invalidateQueries({
                    queryKey: getBrowseSourcesQueryKey(),
                });
            }}
        />
    );
}

function DetailContent({
    source,
    refs,
    canEdit,
    isEditor,
    isOwner,
    onDeleted,
    onChanged,
}: {
    source: SourceResponse;
    refs: ReferenceCheckResponse | undefined;
    canEdit: boolean;
    isEditor: boolean;
    isOwner: boolean;
    onDeleted: () => void;
    onChanged: () => void;
}) {
    const [title, setTitle] = useState(source.title);
    const [titleDisplay, setTitleDisplay] = useState(source.title_display ?? "");
    const [publicationYear, setPublicationYear] = useState(
        source.publication_year != null ? String(source.publication_year) : "",
    );
    const [publisher, setPublisher] = useState(source.publisher ?? "");
    const [isbn, setIsbn] = useState((source.isbn ?? []).join(", "));
    const [doi, setDoi] = useState(source.doi ?? "");
    const [edition, setEdition] = useState(source.edition ?? "");
    const [volume, setVolume] = useState(source.volume ?? "");
    const [journalName, setJournalName] = useState(source.journal_name ?? "");
    const [url, setUrl] = useState(source.url ?? "");
    const [pageStart, setPageStart] = useState(
        source.page_start != null ? String(source.page_start) : "",
    );
    const [pageEnd, setPageEnd] = useState(
        source.page_end != null ? String(source.page_end) : "",
    );
    const [protectedFlag, setProtectedFlag] = useState(source.protected);

    useEffect(() => {
        setTitle(source.title);
        setTitleDisplay(source.title_display ?? "");
        setPublicationYear(
            source.publication_year != null ? String(source.publication_year) : "",
        );
        setPublisher(source.publisher ?? "");
        setIsbn((source.isbn ?? []).join(", "));
        setDoi(source.doi ?? "");
        setEdition(source.edition ?? "");
        setVolume(source.volume ?? "");
        setJournalName(source.journal_name ?? "");
        setUrl(source.url ?? "");
        setPageStart(source.page_start != null ? String(source.page_start) : "");
        setPageEnd(source.page_end != null ? String(source.page_end) : "");
        setProtectedFlag(source.protected);
    }, [source]);

    const updateMutation = useUpdateSource();
    const deleteMutation = useDeleteSource();

    const handleSave = async () => {
        const yearNum = publicationYear
            ? Number.parseInt(publicationYear, 10)
            : undefined;
        const isbnArr = isbn.trim()
            ? isbn.split(",").map((s) => s.trim()).filter(Boolean)
            : undefined;

        try {
            await updateMutation.mutateAsync({
                id: source.id,
                data: {
                    title: title.trim() || undefined,
                    title_display: titleDisplay.trim() || undefined,
                    publication_year:
                        yearNum != null && !Number.isNaN(yearNum) ? yearNum : undefined,
                    publisher: publisher.trim() || undefined,
                    isbn: isbnArr,
                    doi: doi.trim() || undefined,
                    edition: edition.trim() || undefined,
                    volume: volume.trim() || undefined,
                    journal_name: journalName.trim() || undefined,
                    url: url.trim() || undefined,
                    page_start: pageStart
                        ? Number.parseInt(pageStart, 10)
                        : undefined,
                    page_end: pageEnd ? Number.parseInt(pageEnd, 10) : undefined,
                    protected: isEditor ? protectedFlag : undefined,
                },
            });
            toast.success("Source saved");
            onChanged();
        } catch (err) {
            toast.error(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to save source",
            );
        }
    };

    const handleDelete = async () => {
        if (!confirm("Delete this source? This cannot be undone.")) return;
        try {
            await deleteMutation.mutateAsync({ id: source.id });
            toast.success("Source deleted");
            onDeleted();
        } catch (err) {
            toast.error(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to delete source",
            );
        }
    };

    const refTotal = refs?.total ?? 0;
    const deleteBlocked = refTotal > 0;
    const deleteTooltip = deleteBlocked
        ? `Cannot delete: ${refTotal} reference(s)`
        : "";

    const typeConditional = (types: string[]) => types.includes(source.source_type);

    return (
        <div className="max-w-3xl mx-auto px-8 py-16">
            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <Link
                    to="/user/sources"
                    className="text-sm text-stone-500 hover:text-stone-900 inline-flex items-center gap-1"
                >
                    <ArrowBackOutlined sx={{ fontSize: 16 }} />
                    Sources
                </Link>
                <div className="flex items-center gap-2">
                    <Chip
                        label={source.source_type}
                        size="small"
                        sx={{ fontSize: "0.65rem", height: 20 }}
                    />
                    {source.protected && (
                        <Chip
                            icon={<LockOutlined sx={{ fontSize: 12 }} />}
                            label="Protected"
                            size="small"
                            color="warning"
                            sx={{ fontSize: "0.65rem", height: 20 }}
                        />
                    )}
                    <Button
                        size="small"
                        variant="outlined"
                        color="error"
                        startIcon={<DeleteOutlined />}
                        onClick={handleDelete}
                        disabled={!canEdit || deleteBlocked || deleteMutation.isPending}
                        title={deleteTooltip}
                        sx={{ textTransform: "none" }}
                    >
                        Delete
                    </Button>
                    <Button
                        size="small"
                        variant="contained"
                        startIcon={<SaveOutlined />}
                        onClick={handleSave}
                        disabled={!canEdit || updateMutation.isPending}
                        sx={{ textTransform: "none" }}
                    >
                        Save
                    </Button>
                </div>
            </div>

            {!canEdit && (
                <div className="mb-4 px-3 py-2 bg-amber-50 border border-amber-200 rounded text-sm text-amber-800">
                    {source.protected
                        ? "This source is curated by editors and cannot be edited."
                        : "You can only edit sources you created. This is read-only."}
                </div>
            )}

            {/* Protected toggle (editor only) */}
            {isEditor && (
                <FormControlLabel
                    control={
                        <Checkbox
                            checked={protectedFlag}
                            onChange={(_, v) => setProtectedFlag(v)}
                            size="small"
                        />
                    }
                    label={<span className="text-sm">Protected (editor-curated)</span>}
                    sx={{ mb: 2 }}
                />
            )}

            {/* Metadata form */}
            <div className="flex flex-col gap-3 mb-8">
                <TextField
                    label="Title"
                    value={title}
                    onChange={(e) => setTitle(e.target.value)}
                    size="small"
                    disabled={!canEdit}
                    required
                />
                <TextField
                    label="Display Title"
                    value={titleDisplay}
                    onChange={(e) => setTitleDisplay(e.target.value)}
                    size="small"
                    disabled={!canEdit}
                />
                <div className="flex gap-2">
                    <TextField
                        label="Year"
                        value={publicationYear}
                        onChange={(e) => setPublicationYear(e.target.value)}
                        size="small"
                        type="number"
                        disabled={!canEdit}
                        sx={{ flex: 1 }}
                    />
                    <TextField
                        label="Publisher"
                        value={publisher}
                        onChange={(e) => setPublisher(e.target.value)}
                        size="small"
                        disabled={!canEdit}
                        sx={{ flex: 2 }}
                    />
                </div>
                <div className="flex gap-2">
                    <TextField
                        label="Edition"
                        value={edition}
                        onChange={(e) => setEdition(e.target.value)}
                        size="small"
                        disabled={!canEdit}
                        sx={{ flex: 1 }}
                    />
                    <TextField
                        label="Volume"
                        value={volume}
                        onChange={(e) => setVolume(e.target.value)}
                        size="small"
                        disabled={!canEdit}
                        sx={{ flex: 1 }}
                    />
                </div>
                {typeConditional(["article", "journal"]) && (
                    <TextField
                        label="Journal Name"
                        value={journalName}
                        onChange={(e) => setJournalName(e.target.value)}
                        size="small"
                        disabled={!canEdit}
                    />
                )}
                <TextField
                    label="ISBN (comma-separated)"
                    value={isbn}
                    onChange={(e) => setIsbn(e.target.value)}
                    size="small"
                    disabled={!canEdit}
                />
                <div className="flex gap-2">
                    <TextField
                        label="DOI"
                        value={doi}
                        onChange={(e) => setDoi(e.target.value)}
                        size="small"
                        disabled={!canEdit}
                        sx={{ flex: 1 }}
                    />
                    <TextField
                        label="URL"
                        value={url}
                        onChange={(e) => setUrl(e.target.value)}
                        size="small"
                        disabled={!canEdit}
                        sx={{ flex: 1 }}
                    />
                </div>
                {typeConditional(["article", "chapter"]) && (
                    <div className="flex gap-2">
                        <TextField
                            label="Page Start"
                            value={pageStart}
                            onChange={(e) => setPageStart(e.target.value)}
                            size="small"
                            type="number"
                            disabled={!canEdit}
                            sx={{ flex: 1 }}
                        />
                        <TextField
                            label="Page End"
                            value={pageEnd}
                            onChange={(e) => setPageEnd(e.target.value)}
                            size="small"
                            type="number"
                            disabled={!canEdit}
                            sx={{ flex: 1 }}
                        />
                    </div>
                )}
            </div>

            {/* Contributors */}
            <ContributorsBlock
                source={source}
                canEdit={canEdit}
                isEditor={isEditor}
                onChanged={onChanged}
            />

            {/* References panel */}
            <ReferencesPanel refs={refs} />

            <div className="text-xs text-stone-400 mt-4">
                {isOwner ? "Created by you." : null}
            </div>
        </div>
    );
}

function ContributorsBlock({
    source,
    canEdit,
    isEditor,
    onChanged,
}: {
    source: SourceResponse;
    canEdit: boolean;
    isEditor: boolean;
    onChanged: () => void;
}) {
    const { user: currentUser } = useAuth();
    const myId = currentUser?.id;

    const [search, setSearch] = useState("");
    const debouncedSearch = useDebouncedValue(search);
    const [role, setRole] = useState<string>("author");
    const { data: personResults } = useSearchPersons(
        { q: debouncedSearch },
        { query: { enabled: debouncedSearch.length >= 3 && canEdit } },
    );

    const addMutation = useAddSourcePerson();
    const removeMutation = useRemoveSourcePerson();
    const [newPersonModalOpen, setNewPersonModalOpen] = useState(false);
    const [editingPerson, setEditingPerson] =
        useState<SourcePersonResponse | null>(null);

    const linkPerson = async (person: { id: string; name: string }) => {
        const exists = source.persons.some(
            (p) => p.person_id === person.id && p.role === role,
        );
        if (exists) {
            toast.error("Already linked with that role");
            return;
        }
        try {
            await addMutation.mutateAsync({
                id: source.id,
                data: {
                    person_id: person.id,
                    role,
                    position: source.persons.length,
                },
            });
            setSearch("");
            onChanged();
        } catch (err) {
            toast.error(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to link person",
            );
        }
    };

    const unlinkPerson = async (p: SourcePersonResponse) => {
        try {
            await removeMutation.mutateAsync({
                id: source.id,
                personId: p.person_id,
                role: p.role,
            });
            onChanged();
        } catch (err) {
            toast.error(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to remove person",
            );
        }
    };

    return (
        <div className="border-t border-stone-200 pt-4 mt-4 mb-8">
            <div className="text-sm font-medium text-stone-700 mb-2">
                Contributors
            </div>
            {source.persons.length > 0 && (
                <ul className="space-y-1 mb-2">
                    {source.persons.map((p) => {
                        const canEditPerson =
                            canEdit &&
                            !p.protected &&
                            (isEditor || p.created_by === myId);
                        return (
                            <li
                                key={`${p.person_id}-${p.role}`}
                                className="flex items-center justify-between text-xs px-2 py-1 bg-stone-50 rounded"
                            >
                                <span>
                                    {canEditPerson ? (
                                        <button
                                            type="button"
                                            className="underline decoration-dotted hover:text-amber-800"
                                            onClick={() => setEditingPerson(p)}
                                        >
                                            {p.name}
                                        </button>
                                    ) : (
                                        <span>{p.name}</span>
                                    )}{" "}
                                    <span className="text-stone-400">({p.role})</span>
                                </span>
                                {canEdit && (
                                    <button
                                        type="button"
                                        onClick={() => unlinkPerson(p)}
                                        aria-label={`Remove ${p.name}`}
                                        className="text-red-400 hover:text-red-600 ml-2 text-lg leading-none px-1"
                                    >
                                        &times;
                                    </button>
                                )}
                            </li>
                        );
                    })}
                </ul>
            )}

            {canEdit && (
                <div className="flex gap-2 items-end">
                    <div className="flex-1 relative">
                        <TextField
                            label="Search person"
                            value={search}
                            onChange={(e) => setSearch(e.target.value)}
                            size="small"
                            fullWidth
                        />
                        {Array.isArray(personResults?.data) &&
                            personResults.data.length > 0 &&
                            debouncedSearch.length >= 3 && (
                                <ul className="absolute z-10 w-full border border-stone-200 rounded bg-white mt-0.5 max-h-32 overflow-y-auto shadow-sm">
                                    {personResults.data.map((p) => (
                                        <li key={p.id}>
                                            <button
                                                type="button"
                                                onClick={() => linkPerson(p)}
                                                className="w-full text-left px-2 py-1 text-xs hover:bg-stone-50"
                                            >
                                                {p.name}
                                                {p.sort_name ? ` (${p.sort_name})` : ""}
                                            </button>
                                        </li>
                                    ))}
                                </ul>
                            )}
                    </div>
                    <FormControl size="small" sx={{ minWidth: 100 }}>
                        <InputLabel>Role</InputLabel>
                        <Select
                            value={role}
                            onChange={(e) => setRole(e.target.value)}
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
                        onClick={() => setNewPersonModalOpen(true)}
                        sx={{ whiteSpace: "nowrap" }}
                    >
                        New
                    </Button>
                </div>
            )}

            <PersonFormModal
                open={newPersonModalOpen}
                onClose={() => setNewPersonModalOpen(false)}
                onCreated={(person) => {
                    setNewPersonModalOpen(false);
                    linkPerson({ id: person.id, name: person.name });
                }}
            />

            <PersonEditModal
                person={editingPerson}
                onClose={() => setEditingPerson(null)}
                onSaved={() => {
                    setEditingPerson(null);
                    onChanged();
                }}
            />
        </div>
    );
}

function PersonEditModal({
    person,
    onClose,
    onSaved,
}: {
    person: SourcePersonResponse | null;
    onClose: () => void;
    onSaved: (updated: PersonResponse) => void;
}) {
    const [name, setName] = useState("");
    const [sortName, setSortName] = useState("");
    const updateMutation = useUpdatePerson();

    useEffect(() => {
        if (person) {
            setName(person.name);
            setSortName(person.sort_name ?? "");
        }
    }, [person]);

    if (!person) return null;

    const handleSave = async () => {
        try {
            const result = await updateMutation.mutateAsync({
                id: person.person_id,
                data: {
                    name: name.trim() || undefined,
                    sort_name: sortName.trim() || undefined,
                },
            });
            if (result.data) {
                toast.success(`Updated ${result.data.name}`);
                onSaved(result.data);
            }
        } catch (err) {
            toast.error(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to update person",
            );
        }
    };

    return (
        <Dialog open={!!person} onClose={onClose} maxWidth="xs" fullWidth>
            <DialogTitle sx={{ fontSize: 16 }}>Edit Person</DialogTitle>
            <DialogContent
                sx={{
                    display: "flex",
                    flexDirection: "column",
                    gap: 2,
                    pt: "8px !important",
                }}
            >
                <TextField
                    label="Name"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    size="small"
                    autoFocus
                    required
                />
                <TextField
                    label="Sort Name"
                    value={sortName}
                    onChange={(e) => setSortName(e.target.value)}
                    size="small"
                />
            </DialogContent>
            <DialogActions>
                <Button onClick={onClose} size="small">
                    Cancel
                </Button>
                <Button
                    onClick={handleSave}
                    variant="contained"
                    size="small"
                    disabled={updateMutation.isPending}
                >
                    Save
                </Button>
            </DialogActions>
        </Dialog>
    );
}

function ReferencesPanel({ refs }: { refs: ReferenceCheckResponse | undefined }) {
    const total = refs?.total ?? 0;
    return (
        <div className="border-t border-stone-200 pt-4 mt-4">
            <div className="text-sm font-medium text-stone-700 mb-2">
                References
            </div>
            {total === 0 ? (
                <div className="text-xs text-stone-400">
                    This source has no references. It can be deleted.
                </div>
            ) : (
                <div className="text-xs text-stone-600 space-y-2">
                    <div>
                        Referenced in {refs?.resources.count ?? 0} resource(s),
                        {" "}
                        {refs?.child_sources.count ?? 0} child source(s),
                        {" "}
                        {refs?.articles.count ?? 0} article(s).
                    </div>
                    {refs?.child_sources.items && refs.child_sources.items.length > 0 && (
                        <div>
                            <div className="font-medium text-stone-500 mb-1">
                                Child sources
                            </div>
                            <ul className="space-y-0.5">
                                {refs.child_sources.items.map((c) => (
                                    <li key={c.id}>
                                        <Link
                                            to="/user/sources/$id"
                                            params={{ id: c.id }}
                                            className="text-amber-700 hover:underline"
                                        >
                                            {c.title}
                                        </Link>
                                        <span className="text-stone-400"> ({c.relation})</span>
                                    </li>
                                ))}
                            </ul>
                        </div>
                    )}
                    {refs?.articles.items && refs.articles.items.length > 0 && (
                        <div>
                            <div className="font-medium text-stone-500 mb-1">
                                Articles
                            </div>
                            <ul className="space-y-0.5">
                                {refs.articles.items.map((a) => {
                                    const linkable =
                                        a.status === "published" || a.is_mine;
                                    return (
                                        <li key={a.id}>
                                            {linkable && a.is_mine ? (
                                                <Link
                                                    to="/user/articles/$slug"
                                                    params={{ slug: a.slug }}
                                                    className="text-amber-700 hover:underline"
                                                >
                                                    {a.title}
                                                </Link>
                                            ) : linkable ? (
                                                <Link
                                                    to="/articles/$slug"
                                                    params={{ slug: a.slug }}
                                                    className="text-amber-700 hover:underline"
                                                >
                                                    {a.title}
                                                </Link>
                                            ) : (
                                                <span className="text-stone-400">
                                                    Private article
                                                </span>
                                            )}
                                        </li>
                                    );
                                })}
                            </ul>
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}
