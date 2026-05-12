import {
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    TextField,
} from "@mui/material";
import { useLocation } from "@tanstack/react-router";
import toast from "react-hot-toast";
import { useCreateFeedback } from "../../api/feedback/feedback";
import { FetchError } from "../../api/fetcher";
import { useFeedback } from "./FeedbackProvider";

const MIN_LEN = 5;
const MAX_LEN = 5000;

export function FeedbackModal() {
    const { open, draft, setDraft, closeModal, clearDraft } = useFeedback();
    const location = useLocation();
    const mutation = useCreateFeedback();

    const trimmedLen = draft.trim().length;
    const tooShort = trimmedLen > 0 && trimmedLen < MIN_LEN;
    const tooLong = trimmedLen > MAX_LEN;
    const canSubmit = trimmedLen >= MIN_LEN && !tooLong && !mutation.isPending;

    const handleSubmit = async () => {
        if (!canSubmit) return;
        try {
            await mutation.mutateAsync({
                data: {
                    body: draft.trim(),
                    url:
                        typeof window !== "undefined"
                            ? window.location.href
                            : location.href,
                    user_agent:
                        typeof navigator !== "undefined"
                            ? navigator.userAgent
                            : undefined,
                    viewport_w:
                        typeof window !== "undefined"
                            ? window.innerWidth
                            : undefined,
                    viewport_h:
                        typeof window !== "undefined"
                            ? window.innerHeight
                            : undefined,
                },
            });
            toast.success("Thanks, we got it!");
            clearDraft();
            closeModal();
        } catch (err: unknown) {
            const message =
                err instanceof FetchError && err.message
                    ? err.message
                    : "Failed to send feedback";
            toast.error(message);
        }
    };

    return (
        <Dialog open={open} onClose={closeModal} maxWidth="sm" fullWidth>
            <DialogTitle sx={{ fontSize: "0.95rem", pb: 0.5 }}>
                Send feedback
            </DialogTitle>
            <DialogContent>
                <p className="text-xs text-stone-500 mb-3">
                    Bug reports, suggestions, questions — anything goes. We
                    automatically attach the page you're on so you don't have to
                    describe where to find it.
                </p>
                <TextField
                    autoFocus
                    multiline
                    minRows={5}
                    maxRows={15}
                    fullWidth
                    placeholder="What's on your mind?"
                    value={draft}
                    onChange={(e) => setDraft(e.target.value)}
                    error={tooShort || tooLong}
                    helperText={
                        tooShort
                            ? `At least ${MIN_LEN} characters.`
                            : tooLong
                              ? `At most ${MAX_LEN} characters.`
                              : `${trimmedLen}/${MAX_LEN}`
                    }
                />
            </DialogContent>
            <DialogActions sx={{ px: 3, pb: 2 }}>
                <Button onClick={closeModal} size="small">
                    Cancel
                </Button>
                <Button
                    onClick={handleSubmit}
                    variant="contained"
                    size="small"
                    disabled={!canSubmit}
                    sx={{ textTransform: "none" }}
                >
                    {mutation.isPending ? "Sending…" : "Send"}
                </Button>
            </DialogActions>
        </Dialog>
    );
}
