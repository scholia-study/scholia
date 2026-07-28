import OpenInNewOutlined from "@mui/icons-material/OpenInNewOutlined";
import {
    Box,
    Chip,
    IconButton,
    MenuItem,
    Select,
    Tab,
    Tabs,
    Tooltip,
} from "@mui/material";
import {
    DataGrid,
    type GridColDef,
    type GridPaginationModel,
} from "@mui/x-data-grid";
import { keepPreviousData, useQueryClient } from "@tanstack/react-query";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";
import toast from "react-hot-toast";
import {
    getListArticleReviewQueueQueryKey,
    useAssignArticleReview,
    useListArticleReviewers,
    useListArticleReviewQueue,
} from "../api/article-reviews/article-reviews";
import { FetchError } from "../api/fetcher";
import type { ArticleReviewQueueItem } from "../api/model";
import { useAuth } from "../hooks/useAuth";

export const Route = createFileRoute("/_auth/_editor/editor/article-reviews/")({
    component: ArticleReviewQueue,
});

const FILTERS = ["pending", "approved", "declined", "resolved", "all"] as const;
type Filter = (typeof FILTERS)[number];

const STATUS_COLORS: Record<
    string,
    "warning" | "success" | "error" | "default"
> = {
    pending: "warning",
    approved: "success",
    declined: "error",
    resolved: "default",
};

