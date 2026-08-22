import { FormControlLabel, Radio, RadioGroup } from "@mui/material";

export type ReviewVisibility = "members" | "stewards";

/**
 * The who-sees-submissions choice, shared by the create and settings
 * dialogs. Governs future submissions only — each review request keeps
 * the visibility it was submitted under.
 */
export function ReviewVisibilityField({
    value,
    onChange,
}: {
    value: ReviewVisibility;
    onChange: (value: ReviewVisibility) => void;
}) {
    return (
        <div>
            <div className="text-sm font-medium text-stone-700 mb-1">
                Who reviews submissions?
            </div>
            <RadioGroup
                value={value}
                onChange={(e) => onChange(e.target.value as ReviewVisibility)}
            >
                <FormControlLabel
                    value="members"
                    control={<Radio size="small" />}
                    label={
                        <span className="text-sm">
                            <strong>All members</strong> — everyone sees and
                            comments on each other's submissions (writing-circle
                            mode).
                        </span>
                    }
                />
                <FormControlLabel
                    value="stewards"
                    control={<Radio size="small" />}
                    label={
                        <span className="text-sm">
                            <strong>Stewards only</strong> — members see only
                            their own submissions; stewards see all (classroom
                            mode).
                        </span>
                    }
                />
            </RadioGroup>
        </div>
    );
}
