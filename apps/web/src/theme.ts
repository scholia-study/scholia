import Fade from "@mui/material/Fade";
import { createTheme } from "@mui/material/styles";

// Global MUI theme.
// Tooltip:
//  - Use Fade instead of the default Grow. Grow's `transform: scale()` causes
//    a subpixel settle shift when the transition finishes; Fade is opacity-only.
//  - Disable Popper's GPU acceleration so it positions via integer top/left
//    instead of translate3d. See mui/material-ui#39064.
export const theme = createTheme({
    components: {
        MuiChip: {
            styleOverrides: {
                root: {
                    "& .MuiChip-label:empty": { paddingLeft: 0 },
                },
            },
        },
        MuiTooltip: {
            defaultProps: {
                TransitionComponent: Fade,
                slotProps: {
                    popper: {
                        modifiers: [
                            {
                                name: "computeStyles",
                                options: { gpuAcceleration: false },
                            },
                        ],
                    },
                },
            },
        },
    },
});
