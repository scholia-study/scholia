import { Chip } from "@mui/material";

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
 * the role.
 */
const STYLES: Record<
    string,
    {
        label: string;
        color: "default" | "primary" | "secondary" | "warning";
        sx?: object;
    }
> = {
    editor: {
        label: "Editor",
        color: "secondary",
    },
    scholiast: {
        label: "Scholiast",
        color: "default",
    },
    scholiast_benefactor: {
        label: "Benefactor",
        color: "primary",
    },
    scholiast_patron: {
        label: "Patron",
        color: "warning",
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
                        sx={{
                            height: size === "small" ? 20 : 24,
                            fontSize: size === "small" ? "0.65rem" : "0.75rem",
                            ...style.sx,
                        }}
                    />
                );
            })}
        </span>
    );
}
