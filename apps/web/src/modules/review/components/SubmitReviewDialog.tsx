import {
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    FormControlLabel,
    Radio,
    RadioGroup,
    TextField,
} from "@mui/material";
import { useState } from "react";
import toast from "react-hot-toast";
import { useCreateReviewRequest } from "#/api/article-reviews/article-reviews";
import { FetchError } from "#/api/fetcher";

interface SubmitReviewDialogProps {
    open: boolean;
    onClose: () => void;
    articleSlug: string;
    articleStatus: string;
    /** Called with the new request id after a successful submission. */
    onSubmitted: (requestId: string) => void;
}

/**
 * Author-facing dialog to submit an article for editorial review, with
 * the intent choice: plain feedback, or a publication hand-off where an
 * approving editor publishes the draft and applies the Imprimatur label.
 */
export function SubmitReviewDialog({
    open,
    onClose,
    articleSlug,
    articleStatus,
    onSubmitted,
}: SubmitReviewDialogProps) {
    const [intent, setIntent] = useState<"feedback" | "publication">(
        "feedback",
    );
    const [message, setMessage] = useState("");
    const createMutation = useCreateReviewRequest();

    const isDraft = articleStatus === "draft";

    const submit = async () => {
        try {
            const result = await createMutation.mutateAsync({
                slug: articleSlug,
                data: {
                    intent,
                    message: message.trim() ? message.trim() : undefined,
                },
            });
            toast.success("Submitted for review.");
            onSubmitted(result.data.id);
        } catch (err) {
            toast.error(
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to submit for review",
            );
        }
    };

    return (
        <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
            <DialogTitle sx={{ fontSize: 16 }}>Request a review</DialogTitle>
            <DialogContent
                sx={{ display: "flex", flexDirection: "column", gap: 1.5 }}
            >
                <p className="text-sm text-stone-600">
                    An editor will read a snapshot of the article as it is right
                    now and respond with comments. Submitting shares the article
                    with the editorial team, even while it is a draft.
                </p>
                <RadioGroup
                    value={intent}
                    onChange={(e) =>
                        setIntent(e.target.value as "feedback" | "publication")
                    }
                >
                    <FormControlLabel
                        value="feedback"
                        control={<Radio size="small" />}
                        label={
                            <span className="text-sm">
                                <strong>Feedback</strong> — I'd like comments
                                and suggestions.
                            </span>
                        }
                    />
                    <FormControlLabel
                        value="publication"
                        control={<Radio size="small" />}
                        label={
                            <span className="text-sm">
                                <strong>Publication</strong> — if everything
                                looks good, approve it
                                {isDraft ? " and publish it for me" : ""}.
                            </span>
                        }
                    />
                </RadioGroup>
                {intent === "publication" && (
                    <p className="text-xs text-stone-500">
                        {isDraft
                            ? "If approved, the editor will publish this article on your behalf and apply the Imprimatur label. "
                            : "If approved, the editor will apply the Imprimatur label to the published article. "}
                        <em>Imprimatur</em> ("let it be printed") is Scholia's
                        editorial seal: a badge on the published article showing
                        it was reviewed and approved by an editor. It is revoked
                        automatically if you edit the article afterwards, and
                        you can resubmit to earn it back.
                    </p>
                )}
                <TextField
                    label="Message to the editors (optional)"
                    value={message}
                    onChange={(e) => setMessage(e.target.value)}
                    size="small"
                    multiline
                    rows={3}
                    fullWidth
                    sx={{ mt: 1 }}
                />
            </DialogContent>
            <DialogActions sx={{ px: 3, pb: 2 }}>
                <Button
                    onClick={onClose}
                    size="small"
                    disabled={createMutation.isPending}
                >
                    Cancel
                </Button>
                <Button
                    onClick={submit}
                    size="small"
                    variant="contained"
                    disabled={createMutation.isPending}
                >
                    Submit for review
                </Button>
            </DialogActions>
        </Dialog>
    );
}
