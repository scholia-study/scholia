import AddOutlined from "@mui/icons-material/AddOutlined";
import ArrowDownwardOutlined from "@mui/icons-material/ArrowDownwardOutlined";
import ArrowUpwardOutlined from "@mui/icons-material/ArrowUpwardOutlined";
import CloseOutlined from "@mui/icons-material/CloseOutlined";
import OpenInNewOutlined from "@mui/icons-material/OpenInNewOutlined";
import {
    Button,
    Chip,
    Drawer,
    FormControlLabel,
    IconButton,
    MenuItem,
    Switch,
    TextField,
    Tooltip,
} from "@mui/material";
import { DataGrid, type GridColDef } from "@mui/x-data-grid";
import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute, notFound } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import toast from "react-hot-toast";
import {
    getAdminListSeriesQueryKey,
    getListSeriesMembersQueryKey,
    useAddSeriesArticle,
    useAdminListSeries,
    useCreateSeries,
    useDeleteSeries,
    useListSeriesMembers,
    useRemoveSeriesArticle,
    useReorderSeriesArticles,
    useUpdateSeries,
} from "../api/admin/admin";
import { useListPublishedArticles } from "../api/articles/articles";
import { getMeQueryOptions } from "../api/auth/auth";
import { FetchError } from "../api/fetcher";
import type { SeriesAdminResponse, SeriesMemberResponse } from "../api/model";
import { useListTopics } from "../api/topics/topics";
import { useDebouncedValue } from "../hooks/useDebouncedValue";

export const Route = createFileRoute("/_auth/_manage/manage/series/")({
    beforeLoad: async ({ context }) => {
        const me = await context.queryClient.fetchQuery(getMeQueryOptions());
        if (!me?.data?.permissions?.includes("series_manage")) {
            throw notFound();
        }
    },
    component: SeriesPage,
});

function errorMessage(err: unknown, fallback: string) {
    return err instanceof FetchError && err.message ? err.message : fallback;
}

function SeriesPage() {
    const queryClient = useQueryClient();
    const { data, isFetching } = useAdminListSeries();
    const series = data?.data?.series ?? [];

    const invalidate = () =>
        queryClient.invalidateQueries({
            queryKey: getAdminListSeriesQueryKey(),
        });

    const [newName, setNewName] = useState("");
    const createMutation = useCreateSeries({
        mutation: {
            onSuccess: () => {
                setNewName("");
                invalidate();
            },
            onError: (err) =>
                toast.error(errorMessage(err, "Failed to create series")),
        },
    });

    const handleCreate = () => {
        const name = newName.trim();
        if (!name) return;
        createMutation.mutate({ data: { name } });
    };

    const [activeId, setActiveId] = useState<string | null>(null);
    const active = series.find((s) => s.id === activeId) ?? null;

    const columns = useMemo<GridColDef<SeriesAdminResponse>[]>(
        () => [
            { field: "name", headerName: "Name", flex: 1, minWidth: 180 },
            {
                field: "slug",
                headerName: "Slug",
                flex: 1,
                minWidth: 140,
                renderCell: (params) => (
                    <span className="font-mono text-xs text-stone-500">
                        {params.value}
                    </span>
                ),
            },
            {
                field: "article_count",
                headerName: "Articles",
                width: 110,
                valueGetter: (_, row) =>
                    row.published_count === row.article_count
                        ? `${row.article_count}`
                        : `${row.published_count} / ${row.article_count}`,
                sortable: false,
            },
            {
                field: "pinned",
                headerName: "Pinned",
                width: 90,
                type: "boolean",
            },
            { field: "sort_order", headerName: "Order", width: 80 },
            {
                field: "updated_at",
                headerName: "Updated",
                width: 110,
                valueFormatter: (value: string) =>
                    new Date(value).toLocaleDateString(),
            },
            {
                field: "view",
                headerName: "",
                width: 56,
                sortable: false,
                disableColumnMenu: true,
                renderCell: (params) => (
                    <IconButton
                        size="small"
                        component="a"
                        href={`/articles/series/${params.row.slug}`}
                        target="_blank"
                        rel="noopener"
                        onClick={(e) => e.stopPropagation()}
                        aria-label="Open series page in new tab"
                    >
                        <OpenInNewOutlined sx={{ fontSize: 16 }} />
                    </IconButton>
                ),
            },
        ],
        [],
    );

    return (
        <div className="max-w-4xl mx-auto px-8 py-16">
            <h1 className="text-lg font-semibold text-stone-800 mb-1">
                Series
            </h1>
            <p className="text-sm text-stone-500 mb-6">
                Curated, ordered collections of published articles. Adding an
                article puts it first; pinned series appear as shelves on the
                articles page. The slug is generated once and never changes.
            </p>

            <div className="flex gap-2 mb-8">
                <TextField
                    label="New series"
                    value={newName}
                    onChange={(e) => setNewName(e.target.value)}
                    onKeyDown={(e) => {
                        if (e.key === "Enter") handleCreate();
                    }}
                    size="small"
                    sx={{ flex: 1 }}
                />
                <Button
                    variant="contained"
                    size="small"
                    onClick={handleCreate}
                    disabled={!newName.trim() || createMutation.isPending}
                    sx={{ textTransform: "none" }}
                >
                    Add
                </Button>
            </div>

            <DataGrid
                rows={series}
                columns={columns}
                loading={isFetching}
                disableColumnFilter
                disableRowSelectionOnClick
                onRowClick={(params) => setActiveId(params.row.id)}
                sx={{
                    border: "1px solid rgb(214 211 209)",
                    "& .MuiDataGrid-row": { cursor: "pointer" },
                }}
            />

            <Drawer
                anchor="right"
                open={active !== null}
                onClose={() => setActiveId(null)}
                slotProps={{
                    paper: {
                        sx: { width: { xs: "100%", sm: 480 }, p: 3 },
                    },
                }}
            >
                {active && (
                    <SeriesDrawer
                        key={active.id}
                        series={active}
                        onChanged={invalidate}
                        onDeleted={() => {
                            setActiveId(null);
                            invalidate();
                        }}
                        onClose={() => setActiveId(null)}
                    />
                )}
            </Drawer>
        </div>
    );
}

