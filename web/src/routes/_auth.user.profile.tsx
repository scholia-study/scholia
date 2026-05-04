import Avatar from "@mui/material/Avatar";
import Button from "@mui/material/Button";
import Chip from "@mui/material/Chip";
import TextField from "@mui/material/TextField";
import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute, Link, redirect } from "@tanstack/react-router";
import { useState } from "react";
import toast from "react-hot-toast";
import {
    getMeQueryKey,
    useGetProfileSuspense,
    useRequestPasswordChange,
    useUpdateProfile,
} from "../api/auth/auth";
import { FetchError } from "../api/fetcher";

export const Route = createFileRoute("/_auth/user/profile")({
    component: ProfilePage,
});

const HANDLE_COOLDOWN_DAYS = 30;

function daysUntilHandleEditable(handleChangedAt: string | null | undefined) {
    if (!handleChangedAt) return 0;
    const last = new Date(handleChangedAt).getTime();
    if (Number.isNaN(last)) return 0;
    const elapsedDays = (Date.now() - last) / 86_400_000;
    return Math.max(0, Math.ceil(HANDLE_COOLDOWN_DAYS - elapsedDays));
}

function ProfilePage() {
    const queryClient = useQueryClient();
    const { data: profileData } = useGetProfileSuspense();

    if (profileData.status !== 200) {
        throw redirect({ to: "/login" });
    }

    const profile = profileData.data;

    const [displayName, setDisplayName] = useState(profile.display_name);
    const [sortName, setSortName] = useState(profile.sort_name ?? "");
    const [handle, setHandle] = useState(profile.handle ?? "");
    const [bio, setBio] = useState(profile.bio ?? "");
    const [title, setTitle] = useState(profile.title ?? "");
    const [location, setLocation] = useState(profile.location ?? "");
    const [websiteUrl, setWebsiteUrl] = useState(profile.website_url ?? "");

    const handleCooldownDays = daysUntilHandleEditable(
        profile.handle_changed_at,
    );
    const handleLocked = handleCooldownDays > 0;

    const updateMutation = useUpdateProfile();
    const passwordChangeMutation = useRequestPasswordChange();

    const handleSave = async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            await updateMutation.mutateAsync({
                data: {
                    display_name: displayName,
                    sort_name: sortName,
                    handle,
                    bio,
                    title,
                    location,
                    website_url: websiteUrl,
                },
            });
            toast.success("Profile updated.");
            queryClient.invalidateQueries({ queryKey: getMeQueryKey() });
        } catch (err) {
            if (err instanceof FetchError) {
                toast.error(err.message);
            } else {
                toast.error("Something went wrong.");
            }
        }
    };

    const dirty =
        displayName.trim() !== profile.display_name ||
        sortName.trim() !== (profile.sort_name ?? "") ||
        handle.trim() !== (profile.handle ?? "") ||
        bio !== (profile.bio ?? "") ||
        title !== (profile.title ?? "") ||
        location !== (profile.location ?? "") ||
        websiteUrl !== (profile.website_url ?? "");

    const handlePasswordChange = async () => {
        try {
            await passwordChangeMutation.mutateAsync();
            toast.success("Password change link sent to your email.");
        } catch {
            toast.error("Failed to send password change email.");
        }
    };

    return (
        <div className="max-w-md mx-auto px-8 py-16">
            <div className="flex items-baseline justify-between mb-8">
                <h1 className="text-2xl font-bold text-stone-900">Profile</h1>
                {profile.handle ? (
                    <Link
                        to="/users/$handle"
                        params={{ handle: profile.handle }}
                        className="text-sm text-stone-500 hover:text-stone-900 underline"
                    >
                        View public →
                    </Link>
                ) : null}
            </div>

            <div className="mb-8">
                <div className="flex items-center gap-4">
                    <Avatar
                        src={profile.avatar_url ?? undefined}
                        alt={profile.display_name}
                        sx={{ width: 56, height: 56, fontSize: 24 }}
                    >
                        {profile.display_name.charAt(0).toUpperCase()}
                    </Avatar>
                    <div>
                        <p className="font-medium text-stone-900">
                            {profile.display_name}
                        </p>
                        <p className="text-sm text-stone-500">
                            {profile.email}
                        </p>
                    </div>
                </div>
                <div className="flex flex-wrap gap-1.5 mt-3">
                    {profile.roles.map((role) => (
                        <Chip
                            key={role}
                            label={role}
                            size="small"
                            color="primary"
                            variant="outlined"
                        />
                    ))}
                </div>
            </div>

            <form onSubmit={handleSave} className="flex flex-col gap-3 mb-8">
                <TextField
                    label="Display name"
                    value={displayName}
                    onChange={(e) => setDisplayName(e.target.value)}
                    fullWidth
                    size="small"
                    required
                />
                <TextField
                    label="Sort name"
                    value={sortName}
                    onChange={(e) => setSortName(e.target.value)}
                    fullWidth
                    size="small"
                    placeholder="Niklas, Filip"
                    helperText="Used in bibliographies. Leave blank to auto-derive from display name."
                />
                <TextField
                    label="Handle"
                    value={handle}
                    onChange={(e) => setHandle(e.target.value)}
                    fullWidth
                    size="small"
                    disabled={handleLocked}
                    placeholder="filip-niklas"
                    helperText={
                        handleLocked
                            ? `Public URL identifier. Locked for ${handleCooldownDays} more day(s) since the last change.`
                            : "Public URL identifier — your profile lives at /users/<handle>. Lowercase, digits, and hyphens. Once changed, locked for 30 days."
                    }
                />
                <TextField
                    label="Title"
                    value={title}
                    onChange={(e) => setTitle(e.target.value)}
                    fullWidth
                    size="small"
                    placeholder="e.g. Independent scholar"
                />
                <TextField
                    label="Location"
                    value={location}
                    onChange={(e) => setLocation(e.target.value)}
                    fullWidth
                    size="small"
                    placeholder="e.g. Oslo, Norway"
                />
                <TextField
                    label="Website"
                    value={websiteUrl}
                    onChange={(e) => setWebsiteUrl(e.target.value)}
                    fullWidth
                    size="small"
                    placeholder="https://example.com"
                />
                <TextField
                    label="Bio"
                    value={bio}
                    onChange={(e) => setBio(e.target.value)}
                    fullWidth
                    size="small"
                    multiline
                    minRows={3}
                    maxRows={10}
                    helperText={`${bio.length}/500`}
                />
                <Button
                    type="submit"
                    variant="contained"
                    disabled={updateMutation.isPending || !dirty}
                    sx={{
                        alignSelf: "flex-end",
                        mt: 2,
                        textTransform: "none",
                    }}
                >
                    {updateMutation.isPending ? "Saving..." : "Save"}
                </Button>
            </form>

            <div className="border-t border-stone-200 pt-6 mb-8">
                <h2 className="text-sm font-semibold text-stone-700 mb-3">
                    Linked accounts
                </h2>
                {profile.providers.length > 0 ? (
                    <div className="flex flex-wrap gap-2">
                        {profile.providers.map((p) => (
                            <Chip
                                key={p.provider}
                                label={`${p.provider}${p.email ? ` (${p.email})` : ""}`}
                                variant="outlined"
                                size="small"
                            />
                        ))}
                    </div>
                ) : (
                    <p className="text-sm text-stone-500">
                        No linked accounts.
                    </p>
                )}
            </div>

            {profile.has_password && (
                <div className="border-t border-stone-200 pt-6">
                    <h2 className="text-sm font-semibold text-stone-700 mb-3">
                        Password
                    </h2>
                    <Button
                        variant="outlined"
                        onClick={handlePasswordChange}
                        disabled={passwordChangeMutation.isPending}
                        sx={{ textTransform: "none" }}
                    >
                        {passwordChangeMutation.isPending
                            ? "Sending..."
                            : "Change password"}
                    </Button>
                </div>
            )}
        </div>
    );
}
