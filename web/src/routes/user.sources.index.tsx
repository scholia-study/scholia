import AddOutlined from "@mui/icons-material/AddOutlined";
import EditOutlined from "@mui/icons-material/EditOutlined";
import LockOutlined from "@mui/icons-material/LockOutlined";
import {
    Button,
    Chip,
    FormControl,
    FormControlLabel,
    IconButton,
    InputLabel,
    MenuItem,
    Pagination,
    Paper,
    Select,
    Switch,
    TextField,
    Tooltip,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { Link, createFileRoute, redirect } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import {
    getBrowseSourcesQueryKey,
    useBrowseSources,
} from "../api/sources/sources";
import { getGetProfileQueryOptions } from "../api/auth/auth";
import type { SourceResponse, SourceSearchResponse } from "../api/model";
import { SourceFormModal } from "../components/SourceFormModal";
import { useAuth } from "../hooks/useAuth";
import { useDebouncedValue } from "../hooks/useDebouncedValue";

export const Route = createFileRoute("/user/sources/")({
    beforeLoad: async ({ context }) => {
        const data = await context.queryClient.fetchQuery(
            getGetProfileQueryOptions(),
        );
        if (!data?.data) {
            throw redirect({ to: "/login" });
        }
    },
    component: SourcesListPage,
});

const SOURCE_TYPES = ["book", "article", "chapter", "journal", "web"] as const;
const PER_PAGE = 20;

function SourcesListPage() {
    const queryClient = useQueryClient();
    const { user, hasPermission } = useAuth();
    const isEditor = hasPermission("resources_manage");

    const [q, setQ] = useState("");
    const debouncedQ = useDebouncedValue(q);
    const [sourceType, setSourceType] = useState<string>("");
    const [createdByMe, setCreatedByMe] = useState(false);
    const [protectedFilter, setProtectedFilter] = useState(false);
    const [page, setPage] = useState(1);

    const [createOpen, setCreateOpen] = useState(false);

    const params = useMemo(
        () => ({
            q: debouncedQ || undefined,
            source_type: sourceType || undefined,
            created_by_me: createdByMe || undefined,
            protected: isEditor && protectedFilter ? true : undefined,
            page,
            per_page: PER_PAGE,
        }),
        [debouncedQ, sourceType, createdByMe, protectedFilter, isEditor, page],
    );

    const { data, isLoading } = useBrowseSources(params);
    const sources = data?.data?.sources ?? [];
    const total = data?.data?.total ?? 0;
    const pageCount = Math.max(1, Math.ceil(total / PER_PAGE));

    const handleCreated = (_source: SourceResponse) => {
        setCreateOpen(false);
        queryClient.invalidateQueries({
            queryKey: getBrowseSourcesQueryKey(),
        });
    };

    return (
        <div className="w-full max-w-4xl mx-auto px-8 py-16">
            <div className="flex items-center justify-between mb-6">
                <h1 className="text-2xl font-bold text-stone-900">Sources</h1>
                <Button
                    variant="contained"
                    size="small"
                    startIcon={<AddOutlined />}
                    onClick={() => setCreateOpen(true)}
                    sx={{ textTransform: "none" }}
                >
                    New Source
                </Button>
            </div>

            <div className="flex flex-wrap gap-3 items-center mb-4">
                <TextField
                    label="Search title"
                    value={q}
                    onChange={(e) => {
                        setQ(e.target.value);
                        setPage(1);
                    }}
                    size="small"
                    sx={{ flex: "1 1 240px" }}
                />
                <FormControl size="small" sx={{ minWidth: 140 }}>
                    <InputLabel>Type</InputLabel>
                    <Select
                        value={sourceType}
                        onChange={(e) => {
                            setSourceType(e.target.value);
                            setPage(1);
                        }}
                        label="Type"
                    >
                        <MenuItem value="">All types</MenuItem>
                        {SOURCE_TYPES.map((t) => (
                            <MenuItem key={t} value={t}>
                                {t.charAt(0).toUpperCase() + t.slice(1)}
                            </MenuItem>
                        ))}
                    </Select>
                </FormControl>
                <FormControlLabel
                    control={
                        <Switch
                            checked={createdByMe}
                            onChange={(_, v) => {
                                setCreatedByMe(v);
                                setPage(1);
                            }}
                            size="small"
                        />
                    }
                    label={<span className="text-sm">Created by me</span>}
                />
                {isEditor && (
                    <FormControlLabel
                        control={
                            <Switch
                                checked={protectedFilter}
                                onChange={(_, v) => {
                                    setProtectedFilter(v);
                                    setPage(1);
                                }}
                                size="small"
                            />
                        }
                        label={<span className="text-sm">Protected only</span>}
                    />
                )}
            </div>

            {isLoading && (
                <p className="text-sm text-stone-400">Loading...</p>
            )}

            {!isLoading && sources.length === 0 && (
                <p className="text-sm text-stone-400">No sources found.</p>
            )}

            <div className="space-y-2">
                {sources.map((s) => (
                    <SourceRow
                        key={s.id}
                        source={s}
                        userId={user?.id}
                        isEditor={isEditor}
                    />
                ))}
            </div>

            {pageCount > 1 && (
                <div className="flex justify-center mt-6">
                    <Pagination
                        page={page}
                        count={pageCount}
                        onChange={(_, p) => setPage(p)}
                        size="small"
                    />
                </div>
            )}

            <SourceFormModal
                open={createOpen}
                onClose={() => setCreateOpen(false)}
                onCreated={handleCreated}
            />
        </div>
    );
}

function SourceRow({
    source,
    userId,
    isEditor,
}: {
    source: SourceSearchResponse;
    userId: string | undefined;
    isEditor: boolean;
}) {
    const isOwner = !!userId && source.created_by === userId;
    const canEdit = isEditor ? !source.protected || isEditor : isOwner && !source.protected;
    const tooltip = source.protected && !isEditor
        ? "This source is curated by editors and cannot be edited."
        : !canEdit
          ? "You can only edit sources you created."
          : "Edit";

    const authors = source.persons
        .filter((p) => p.role === "author")
        .map((p) => p.name)
        .join(", ");

    return (
        <Paper
            elevation={0}
            sx={{
                border: "1px solid rgb(214 211 209)",
                p: 1.5,
                display: "flex",
                alignItems: "center",
                gap: 1.5,
                transition: "box-shadow 0.15s",
                "&:hover": { boxShadow: 3 },
            }}
        >
            <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-0.5">
                    <span className="text-sm font-medium text-stone-900 truncate">
                        {source.title}
                    </span>
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
                </div>
                <div className="text-xs text-stone-400 truncate">
                    {authors && <span>{authors}</span>}
                    {authors && source.publication_year && (
                        <span> &middot; </span>
                    )}
                    {source.publication_year && (
                        <span>{source.publication_year}</span>
                    )}
                </div>
            </div>

            <div className="flex items-center gap-0.5 shrink-0">
                {canEdit ? (
                    <Tooltip title={tooltip}>
                        <Link to="/user/sources/$id" params={{ id: source.id }}>
                            <IconButton size="small">
                                <EditOutlined fontSize="small" />
                            </IconButton>
                        </Link>
                    </Tooltip>
                ) : (
                    <Tooltip title={tooltip}>
                        <span>
                            <IconButton size="small" disabled>
                                <EditOutlined fontSize="small" />
                            </IconButton>
                        </span>
                    </Tooltip>
                )}
            </div>
        </Paper>
    );
}