function SeriesDrawer({
    series,
    onChanged,
    onDeleted,
    onClose,
}: {
    series: SeriesAdminResponse;
    onChanged: () => void;
    onDeleted: () => void;
    onClose: () => void;
}) {
    const [name, setName] = useState(series.name);
    const [description, setDescription] = useState(series.description ?? "");
    const [pinned, setPinned] = useState(series.pinned);
    const [sortOrder, setSortOrder] = useState(String(series.sort_order));

    const updateMutation = useUpdateSeries({
        mutation: {
            onSuccess: onChanged,
            onError: (err) =>
                toast.error(errorMessage(err, "Failed to update series")),
        },
    });
    const deleteMutation = useDeleteSeries({
        mutation: {
            onSuccess: onDeleted,
            onError: (err) =>
                toast.error(errorMessage(err, "Failed to delete series")),
        },
    });

    const parsedSortOrder = Number.parseInt(sortOrder, 10);
    const dirty =
        name.trim() !== series.name ||
        description.trim() !== (series.description ?? "") ||
        pinned !== series.pinned ||
        (!Number.isNaN(parsedSortOrder) &&
            parsedSortOrder !== series.sort_order);

    const handleSave = () => {
        if (!dirty || !name.trim()) return;
        updateMutation.mutate({
            id: series.id,
            data: {
                name: name.trim() !== series.name ? name.trim() : undefined,
                // An empty string clears the description server-side.
                description:
                    description.trim() !== (series.description ?? "")
                        ? description.trim()
                        : undefined,
                pinned: pinned !== series.pinned ? pinned : undefined,
                sort_order:
                    !Number.isNaN(parsedSortOrder) &&
                    parsedSortOrder !== series.sort_order
                        ? parsedSortOrder
                        : undefined,
            },
        });
    };

    const inUse = series.article_count > 0;

    return (
        <div className="flex flex-col gap-6">
            <div className="flex items-center justify-between">
                <h2 className="text-base font-semibold text-stone-800">
                    Edit series
                </h2>
                <IconButton size="small" onClick={onClose} aria-label="Close">
                    <CloseOutlined sx={{ fontSize: 18 }} />
                </IconButton>
            </div>

            <div className="flex flex-col gap-4">
                <TextField
                    label="Name"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    size="small"
                    fullWidth
                />
                <TextField
                    label="Description"
                    value={description}
                    onChange={(e) => setDescription(e.target.value)}
                    size="small"
                    fullWidth
                    multiline
                    minRows={2}
                />
                <div className="flex items-center gap-4">
                    <FormControlLabel
                        control={
                            <Switch
                                size="small"
                                checked={pinned}
                                onChange={(e) => setPinned(e.target.checked)}
                            />
                        }
                        label={
                            <span className="text-sm text-stone-600">
                                Pinned
                            </span>
                        }
                    />
                    <TextField
                        label="Sort order"
                        value={sortOrder}
                        onChange={(e) => setSortOrder(e.target.value)}
                        size="small"
                        type="number"
                        sx={{ width: 110 }}
                    />
                </div>
                <div className="flex items-center justify-between">
                    <Button
                        variant="contained"
                        size="small"
                        onClick={handleSave}
                        disabled={
                            !dirty || !name.trim() || updateMutation.isPending
                        }
                        sx={{ textTransform: "none" }}
                    >
                        Save
                    </Button>
                    <Tooltip title={inUse ? "Detach all articles first" : ""}>
                        <span>
                            <Button
                                size="small"
                                color="error"
                                onClick={() => {
                                    if (
                                        confirm(
                                            `Delete series "${series.name}"?`,
                                        )
                                    ) {
                                        deleteMutation.mutate({
                                            id: series.id,
                                        });
                                    }
                                }}
                                disabled={inUse || deleteMutation.isPending}
                                sx={{
                                    textTransform: "none",
                                    fontSize: "0.75rem",
                                }}
                            >
                                Delete
                            </Button>
                        </span>
                    </Tooltip>
                </div>
                <p className="text-[0.7rem] font-mono text-stone-400">
                    /articles/series/{series.slug}
                </p>
            </div>

            <SeriesMembers seriesId={series.id} onChanged={onChanged} />
        </div>
    );
}

