import {
    Box,
    Button,
    Chip,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    Tab,
    Tabs,
    TextField,
} from "@mui/material";
import {
    DataGrid,
    type GridColDef,
    type GridPaginationModel,
} from "@mui/x-data-grid";
import { keepPreviousData, useQueryClient } from "@tanstack/react-query";
import { createFileRoute, notFound } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import toast from "react-hot-toast";
import { getMeQueryOptions } from "../api/auth/auth";
import { FetchError } from "../api/fetcher";
import type { ResourceSubmissionResponse } from "../api/model";
import {
    getListResourceSubmissionsQueryKey,
    useListResourceSubmissions,
    useReviewResourceSubmission,
} from "../api/resources/resources";
import { ResourceFormModal } from "../modules/reader";

export const Route = createFileRoute(
    "/_auth/_manage/manage/resource-submissions/",
)({
    beforeLoad: async ({ context }) => {
        const me = await context.queryClient.fetchQuery(getMeQueryOptions());
        if (!me?.data?.permissions?.includes("resources_manage")) {
            throw notFound();
        }
    },
    component: SubmissionsQueue,
});

const FILTERS = ["pending", "approved", "rejected", "all"] as const;
type Filter = (typeof FILTERS)[number];

const STATUS_COLORS: Record<string, "warning" | "success" | "default"> = {
    pending: "warning",
    approved: "success",
    rejected: "default",
};

function locationOf(r: ResourceSubmissionResponse["resource"]): string {
    if (r.source_page_start != null) {
        return r.source_page_end && r.source_page_end !== r.source_page_start
            ? `pp. ${r.source_page_start}–${r.source_page_end}`
            : `p. ${r.source_page_start}`;
    }
    return r.source_location_freeform ?? "—";
}

function SubmissionsQueue() {
    const queryClient = useQueryClient();
    const reviewMutation = useReviewResourceSubmission();

    const [filter, setFilter] = useState<Filter>("pending");
    const [pagination, setPagination] = useState<GridPaginationModel>({
        page: 0,
        pageSize: 25,
    });

    // The submission being reviewed in the detail dialog.
    const [active, setActive] = useState<ResourceSubmissionResponse | null>(
        null,
    );
    const [rejectNote, setRejectNote] = useState("");
    const [editing, setEditing] = useState(false);

    const { data, isFetching } = useListResourceSubmissions(
        {
            filter,
            page: pagination.page + 1,
            per_page: pagination.pageSize,
        },
        { query: { placeholderData: keepPreviousData } },
    );

    const submissions = data?.data?.submissions ?? [];
    const total = data?.data?.total ?? 0;
    const rows = submissions.map((s) => s.resource);

    const invalidate = () =>
        queryClient.invalidateQueries({
            queryKey: getListResourceSubmissionsQueryKey(),
        });

    const closeDialog = () => {
        setActive(null);
        setRejectNote("");
    };

    const review = async (status: "approved" | "rejected") => {
        if (!active) return;
        try {
            await reviewMutation.mutateAsync({
                id: active.resource.id,
                data: {
                    status,
                    review_note:
                        status === "rejected" && rejectNote.trim()
                            ? rejectNote.trim()
                            : null,
                },
            });
            toast.success(
                status === "approved"
                    ? "Submission approved."
                    : "Submission rejected.",
            );
            invalidate();
            closeDialog();
        } catch (err) {
            toast.error(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to review submission",
            );
        }
    };

    const columns = useMemo<
        GridColDef<ResourceSubmissionResponse["resource"]>[]
    >(() => {
        const bySubmissionId = new Map(
            submissions.map((s) => [s.resource.id, s]),
        );
        return [
            {
                field: "created_at",
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
                field: "submitter",
                headerName: "Submitter",
                flex: 1,
                minWidth: 140,
                sortable: false,
                valueGetter: (_value, row) =>
                    bySubmissionId.get(row.id)?.submitter?.display_name ?? "—",
            },
            {
                field: "book",
                headerName: "Text",
                flex: 1.2,
                minWidth: 160,
                sortable: false,
                valueGetter: (_value, row) =>
                    bySubmissionId.get(row.id)?.book_title ?? "—",
            },
            {
                field: "resource_type",
                headerName: "Type",
                width: 110,
                sortable: false,
            },
            {
                field: "source",
                headerName: "Source",
                flex: 1.4,
                minWidth: 180,
                sortable: false,
                valueGetter: (_value, row) => row.source?.title ?? "—",
            },
            {
                field: "location",
                headerName: "Location",
                width: 110,
                sortable: false,
                valueGetter: (_value, row) => locationOf(row),
            },
            {
                field: "review_status",
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
        ];
    }, [submissions]);

    return (
        <div className="w-full max-w-6xl mx-auto px-8 py-12">
            <h1 className="text-2xl font-bold text-stone-900 mb-1">
                Resource submissions
            </h1>
            <p className="text-sm text-stone-500 mb-6">
                Community-suggested sources awaiting review. Click a row to
                review, edit, and approve or reject.
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
                    rows={rows}
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
                        const s = submissions.find(
                            (x) => x.resource.id === params.id,
                        );
                        if (s) {
                            setActive(s);
                            setRejectNote("");
                        }
                    }}
                    sx={{
                        border: "1px solid rgb(214 211 209)",
                        "& .MuiDataGrid-row": { cursor: "pointer" },
                    }}
                />
            </Box>

            {active && (
                <ReviewDialog
                    submission={active}
                    rejectNote={rejectNote}
                    onRejectNoteChange={setRejectNote}
                    pending={reviewMutation.isPending}
                    onApprove={() => review("approved")}
                    onReject={() => review("rejected")}
                    onEdit={() => setEditing(true)}
                    onClose={closeDialog}
                />
            )}

            {active && editing && (
                <ResourceFormModal
                    key={active.resource.id}
                    open
                    onClose={() => {
                        setEditing(false);
                        // The edit persisted to the resource row; refresh the
                        // queue so the dialog reflects it on next open.
                        invalidate();
                    }}
                    bookSlug={active.book_slug}
                    mode="edit"
                    initialData={active.resource}
                    isAdmin
                    isEditor
                />
            )}
        </div>
    );
}

