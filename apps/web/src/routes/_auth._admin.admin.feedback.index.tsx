import { Chip, Pagination, Paper, Tab, Tabs, Typography } from "@mui/material";
import { createFileRoute, Link } from "@tanstack/react-router";
import { useState } from "react";
import { useListFeedback } from "../api/feedback/feedback";
import type { FeedbackResponse, FeedbackStatus } from "../api/model";

const FILTERS = [
    "active",
    "todo",
    "in_progress",
    "done",
    "cancelled",
    "all",
] as const;
type Filter = (typeof FILTERS)[number];

export const Route = createFileRoute("/_auth/_admin/admin/feedback/")({
    component: FeedbackDashboard,
});

const STATUS_COLORS: Record<
    FeedbackStatus,
    "default" | "primary" | "success" | "warning"
> = {
    todo: "primary",
    in_progress: "warning",
    done: "success",
    cancelled: "default",
};

function FeedbackDashboard() {
    const [filter, setFilter] = useState<Filter>("active");
    const [page, setPage] = useState(1);
    const perPage = 25;

    const { data, isLoading } = useListFeedback({
        filter,
        page,
        per_page: perPage,
    });

    const list = data?.data?.feedback ?? [];
    const total = data?.data?.total ?? 0;
    const totalPages = Math.max(1, Math.ceil(total / perPage));

    return (
        <div className="w-full max-w-4xl mx-auto px-8 py-12">
            <h1 className="text-2xl font-bold text-stone-900 mb-1">Feedback</h1>
            <p className="text-sm text-stone-500 mb-6">
                Review user-submitted feedback.
            </p>

            <Tabs
                value={filter}
                onChange={(_, v) => {
                    setFilter(v as Filter);
                    setPage(1);
                }}
                sx={{ mb: 3, minHeight: 36 }}
            >
                {FILTERS.map((f) => (
                    <Tab
                        key={f}
                        value={f}
                        label={
                            f === "in_progress"
                                ? "In progress"
                                : f.charAt(0).toUpperCase() + f.slice(1)
                        }
                        sx={{ textTransform: "none", minHeight: 36 }}
                    />
                ))}
            </Tabs>

            {isLoading && <p className="text-sm text-stone-400">Loading…</p>}

            {!isLoading && list.length === 0 && (
                <p className="text-sm text-stone-400">No feedback to show.</p>
            )}

            <div className="space-y-2">
                {list.map((f) => (
                    <FeedbackRow key={f.id} f={f} />
                ))}
            </div>

            {totalPages > 1 && (
                <div className="flex justify-center mt-6">
                    <Pagination
                        page={page}
                        count={totalPages}
                        onChange={(_, p) => setPage(p)}
                        size="small"
                    />
                </div>
            )}
        </div>
    );
}

function FeedbackRow({ f }: { f: FeedbackResponse }) {
    const submitter = f.submitter
        ? `${f.submitter.display_name} · ${f.submitter.email}`
        : "User deleted";
    const preview = f.body.length > 140 ? `${f.body.slice(0, 140)}…` : f.body;
    const date = new Date(f.created_at).toLocaleDateString(undefined, {
        month: "short",
        day: "numeric",
        year: "numeric",
    });

    return (
        <Link
            to="/admin/feedback/$id"
            params={{ id: f.id }}
            style={{ textDecoration: "none", color: "inherit" }}
        >
            <Paper
                elevation={0}
                sx={{
                    border: "1px solid rgb(214 211 209)",
                    p: 1.5,
                    transition: "box-shadow 0.15s",
                    "&:hover": { boxShadow: 3 },
                }}
            >
                <div className="flex items-start gap-3">
                    <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                            <Chip
                                label={
                                    f.status === "in_progress"
                                        ? "in progress"
                                        : f.status
                                }
                                size="small"
                                color={STATUS_COLORS[f.status]}
                                sx={{ height: 20, fontSize: "0.65rem" }}
                            />
                            <Typography
                                variant="caption"
                                sx={{ color: "rgb(120 113 108)" }}
                            >
                                {submitter}
                            </Typography>
                        </div>
                        <p className="text-sm text-stone-700 whitespace-pre-wrap break-words">
                            {preview}
                        </p>
                        {f.url && (
                            <p className="text-[10px] text-stone-400 mt-1 truncate">
                                {f.url}
                            </p>
                        )}
                    </div>
                    <div className="text-[10px] text-stone-400 shrink-0 self-center">
                        {date}
                    </div>
                </div>
            </Paper>
        </Link>
    );
}