function SeriesMembers({
    seriesId,
    onChanged,
}: {
    seriesId: string;
    onChanged: () => void;
}) {
    const queryClient = useQueryClient();
    const { data } = useListSeriesMembers(seriesId);
    const members = data?.data?.articles ?? [];

    const invalidateMembers = () => {
        queryClient.invalidateQueries({
            queryKey: getListSeriesMembersQueryKey(seriesId),
        });
        onChanged();
    };

    const removeMutation = useRemoveSeriesArticle({
        mutation: {
            onSuccess: invalidateMembers,
            onError: (err) =>
                toast.error(errorMessage(err, "Failed to remove article")),
        },
    });
    const reorderMutation = useReorderSeriesArticles({
        mutation: {
            onSuccess: invalidateMembers,
            onError: (err) =>
                toast.error(errorMessage(err, "Failed to reorder articles")),
        },
    });

    const move = (index: number, delta: -1 | 1) => {
        const target = index + delta;
        if (target < 0 || target >= members.length) return;
        const ids = members.map((m) => m.article_id);
        [ids[index], ids[target]] = [ids[target], ids[index]];
        reorderMutation.mutate({
            id: seriesId,
            data: { article_ids: ids },
        });
    };

    return (
        <div>
            <h3 className="text-sm font-semibold text-stone-700 mb-2">
                Articles
            </h3>
            {members.length === 0 && (
                <p className="text-sm text-stone-400 mb-2">
                    No articles in this series yet.
                </p>
            )}
            <ul className="divide-y divide-stone-100 border border-stone-200 rounded mb-4">
                {members.map((member, index) => (
                    <MemberRow
                        key={member.article_id}
                        member={member}
                        canMoveUp={index > 0}
                        canMoveDown={index < members.length - 1}
                        busy={
                            reorderMutation.isPending ||
                            removeMutation.isPending
                        }
                        onMoveUp={() => move(index, -1)}
                        onMoveDown={() => move(index, 1)}
                        onRemove={() =>
                            removeMutation.mutate({
                                id: seriesId,
                                articleId: member.article_id,
                            })
                        }
                    />
                ))}
            </ul>

            <ArticleFinder
                seriesId={seriesId}
                memberIds={members.map((m) => m.article_id)}
                onAdded={invalidateMembers}
            />
        </div>
    );
}