interface ReviewDialogProps {
    submission: ResourceSubmissionResponse;
    rejectNote: string;
    onRejectNoteChange: (v: string) => void;
    pending: boolean;
    onApprove: () => void;
    onReject: () => void;
    onEdit: () => void;
    onClose: () => void;
}

function ReviewDialog({
    submission,
    rejectNote,
    onRejectNoteChange,
    pending,
    onApprove,
    onReject,
    onEdit,
    onClose,
}: ReviewDialogProps) {
    const r = submission.resource;
    const persons = r.source?.persons?.map((p) => p.name).join(", ");
    const anchor =
        r.anchor_sentence_end_number &&
        r.anchor_sentence_end_number !== r.anchor_sentence_start_number
            ? `${r.anchor_sentence_start_number}–${r.anchor_sentence_end_number}`
            : String(r.anchor_sentence_start_number);
    const isPending = r.review_status === "pending";

    return (
        <Dialog open onClose={onClose} maxWidth="sm" fullWidth>
            <DialogTitle sx={{ fontSize: 16 }}>Review submission</DialogTitle>
            <DialogContent
                sx={{ display: "flex", flexDirection: "column", gap: 1.5 }}
            >
                <Field label="Text">{submission.book_title}</Field>
                <Field label="Anchor">
                    {r.sentence_kind} sentence {anchor}
                </Field>
                <Field label="Type">
                    {r.resource_type}
                    {r.verbatim_kind ? ` (${r.verbatim_kind})` : ""}
                </Field>
                <Field label="Applies to">
                    {r.scope === "work"
                        ? "The work (all editions)"
                        : r.scope === "language"
                          ? "This language's editions"
                          : "This edition only"}
                </Field>
                <Field label="Source">
                    {r.source ? (
                        <>
                            {r.source.title}
                            {r.source.publication_year
                                ? ` (${r.source.publication_year})`
                                : ""}
                            {persons ? ` — ${persons}` : ""}
                        </>
                    ) : (
                        "—"
                    )}
                </Field>
                <Field label="Location">{locationOf(r)}</Field>
                {r.quoted_text && (
                    <Field label="Quoted text">{r.quoted_text}</Field>
                )}
                {r.editor_note && <Field label="Note">{r.editor_note}</Field>}
                <Field label="Submitter">
                    {submission.submitter?.display_name ?? "—"}
                </Field>

                {isPending && (
                    <TextField
                        label="Rejection note (optional)"
                        value={rejectNote}
                        onChange={(e) => onRejectNoteChange(e.target.value)}
                        size="small"
                        multiline
                        rows={2}
                        fullWidth
                        sx={{ mt: 1 }}
                    />
                )}
                {!isPending && submission.review_note && (
                    <Field label="Review note">{submission.review_note}</Field>
                )}
            </DialogContent>
            <DialogActions sx={{ px: 3, pb: 2 }}>
                <Button onClick={onEdit} size="small" disabled={pending}>
                    Edit
                </Button>
                <Box sx={{ flex: 1 }} />
                <Button onClick={onClose} size="small" disabled={pending}>
                    Close
                </Button>
                {isPending && (
                    <>
                        <Button
                            onClick={onReject}
                            size="small"
                            color="error"
                            disabled={pending}
                        >
                            Reject
                        </Button>
                        <Button
                            onClick={onApprove}
                            size="small"
                            variant="contained"
                            disabled={pending}
                        >
                            Approve
                        </Button>
                    </>
                )}
            </DialogActions>
        </Dialog>
    );
}

function Field({
    label,
    children,
}: {
    label: string;
    children: React.ReactNode;
}) {
    return (
        <div>
            <div className="text-xs uppercase tracking-wide text-stone-400">
                {label}
            </div>
            <div className="text-sm text-stone-800 whitespace-pre-wrap">
                {children}
            </div>
        </div>
    );
}
