import EditOutlined from "@mui/icons-material/EditOutlined";
import {
    Button,
    Checkbox,
    Chip,
    CircularProgress,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    FormControlLabel,
    Tooltip,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import toast from "react-hot-toast";
import {
    getGetPublishedArticleQueryKey,
    getListPublishedArticlesQueryKey,
    useApplyArticleLabel,
    useListEditorialLabels,
    useRemoveArticleLabel,
} from "#/api/articles/articles";
import { FetchError } from "#/api/fetcher";
import type { EditorialLabelResponse } from "#/api/model";

interface EditorialLabelManagerProps {
    articleSlug: string;
    /** Currently-applied labels on the article (from the article response). */
    appliedLabels: EditorialLabelResponse[];
}

/**
 * Editor-only "Manage labels" affordance. Renders a small button that
 * opens a modal listing every available editorial label with a checkbox.
 * Diffs the user's selection against the originally-applied set on save
 * and issues per-label apply/remove mutations.
 *
 * Caller is responsible for gating visibility via permission
 * (`article_labels_manage`); the component itself assumes the user has
 * the right to click it.
 */
export function EditorialLabelManager({
    articleSlug,
    appliedLabels,
}: EditorialLabelManagerProps) {
    const [open, setOpen] = useState(false);
    const queryClient = useQueryClient();

    const { data: labelsData } = useListEditorialLabels({
        query: { enabled: open },
    });
    const allLabels = labelsData?.data?.labels ?? [];

    const appliedSlugs = useMemo(
        () => new Set(appliedLabels.map((l) => l.slug)),
        [appliedLabels],
    );
    const [selected, setSelected] = useState<Set<string>>(new Set());

    const applyMutation = useApplyArticleLabel();
    const removeMutation = useRemoveArticleLabel();

    const handleOpen = () => {
        setSelected(new Set(appliedSlugs));
        setOpen(true);
    };

    const handleClose = () => setOpen(false);

    const handleToggle = (slug: string) => {
        setSelected((prev) => {
            const next = new Set(prev);
            if (next.has(slug)) next.delete(slug);
            else next.add(slug);
            return next;
        });
    };

    const handleSave = async () => {
        const toApply = [...selected].filter((s) => !appliedSlugs.has(s));
        const toRemove = [...appliedSlugs].filter((s) => !selected.has(s));

        try {
            for (const slug of toApply) {
                await applyMutation.mutateAsync({
                    slug: articleSlug,
                    data: { label_slug: slug },
                });
            }
            for (const slug of toRemove) {
                await removeMutation.mutateAsync({
                    slug: articleSlug,
                    labelSlug: slug,
                });
            }
            await queryClient.invalidateQueries({
                queryKey: getGetPublishedArticleQueryKey(articleSlug),
            });
            await queryClient.invalidateQueries({
                queryKey: getListPublishedArticlesQueryKey(),
            });
            setOpen(false);
            const changed = toApply.length + toRemove.length;
            if (changed > 0) {
                toast.success(
                    changed === 1
                        ? "Label updated."
                        : `${changed} labels updated.`,
                );
            }
        } catch (err) {
            const message =
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to update labels.";
            toast.error(message);
        }
    };

    const isPending = applyMutation.isPending || removeMutation.isPending;

    return (
        <>
            <Tooltip title="Manage editorial labels">
                <Chip
                    icon={<EditOutlined sx={{ fontSize: "0.9rem" }} />}
                    label="Manage"
                    size="small"
                    variant="outlined"
                    onClick={handleOpen}
                    sx={{
                        fontSize: "0.7rem",
                        borderStyle: "dashed",
                        cursor: "pointer",
                    }}
                />
            </Tooltip>
            <Dialog open={open} onClose={handleClose} maxWidth="xs" fullWidth>
                <DialogTitle>Editorial labels</DialogTitle>
                <DialogContent>
                    {allLabels.length === 0 && (
                        <p className="text-sm text-stone-400">
                            Loading labels...
                        </p>
                    )}
                    <div className="flex flex-col">
                        {allLabels.map((l) => (
                            <FormControlLabel
                                key={l.id}
                                control={
                                    <Checkbox
                                        checked={selected.has(l.slug)}
                                        onChange={() => handleToggle(l.slug)}
                                        size="small"
                                    />
                                }
                                label={
                                    <div>
                                        <div className="text-sm text-stone-900">
                                            {l.name}
                                        </div>
                                    </div>
                                }
                            />
                        ))}
                    </div>
                </DialogContent>
                <DialogActions sx={{ px: 3, pb: 2 }}>
                    <Button onClick={handleClose} size="small">
                        Cancel
                    </Button>
                    <Button
                        onClick={handleSave}
                        size="small"
                        variant="contained"
                        disabled={isPending}
                        startIcon={
                            isPending ? (
                                <CircularProgress size={14} />
                            ) : undefined
                        }
                    >
                        Save
                    </Button>
                </DialogActions>
            </Dialog>
        </>
    );
}
