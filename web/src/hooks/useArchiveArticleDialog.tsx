import {
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogContentText,
    DialogTitle,
} from "@mui/material";
import { useCallback, useState } from "react";

interface UseArchiveArticleDialogOptions {
    /** Called when the user confirms; receives the slug passed to `openFor`. */
    onConfirm: (slug: string) => void | Promise<void>;
    isPending?: boolean;
}

interface UseArchiveArticleDialogResult {
    /** Open the dialog for a specific article slug. */
    openFor: (slug: string) => void;
    /** The dialog element to render somewhere in the tree. */
    dialog: React.ReactElement;
}

/**
 * Shared confirmation dialog for archiving an article. Used by both the
 * article list and the article editor — keeps copy and styling in one place.
 */
export function useArchiveArticleDialog({
    onConfirm,
    isPending,
}: UseArchiveArticleDialogOptions): UseArchiveArticleDialogResult {
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
            <DialogTitle>Archive this article?</DialogTitle>
            <DialogContent>
                <DialogContentText sx={{ fontSize: "0.875rem", mb: 1.5 }}>
                    Archiving is <strong>irreversible</strong>. Remember that
                    you can <i>keep editing</i> the article as long as you like.
                    Once archived:
                </DialogContentText>
                <ul className="text-sm text-stone-600 list-disc pl-5 space-y-1">
                    <li>The article is removed from public listings</li>
                    <li>
                        It remains accessible via its direct link, so historical
                        references keep working
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
                    color="warning"
                    disabled={isPending}
                >
                    Archive
                </Button>
            </DialogActions>
        </Dialog>
    );

    return { openFor, dialog };
}
