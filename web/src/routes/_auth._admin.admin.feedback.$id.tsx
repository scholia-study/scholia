import {
    Button,
    Chip,
    MenuItem,
    Paper,
    Select,
    TextField,
    Typography,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute, Link } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import toast from "react-hot-toast";
import {
    getGetFeedbackQueryKey,
    getListFeedbackQueryKey,
    useGetFeedback,
    useUpdateFeedback,
} from "../api/feedback/feedback";
import { FetchError } from "../api/fetcher";
import type { FeedbackStatus } from "../api/model";

export const Route = createFileRoute("/_auth/_admin/admin/feedback/$id")({
    component: FeedbackDetail,
});

const STATUS_OPTIONS: { value: FeedbackStatus; label: string }[] = [
    { value: "todo", label: "To do" },
    { value: "in_progress", label: "In progress" },
    { value: "done", label: "Done" },
    { value: "cancelled", label: "Cancelled" },
];

function FeedbackDetail() {
    const { id } = Route.useParams();
    const queryClient = useQueryClient();
    const { data, isLoading } = useGetFeedback(id);
    const f = data?.data;

    const [status, setStatus] = useState<FeedbackStatus | "">("");
    const [notes, setNotes] = useState("");

    useEffect(() => {
        if (f) {
            setStatus(f.status);
            setNotes(f.admin_notes ?? "");
        }
    }, [f]);

    const updateMutation = useUpdateFeedback();

    const dirty =
        f != null && (status !== f.status || notes !== (f.admin_notes ?? ""));

    const handleSave = async () => {
        if (!f || !dirty) return;
        try {
            await updateMutation.mutateAsync({
                id: f.id,
                data: {
                    status: status as FeedbackStatus,
                    admin_notes: notes,
                },
            });
            toast.success("Feedback updated.");
            queryClient.invalidateQueries({
                queryKey: getGetFeedbackQueryKey(f.id),
            });
            queryClient.invalidateQueries({
                queryKey: getListFeedbackQueryKey(),
            });
        } catch (err: unknown) {
            const message =
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to update feedback";
            toast.error(message);
        }
    };

    return (
        <div className="w-full max-w-3xl mx-auto px-8 py-12">
            <Link
                to="/admin/feedback"
                className="text-xs text-stone-500 no-underline hover:underline"
            >
                ← Back to feedback
            </Link>

            {isLoading && (
                <p className="text-sm text-stone-400 mt-6">Loading…</p>
            )}

            {!isLoading && !f && (
                <p className="text-sm text-stone-400 mt-6">
                    Feedback not found.
                </p>
            )}

            {f ? (
                <>
                    <div className="flex items-center gap-2 mt-3 mb-4">
                        <h1 className="text-2xl font-bold text-stone-900">
                            Feedback
                        </h1>
                        <Chip
                            label={
                                f.status === "in_progress"
                                    ? "in progress"
                                    : f.status
                            }
                            size="small"
                            sx={{ height: 22 }}
                        />
                    </div>

                    <Paper
                        variant="outlined"
                        sx={{ p: 2, mb: 3, bgcolor: "#fff" }}
                    >
                        <p className="text-sm text-stone-700 whitespace-pre-wrap break-words">
                            {f.body}
                        </p>
                    </Paper>

                    <dl className="grid grid-cols-[120px_1fr] gap-y-1 gap-x-4 text-xs text-stone-600 mb-6">
                        <dt className="text-stone-400">Submitter</dt>
                        <dd>
                            {f.submitter
                                ? `${f.submitter.display_name} · ${f.submitter.email}`
                                : "User deleted"}
                        </dd>
                        <dt className="text-stone-400">Submitted</dt>
                        <dd>
                            {new Date(f.created_at).toLocaleString(undefined, {
                                dateStyle: "medium",
                                timeStyle: "short",
                            })}
                        </dd>
                        {f.url && (
                            <>
                                <dt className="text-stone-400">URL</dt>
                                <dd className="break-all">
                                    <a
                                        href={f.url}
                                        target="_blank"
                                        rel="noreferrer"
                                        className="text-stone-700 hover:underline"
                                    >
                                        {f.url}
                                    </a>
                                </dd>
                            </>
                        )}
                        {f.user_agent && (
                            <>
                                <dt className="text-stone-400">User agent</dt>
                                <dd className="break-all">{f.user_agent}</dd>
                            </>
                        )}
                        {f.viewport_w != null && f.viewport_h != null && (
                            <>
                                <dt className="text-stone-400">Viewport</dt>
                                <dd>
                                    {f.viewport_w}×{f.viewport_h}
                                </dd>
                            </>
                        )}
                        {f.handled_by && (
                            <>
                                <dt className="text-stone-400">Last handled</dt>
                                <dd>
                                    {f.handled_by.display_name} ·{" "}
                                    {new Date(f.updated_at).toLocaleString(
                                        undefined,
                                        {
                                            dateStyle: "medium",
                                            timeStyle: "short",
                                        },
                                    )}
                                </dd>
                            </>
                        )}
                    </dl>

                    <div className="space-y-3">
                        <div>
                            <Typography
                                variant="caption"
                                sx={{
                                    color: "rgb(120 113 108)",
                                    display: "block",
                                    mb: 0.5,
                                }}
                            >
                                Status
                            </Typography>
                            <Select
                                size="small"
                                value={status}
                                onChange={(e) =>
                                    setStatus(e.target.value as FeedbackStatus)
                                }
                                sx={{ minWidth: 200 }}
                            >
                                {STATUS_OPTIONS.map((s) => (
                                    <MenuItem key={s.value} value={s.value}>
                                        {s.label}
                                    </MenuItem>
                                ))}
                            </Select>
                        </div>

                        <TextField
                            label="Admin notes"
                            multiline
                            minRows={4}
                            fullWidth
                            value={notes}
                            onChange={(e) => setNotes(e.target.value)}
                            placeholder="Internal notes — not shown to the submitter."
                        />

                        <div className="flex justify-end">
                            <Button
                                variant="contained"
                                size="small"
                                onClick={handleSave}
                                disabled={!dirty || updateMutation.isPending}
                                sx={{ textTransform: "none" }}
                            >
                                {updateMutation.isPending ? "Saving…" : "Save"}
                            </Button>
                        </div>
                    </div>
                </>
            ) : null}
        </div>
    );
}
