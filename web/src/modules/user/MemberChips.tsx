import HistoryEduIcon from "@mui/icons-material/HistoryEdu";
import StarBorderPurple500Icon from "@mui/icons-material/StarBorderPurple500";
import StarsIcon from "@mui/icons-material/Stars";
import WorkspacePremiumIcon from "@mui/icons-material/WorkspacePremium";
import { Chip } from "@mui/material";
import type { ReactElement } from "react";

interface MemberChipsProps {
    /**
     * Public-visible role names returned by the backend (already
     * filtered to exclude `admin` and `user`). See
     * `auth/permissions.rs::filter_public_roles`.
     */
    roles: string[];
    size?: "small" | "medium";
}

/**
 * Per-role visual style. Tier ordering (visual weight) follows the
 * patronage hierarchy: scholiast → benefactor → patron, with `editor`
 * styled distinctly to signal the operational, not patronage, nature of
 * the role. Each paid tier carries a dedicated icon to make the rank
 * recognizable at a glance.
 */
const STYLES: Record<
    string,
    {
        label: string;
        color: "default" | "primary" | "secondary" | "warning";
        icon?: ReactElement;
        sx?: object;
    }
> = {
    editor: {
        label: "Editor",
        color: "secondary",
        icon: <HistoryEduIcon />,
    },
    scholiast: {
        label: "Scholiast",
        color: "default",
        icon: <StarBorderPurple500Icon />,
    },
    scholiast_benefactor: {
        label: "Scholiast Benefactor",
        color: "primary",
        icon: <StarsIcon />,
    },
    scholiast_patron: {
        label: "Scholiast Patron",
        color: "warning",
        icon: <WorkspacePremiumIcon />,
    },
};

export function MemberChips({ roles, size = "small" }: MemberChipsProps) {
    if (!roles || roles.length === 0) return null;
    return (
        <span className="inline-flex flex-wrap gap-1">
            {roles.map((role) => {
                const style = STYLES[role];
                if (!style) return null;
                return (
                    <Chip
                        key={role}
                        label={style.label}
                        size={size}
                        color={style.color}
                        variant="outlined"
                        icon={style.icon}
                        sx={{
                            height: size === "small" ? 20 : 24,
                            fontSize: size === "small" ? "0.65rem" : "0.75rem",
                            // Scale the icon so it doesn't dwarf the
                            // small chip; MUI's default chip-icon size
                            // assumes a larger chip.
                            "& .MuiChip-icon": {
                                fontSize: size === "small" ? "0.85rem" : "1rem",
                            },
                            ...style.sx,
                        }}
                    />
                );
            })}
        </span>
    );
}