function MemberRow({
    member,
    canMoveUp,
    canMoveDown,
    busy,
    onMoveUp,
    onMoveDown,
    onRemove,
}: {
    member: SeriesMemberResponse;
    canMoveUp: boolean;
    canMoveDown: boolean;
    busy: boolean;
    onMoveUp: () => void;
    onMoveDown: () => void;
    onRemove: () => void;
}) {
    const unpublished = member.status !== "published";
    return (
        <li className="flex items-center gap-2 px-3 py-2">
            <span className="text-xs text-stone-400 tabular-nums w-5 shrink-0">
                {member.position}
            </span>
            <div
                className={`flex-1 min-w-0 ${unpublished ? "opacity-50" : ""}`}
            >
                <span className="block text-sm text-stone-800 truncate">
                    {member.title}
                </span>
                <span className="text-xs text-stone-400">
                    {member.author_display_name}
                </span>
                {unpublished && (
                    <Chip
                        label={member.status}
                        size="small"
                        sx={{ ml: 1, fontSize: "0.65rem", height: 18 }}
                    />
                )}
            </div>
            <IconButton
                size="small"
                onClick={onMoveUp}
                disabled={!canMoveUp || busy}
                aria-label="Move up"
            >
                <ArrowUpwardOutlined sx={{ fontSize: 16 }} />
            </IconButton>
            <IconButton
                size="small"
                onClick={onMoveDown}
                disabled={!canMoveDown || busy}
                aria-label="Move down"
            >
                <ArrowDownwardOutlined sx={{ fontSize: 16 }} />
            </IconButton>
            <IconButton
                size="small"
                onClick={onRemove}
                disabled={busy}
                aria-label="Remove from series"
            >
                <CloseOutlined sx={{ fontSize: 16 }} />
            </IconButton>
        </li>
    );
}

function ArticleFinder({
    seriesId,
    memberIds,
    onAdded,
}: {
    seriesId: string;
    memberIds: string[];
    onAdded: () => void;
}) {
    const [q, setQ] = useState("");
    const [author, setAuthor] = useState("");
    const [topicSlug, setTopicSlug] = useState("");
    const debouncedQ = useDebouncedValue(q);
    const debouncedAuthor = useDebouncedValue(author);

    const { data: topicsData } = useListTopics();
    const topics = topicsData?.data?.topics ?? [];

    const { data, isFetching } = useListPublishedArticles({
        q: debouncedQ.trim() || undefined,
        author: debouncedAuthor.trim() || undefined,
        topic_slug: topicSlug || undefined,
        per_page: 10,
    });
    const results = data?.data?.articles ?? [];

    const addMutation = useAddSeriesArticle({
        mutation: {
            onSuccess: onAdded,
            onError: (err) =>
                toast.error(errorMessage(err, "Failed to add article")),
        },
    });

    return (
        <div>
            <h3 className="text-sm font-semibold text-stone-700 mb-2">
                Add articles
            </h3>
            <div className="flex flex-col gap-2 mb-3">
                <TextField
                    label="Title"
                    value={q}
                    onChange={(e) => setQ(e.target.value)}
                    size="small"
                    fullWidth
                />
                <div className="flex gap-2">
                    <TextField
                        label="Author"
                        value={author}
                        onChange={(e) => setAuthor(e.target.value)}
                        size="small"
                        sx={{ flex: 1 }}
                    />
                    <TextField
                        label="Topic"
                        value={topicSlug}
                        onChange={(e) => setTopicSlug(e.target.value)}
                        size="small"
                        select
                        sx={{ flex: 1 }}
                    >
                        <MenuItem value="">All topics</MenuItem>
                        {topics.map((t) => (
                            <MenuItem key={t.id} value={t.slug}>
                                {t.name}
                            </MenuItem>
                        ))}
                    </TextField>
                </div>
            </div>
            {isFetching && (
                <p className="text-xs text-stone-400 mb-2">Searching…</p>
            )}
            <ul className="divide-y divide-stone-100 border border-stone-200 rounded">
                {results.length === 0 && !isFetching && (
                    <li className="px-3 py-2 text-sm text-stone-400">
                        No matching published articles.
                    </li>
                )}
                {results.map((article) => {
                    const isMember = memberIds.includes(article.id);
                    return (
                        <li
                            key={article.id}
                            className="flex items-center gap-2 px-3 py-2"
                        >
                            <div className="flex-1 min-w-0">
                                <span className="block text-sm text-stone-800 truncate">
                                    {article.title}
                                </span>
                                <span className="text-xs text-stone-400">
                                    {article.author_display_name}
                                </span>
                            </div>
                            <Tooltip
                                title={isMember ? "Already in this series" : ""}
                            >
                                <span>
                                    <IconButton
                                        size="small"
                                        onClick={() =>
                                            addMutation.mutate({
                                                id: seriesId,
                                                data: {
                                                    article_id: article.id,
                                                },
                                            })
                                        }
                                        disabled={
                                            isMember || addMutation.isPending
                                        }
                                        aria-label="Add to series"
                                    >
                                        <AddOutlined sx={{ fontSize: 16 }} />
                                    </IconButton>
                                </span>
                            </Tooltip>
                        </li>
                    );
                })}
            </ul>
        </div>
    );
}
