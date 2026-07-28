import OpenInNewOutlined from "@mui/icons-material/OpenInNewOutlined";
import { Box, Chip, IconButton, Tab, Tabs, Tooltip } from "@mui/material";
import {
    DataGrid,
    type GridColDef,
    type GridPaginationModel,
} from "@mui/x-data-grid";
import { keepPreviousData } from "@tanstack/react-query";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { useListArticleReviewQueue } from "../api/article-reviews/article-reviews";
import type { ArticleReviewQueueItem } from "../api/model";

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
    const [filter, setFilter] = useState<Filter>("pending");
    const [pagination, setPagination] = useState<GridPaginationModel>({
        page: 0,
        pageSize: 25,
    });

    const { data, isFetching } = useListArticleReviewQueue(
        {
            filter,
            page: pagination.page + 1,
            per_page: pagination.pageSize,
        },
        { query: { placeholderData: keepPreviousData } },
    );

    const items = data?.data?.items ?? [];
    const total = data?.data?.total ?? 0;

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
        [],
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

            <Tabs
                value={filter}
                onChange={(_e, v: Filter) => {
                    setFilter(v);
                    setPagination((p) => ({ ...p, page: 0 }));
                }}
                sx={{ mb: 2, minHeight: 0 }}
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
