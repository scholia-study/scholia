import { Box, Typography } from "@mui/material";

/**
 * Canonical copy for what happens if a member cancels: nothing is
 * deleted; only new operations revert to free-tier limits. Exported
 * so other surfaces (cancellation confirmation modals, FAQ, emails)
 * can reuse the text without restyling the component.
 */
export const CANCELLATION_COPY =
    "You can cancel any time without losing anything. Existing notes, quotations, and articles stay yours. Only new additions and edits will revert to free-tier limits.";

interface CancellationNoteProps {
    /**
     * Layout classes (margin, width). Internal padding/border/typography
     * is owned by the component.
     */
    className?: string;
}

/**
 * Default-styled reassurance box wrapping CANCELLATION_COPY. Used on
 * the welcome page and the membership page when the user is subbed.
 */
export function CancellationNote({ className = "" }: CancellationNoteProps) {
    return (
        <Box
            className={`p-4 border border-stone-200 rounded text-left ${className}`}
        >
            <Typography variant="body2" className="!text-stone-600">
                {CANCELLATION_COPY}
            </Typography>
        </Box>
    );
}
