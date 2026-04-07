import Avatar from "@mui/material/Avatar";
import Divider from "@mui/material/Divider";
import Fade from "@mui/material/Fade";
import IconButton from "@mui/material/IconButton";
import ListItemText from "@mui/material/ListItemText";
import Menu from "@mui/material/Menu";
import MenuItem from "@mui/material/MenuItem";
import Typography from "@mui/material/Typography";
import { useQueryClient } from "@tanstack/react-query";
import { Link, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { getMeQueryKey, useLogout } from "../api/auth/auth";
import { useAuth } from "../hooks/useAuth";

export function Navbar() {
    const navigate = useNavigate();
    const queryClient = useQueryClient();
    const { user, isLoading } = useAuth();
    const logoutMutation = useLogout();
    const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);

    const handleLogout = async () => {
        setAnchorEl(null);
        await logoutMutation.mutateAsync();
        queryClient.removeQueries({ queryKey: getMeQueryKey() });
    };

    return (
        <nav className="fixed top-0 left-0 right-0 z-50 h-12 flex items-center justify-between px-4 bg-white/80 backdrop-blur border-b border-stone-200">
            <Link to="/" className="font-bold text-stone-900 text-sm">
                Prospero
            </Link>

            <div className="w-28 flex justify-end">
                <Fade in={!isLoading} timeout={300}>
                    <div>
                        {user ? (
                            <>
                                <IconButton
                                    onClick={(e) =>
                                        setAnchorEl(e.currentTarget)
                                    }
                                    size="small"
                                >
                                    <Avatar
                                        src={user.avatar_url ?? undefined}
                                        alt={user.display_name}
                                        sx={{
                                            width: 28,
                                            height: 28,
                                            fontSize: 14,
                                        }}
                                    >
                                        {user.display_name
                                            .charAt(0)
                                            .toUpperCase()}
                                    </Avatar>
                                </IconButton>
                                <Menu
                                    anchorEl={anchorEl}
                                    open={Boolean(anchorEl)}
                                    onClose={() => setAnchorEl(null)}
                                    transformOrigin={{
                                        horizontal: "right",
                                        vertical: "top",
                                    }}
                                    anchorOrigin={{
                                        horizontal: "right",
                                        vertical: "bottom",
                                    }}
                                    slotProps={{
                                        paper: { sx: { minWidth: 200 } },
                                    }}
                                >
                                    <MenuItem disabled>
                                        <ListItemText
                                            primary={user.display_name}
                                            secondary={user.email}
                                            slotProps={{
                                                primary: {
                                                    variant: "body2",
                                                    fontWeight: 500,
                                                },
                                                secondary: {
                                                    variant: "caption",
                                                },
                                            }}
                                        />
                                    </MenuItem>
                                    <Divider />
                                    <MenuItem
                                        onClick={() => {
                                            setAnchorEl(null);
                                            navigate({ to: "/user/profile" });
                                        }}
                                    >
                                        <Typography variant="body2">
                                            Profile
                                        </Typography>
                                    </MenuItem>
                                    <MenuItem
                                        onClick={() => {
                                            setAnchorEl(null);
                                            navigate({ to: "/user/quotations" });
                                        }}
                                    >
                                        <Typography variant="body2">
                                            My Quotations
                                        </Typography>
                                    </MenuItem>
                                    <MenuItem
                                        onClick={() => {
                                            setAnchorEl(null);
                                            navigate({ to: "/user/notes" });
                                        }}
                                    >
                                        <Typography variant="body2">
                                            My Notes
                                        </Typography>
                                    </MenuItem>
                                    <MenuItem
                                        onClick={() => {
                                            setAnchorEl(null);
                                            navigate({ to: "/user/articles" });
                                        }}
                                    >
                                        <Typography variant="body2">
                                            My Articles
                                        </Typography>
                                    </MenuItem>
                                    <Divider />
                                    <MenuItem onClick={handleLogout}>
                                        <Typography variant="body2">
                                            Log out
                                        </Typography>
                                    </MenuItem>
                                </Menu>
                            </>
                        ) : (
                            <Link
                                to="/login"
                                className="text-sm px-3 py-1.5 rounded bg-stone-800 text-white hover:bg-stone-700 transition-colors"
                            >
                                Log in
                            </Link>
                        )}
                    </div>
                </Fade>
            </div>
        </nav>
    );
}
