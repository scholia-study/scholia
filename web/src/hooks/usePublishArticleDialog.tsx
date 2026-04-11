import {
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogContentText,
    DialogTitle,
} from "@mui/material";
import { useCallback, useState } from "react";

interface UsePublishArticleDialogOptions {
    /** Called when the user confirms; receives the slug passed to `openFor`. */
    onConfirm: (slug: string) => void | Promise<void>;
    isPending?: boolean;
}

interface UsePublishArticleDialogResult {
    openFor: (slug: string) => void;
    dialog: React.ReactElement;
}

/**
 * Shared confirmation dialog for publishing an article. Used by both the
 * article list and the article editor.
 */
export function usePublishArticleDialog({
    onConfirm,
    isPending,
}: UsePublishArticleDialogOptions): UsePublishArticleDialogResult {
    const [slug, setSlug] = useState<string | null>(null);
    const close = useCallback(() => setSlug(null), []);
    const openFor = useCallback((next: string) => setSlug(next), []);

    const handleConfirm = async () => {
        if (!slug) return;
        const target = slug;
        setSlug(null);
        await onConfirm(target);
    };

    const dialog = (
        <Dialog open={slug != null} onClose={close} maxWidth="sm">
            <DialogTitle>Publish this article?</DialogTitle>
            <DialogContent>
                <DialogContentText sx={{ fontSize: "0.875rem", mb: 1.5 }}>
                    Once published, this article becomes public and cannot be
                    reverted to a draft. You can:
                </DialogContentText>
                <ul className="text-sm text-stone-600 list-disc pl-5 space-y-1">
                    <li>Continue editing the article at any time</li>
                    <li>
                        Archive it later, which removes it from listings but
                        keeps it accessible via direct link for historical
                        references
                    </li>
                </ul>
            </DialogContent>
            <DialogActions sx={{ px: 3, pb: 2 }}>
                <Button onClick={close} size="small">
                    Cancel
                </Button>
                <Button
                    onClick={handleConfirm}
                    size="small"
                    variant="contained"
                    disabled={isPending}
                >
                    Publish
                </Button>
            </DialogActions>
        </Dialog>
    );

    return { openFor, dialog };
}
