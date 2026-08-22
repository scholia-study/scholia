import {
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    FormControl,
    FormControlLabel,
    InputLabel,
    MenuItem,
    Radio,
    RadioGroup,
    Select,
    TextField,
} from "@mui/material";
import { useState } from "react";
import toast from "react-hot-toast";
import { useCreateReviewRequest } from "#/api/article-reviews/article-reviews";
import { useListMyCollegia } from "#/api/collegia/collegia";
import { FetchError } from "#/api/fetcher";

const EDITORS = "editors";

interface SubmitReviewDialogProps {
    open: boolean;
    onClose: () => void;
    articleSlug: string;
    articleStatus: string;
    /** Called with the new request id after a successful submission. */
    onSubmitted: (requestId: string) => void;
}

/**
 * Author-facing dialog to submit an article for review, choosing the
 * audience — the Scholia editors, or one of the author's collegia. Editor
 * review carries the intent choice (feedback or publication hand-off);
 * collegium review is feedback-only.
 */
export function SubmitReviewDialog({
    open,
    onClose,
    articleSlug,
    articleStatus,
    onSubmitted,
}: SubmitReviewDialogProps) {
    const [audience, setAudience] = useState<string>(EDITORS);
    const [intent, setIntent] = useState<"feedback" | "publication">(
        "feedback",
    );
    const [message, setMessage] = useState("");
    const createMutation = useCreateReviewRequest();
    // Membership can change from another client (a steward approving a
    // join request), so bypass the app-wide staleTime: refetch every
    // time the dialog opens.
    const { data: myCollegiaData } = useListMyCollegia({
        query: { enabled: open, staleTime: 0, refetchOnMount: "always" },
    });
    const myCollegia = myCollegiaData?.data?.collegia ?? [];

    const isDraft = articleStatus === "draft";
    const isCollegiumAudience = audience !== EDITORS;
    const effectiveIntent = isCollegiumAudience ? "feedback" : intent;
    const audienceCollegium = myCollegia.find((g) => g.id === audience);

    const submit = async () => {
        try {
            const result = await createMutation.mutateAsync({
                slug: articleSlug,
                data: {
                    intent: effectiveIntent,
                    collegium_id: isCollegiumAudience ? audience : undefined,
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
                {myCollegia.length > 0 && (
                    <FormControl size="small" sx={{ mt: 1 }}>
                        <InputLabel id="review-audience-label">
                            Reviewed by
                        </InputLabel>
                        <Select
                            labelId="review-audience-label"
                            label="Reviewed by"
                            value={audience}
                            onChange={(e) => setAudience(e.target.value)}
                        >
                            <MenuItem value={EDITORS}>
                                The Scholia editors
                            </MenuItem>
                            {myCollegia.map((collegium) => (
                                <MenuItem
                                    key={collegium.id}
                                    value={collegium.id}
                                >
                                    {collegium.name} (collegium)
                                </MenuItem>
                            ))}
                        </Select>
                    </FormControl>
                )}
                <p className="text-sm text-stone-600">
                    {isCollegiumAudience
                        ? audienceCollegium?.review_visibility === "stewards"
                            ? `The stewards of ${audienceCollegium?.name ?? "the collegium"} will read a snapshot of the article as it is right now and respond with comments. Only the collegium's stewards will see this submission, even while it is a draft — other members won't.`
                            : `The members of ${audienceCollegium?.name ?? "the collegium"} will read a snapshot of the article as it is right now and respond with comments. Submitting shares the article with the collegium, even while it is a draft.`
                        : "An editor will read a snapshot of the article as it is right now and respond with comments. Submitting shares the article with the editorial team, even while it is a draft."}
                </p>
                {!isCollegiumAudience && (
                    <RadioGroup
                        value={intent}
                        onChange={(e) =>
                            setIntent(
                                e.target.value as "feedback" | "publication",
                            )
                        }
                    >
                        <FormControlLabel
                            value="feedback"
                            control={<Radio size="small" />}
                            label={
                                <span className="text-sm">
                                    <strong>Feedback</strong> — I'd like
                                    comments and suggestions.
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
                )}
                {!isCollegiumAudience && intent === "publication" && (
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
                {isCollegiumAudience && (
                    <p className="text-xs text-stone-500">
                        {audienceCollegium?.review_visibility === "stewards"
                            ? "Collegium reviews are feedback-only: the collegium's stewards comment on the snapshot and close the round; you can withdraw your submission at any time. "
                            : "Collegium reviews are feedback-only: members comment on the snapshot, and you (or a steward) close the round when you have what you need. "}
                        Publishing and the Imprimatur label remain with the
                        Scholia editors.
                    </p>
                )}
                <TextField
                    label={
                        isCollegiumAudience
                            ? "Message to the collegium (optional)"
                            : "Message to the editors (optional)"
                    }
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
