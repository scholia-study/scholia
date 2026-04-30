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
import { useFeedback } from "../modules/feedback";

export function Navbar() {
    const navigate = useNavigate();
    const queryClient = useQueryClient();
    const { user, isLoading, hasPermission } = useAuth();
    const logoutMutation = useLogout();
    const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
    const { openModal: openFeedbackModal } = useFeedback();

    const handleLogout = async () => {
        setAnchorEl(null);
        await logoutMutation.mutateAsync();
        queryClient.removeQueries({ queryKey: getMeQueryKey() });
    };

    return (
        <nav className="shrink-0 h-12 flex items-center justify-between px-4 bg-white border-b border-stone-200">
            <div className="flex items-center gap-6">
                <Link
                    to="/"
                    className="flex items-center gap-2 font-bold text-stone-900 text-sm"
                >
                    <img
                        src="/images/scholia_book.webp"
                        alt="Scholia"
                        className="h-7 w-7 shrink-0 object-contain"
                    />
                    <span className="hidden min-[400px]:inline">Scholia</span>
                </Link>
                <div className="flex items-center gap-1">
                    <Link
                        to="/"
                        className="text-sm px-3 py-1 rounded transition-colors text-stone-500 no-underline hover:text-stone-900 hover:underline"
                        activeProps={{
                            className:
                                "text-sm px-3 py-1 rounded transition-colors text-stone-900 font-medium",
                        }}
                        activeOptions={{ exact: true }}
                    >
                        Texts
                    </Link>
                    <Link
                        to="/articles"
                        className="text-sm px-3 py-1 rounded transition-colors text-stone-500 no-underline hover:text-stone-900 hover:underline"
                        activeProps={{
                            className:
                                "text-sm px-3 py-1 rounded transition-colors text-stone-900 font-medium",
                        }}
                    >
                        Articles
                    </Link>
                </div>
            </div>

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
                                    slots={{ transition: Fade }}
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
                                            navigate({
                                                to: "/user/quotations",
                                            });
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
                                    <MenuItem
                                        onClick={() => {
                                            setAnchorEl(null);
                                            navigate({ to: "/user/sources" });
                                        }}
                                    >
                                        <Typography variant="body2">
                                            Sources
                                        </Typography>
                                    </MenuItem>
                                    <Divider />
                                    <MenuItem
                                        onClick={() => {
                                            setAnchorEl(null);
                                            openFeedbackModal();
                                        }}
                                    >
                                        <Typography variant="body2">
                                            Send feedback
                                        </Typography>
                                    </MenuItem>
                                    {hasPermission("admin_panel") && (
                                        <MenuItem
                                            onClick={() => {
                                                setAnchorEl(null);
                                                navigate({
                                                    to: "/admin/feedback",
                                                });
                                            }}
                                        >
                                            <Typography variant="body2">
                                                Admin: Feedback
                                            </Typography>
                                        </MenuItem>
                                    )}
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