function ArticleReviewQueue() {
    const navigate = useNavigate();
    const queryClient = useQueryClient();
    const { user } = useAuth();
    const [filter, setFilter] = useState<Filter>("pending");
    // "" = everyone, "me" resolves to the viewer's id, "unassigned", or
    // a reviewer's user id.
    const [assigneeFilter, setAssigneeFilter] = useState("");
    const [pagination, setPagination] = useState<GridPaginationModel>({
        page: 0,
        pageSize: 25,
    });

    const resolvedAssignee =
        assigneeFilter === "me" ? (user?.id ?? "") : assigneeFilter;
    const { data, isFetching } = useListArticleReviewQueue(
        {
            filter,
            assignee: resolvedAssignee || undefined,
            page: pagination.page + 1,
            per_page: pagination.pageSize,
        },
        { query: { placeholderData: keepPreviousData } },
    );
    const { data: reviewersData } = useListArticleReviewers();
    const reviewers = reviewersData?.data?.reviewers ?? [];
    const assignMutation = useAssignArticleReview();

    const items = data?.data?.items ?? [];
    const total = data?.data?.total ?? 0;

    const assign = useCallback(
        async (requestId: string, assigneeId: string | null) => {
            try {
                await assignMutation.mutateAsync({
                    id: requestId,
                    data: { assignee_id: assigneeId },
                });
                queryClient.invalidateQueries({
                    queryKey: getListArticleReviewQueueQueryKey(),
                });
            } catch (err) {
                toast.error(
                    err instanceof FetchError && err.message
                        ? err.message
                        : "Failed to assign reviewer",
                );
            }
        },
        [assignMutation, queryClient],
    );

    const columns = useMemo<GridColDef<ArticleReviewQueueItem>[]>(
        () => [
            {
                field: "submitted_at",
                headerName: "Submitted",
                width: 120,
                valueFormatter: (value: string) =>
                    new Date(value).toLocaleDateString(undefined, {
                        month: "short",
                        day: "numeric",
                        year: "numeric",
                    }),
            },
            {
                field: "article_title",
                headerName: "Article",
                flex: 1.6,
                minWidth: 200,
                sortable: false,
            },
            {
                field: "author_display_name",
                headerName: "Author",
                flex: 1,
                minWidth: 140,
                sortable: false,
            },
            {
                field: "intent",
                headerName: "Intent",
                width: 110,
                sortable: false,
            },
            {
                field: "article_status",
                headerName: "Article",
                width: 100,
                sortable: false,
            },
            {
                field: "open_comment_count",
                headerName: "Open",
                width: 70,
                sortable: false,
            },
            {
                field: "assignee",
                headerName: "Assignee",
                width: 170,
                sortable: false,
                renderCell: (params) => {
                    const row = params.row as ArticleReviewQueueItem;
                    if (row.status !== "pending") {
                        return (
                            <span className="text-sm text-stone-500">
                                {row.assignee?.display_name ?? "—"}
                            </span>
                        );
                    }
                    return (
                        <Select
                            value={row.assignee?.user_id ?? ""}
                            size="small"
                            variant="standard"
                            disableUnderline
                            displayEmpty
                            fullWidth
                            onClick={(e) => e.stopPropagation()}
                            onChange={(e) =>
                                assign(row.id, e.target.value || null)
                            }
                            renderValue={(v) => {
                                if (!v)
                                    return (
                                        <span className="text-stone-400">
                                            Unassigned
                                        </span>
                                    );
                                const r = reviewers.find(
                                    (x) => x.user_id === v,
                                );
                                const name =
                                    r?.display_name ??
                                    row.assignee?.display_name ??
                                    "?";
                                return v === user?.id ? `${name} (me)` : name;
                            }}
                            sx={{ fontSize: "0.8rem" }}
                        >
                            <MenuItem value="">
                                <em>Unassigned</em>
                            </MenuItem>
                            {reviewers.map((r) => (
                                <MenuItem key={r.user_id} value={r.user_id}>
                                    {r.user_id === user?.id
                                        ? `${r.display_name} (me)`
                                        : r.display_name}
                                </MenuItem>
                            ))}
                        </Select>
                    );
                },
            },
            {
                field: "status",
                headerName: "Status",
                width: 110,
                sortable: false,
                renderCell: (params) => (
                    <Chip
                        label={params.value}
                        size="small"
                        color={STATUS_COLORS[params.value] ?? "default"}
                        variant="outlined"
                        sx={{ height: 20, fontSize: "0.65rem" }}
                    />
                ),
            },
            {
                field: "actions",
                headerName: "",
                width: 50,
                sortable: false,
                renderCell: (params) => (
                    <Tooltip title="Open in new tab">
                        <IconButton
                            size="small"
                            component="a"
                            href={`/articles/review/${params.row.id}`}
                            target="_blank"
                            rel="noopener"
                            onClick={(e) => e.stopPropagation()}
                        >
                            <OpenInNewOutlined sx={{ fontSize: 16 }} />
                        </IconButton>
                    </Tooltip>
                ),
            },
        ],
        [reviewers, user?.id, assign],
    );

    return (
        <div className="w-full max-w-6xl mx-auto px-8 py-12">
            <h1 className="text-2xl font-bold text-stone-900 mb-1">
                Article reviews
            </h1>
            <p className="text-sm text-stone-500 mb-6">
                Articles submitted for feedback or publication. Click a row to
                open the review page.
            </p>

            <div className="flex items-center justify-between gap-4 mb-2">
                <Tabs
                    value={filter}
                    onChange={(_e, v: Filter) => {
                        setFilter(v);
                        setPagination((p) => ({ ...p, page: 0 }));
                    }}
                    sx={{ minHeight: 0 }}
                >
                    {FILTERS.map((f) => (
                        <Tab
                            key={f}
                            value={f}
                            label={f}
                            sx={{
                                minHeight: 0,
                                py: 1,
                                textTransform: "capitalize",
                            }}
                        />
                    ))}
                </Tabs>
                <Select
                    value={assigneeFilter}
                    size="small"
                    displayEmpty
                    onChange={(e) => {
                        setAssigneeFilter(e.target.value);
                        setPagination((p) => ({ ...p, page: 0 }));
                    }}
                    sx={{ minWidth: 180, fontSize: "0.85rem" }}
                >
                    <MenuItem value="">Assignee: everyone</MenuItem>
                    <MenuItem value="me">Assigned to me</MenuItem>
                    <MenuItem value="unassigned">Unassigned</MenuItem>
                    {reviewers
                        .filter((r) => r.user_id !== user?.id)
                        .map((r) => (
                            <MenuItem key={r.user_id} value={r.user_id}>
                                {r.display_name}
                            </MenuItem>
                        ))}
                </Select>
            </div>

            <Box sx={{ width: "100%" }}>
                <DataGrid
                    rows={items}
                    columns={columns}
                    loading={isFetching}
                    rowCount={total}
                    paginationMode="server"
                    paginationModel={pagination}
                    onPaginationModelChange={setPagination}
                    pageSizeOptions={[25, 50, 100]}
                    disableColumnFilter
                    disableRowSelectionOnClick
                    onRowClick={(params) => {
                        navigate({
                            to: "/articles/review/$requestId",
                            params: { requestId: String(params.id) },
                        });
                    }}
                    sx={{
                        border: "1px solid rgb(214 211 209)",
                        "& .MuiDataGrid-row": { cursor: "pointer" },
                    }}
                />
            </Box>
        </div>
    );
}
