import {
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogContentText,
    DialogTitle,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useRef, useState } from "react";
import toast from "react-hot-toast";
import {
    getListAllNotesQueryKey,
    getListAllQuotationsQueryKey,
    useDeleteQuotation,
} from "../../../api/quotations/quotations";
import { invalidateAllNodeQuotations } from "./invalidateQuotations";

interface QuotationLike {
    id: string;
    note_count: number;
    book_slug?: string | null;
}

interface UseUnsaveQuotationOptions {
    bookSlug?: string;
    onSuccess?: () => void;
}

export function useUnsaveQuotation({
    bookSlug,
    onSuccess,
}: UseUnsaveQuotationOptions) {
    const queryClient = useQueryClient();
    const [target, setTarget] = useState<QuotationLike | null>(null);
    const slugRef = useRef<string>("");

    const deleteMutation = useDeleteQuotation({
        mutation: {
            onSuccess: () => {
                toast.success("Quotation removed");
                invalidateAllNodeQuotations(queryClient);
                queryClient.invalidateQueries({
                    queryKey: getListAllQuotationsQueryKey(),
                });
                queryClient.invalidateQueries({
                    queryKey: getListAllNotesQueryKey(),
                });
                onSuccess?.();
            },
            onError: () => toast.error("Failed to remove quotation"),
        },
    });

    const resolveSlug = useCallback(
        (quotation: QuotationLike) => quotation.book_slug ?? bookSlug ?? "",
        [bookSlug],
    );

    const requestUnsave = useCallback(
        (quotation: QuotationLike) => {
            const slug = resolveSlug(quotation);
            slugRef.current = slug;
            if (quotation.note_count > 0) {
                setTarget(quotation);
            } else {
                deleteMutation.mutate({ slug, id: quotation.id });
            }
        },
        [resolveSlug, deleteMutation],
    );

    const confirmUnsave = useCallback(() => {
        if (target) {
            deleteMutation.mutate({ slug: slugRef.current, id: target.id });
            setTarget(null);
        }
    }, [target, deleteMutation]);

    const UnsaveDialog = (
        <Dialog open={target != null} onClose={() => setTarget(null)}>
            <DialogTitle sx={{ fontSize: "0.95rem" }}>
                Remove saved quotation?
            </DialogTitle>
            <DialogContent>
                <DialogContentText sx={{ fontSize: "0.875rem" }}>
                    {target && target.note_count > 0
                        ? `This will permanently delete ${target.note_count} note${target.note_count > 1 ? "s" : ""} attached to this quotation.`
                        : "This will remove the saved quotation."}
                </DialogContentText>
            </DialogContent>
            <DialogActions sx={{ px: 3, pb: 2 }}>
                <Button onClick={() => setTarget(null)} size="small">
                    Cancel
                </Button>
                <Button
                    onClick={confirmUnsave}
                    size="small"
                    color="error"
                    variant="contained"
                >
                    Remove
                </Button>
            </DialogActions>
        </Dialog>
    );

    return { requestUnsave, UnsaveDialog, isPending: deleteMutation.isPending };
}
