import Avatar from "@mui/material/Avatar";
import Paper from "@mui/material/Paper";
import { createFileRoute, Link } from "@tanstack/react-router";
import { useGetPublicProfile } from "../api/users/users";
import { MemberChips } from "../modules/user";

export const Route = createFileRoute("/users/$handle")({
    component: PublicProfilePage,
});

function PublicProfilePage() {
    const { handle } = Route.useParams();
    const { data, isLoading, isError } = useGetPublicProfile(handle, {});
    const profile = data?.data;

    if (isLoading) {
        return (
            <div className="max-w-3xl mx-auto px-8 py-16">
                <p className="text-sm text-stone-400">Loading…</p>
            </div>
        );
    }

    if (isError || !profile) {
        return (
            <div className="max-w-3xl mx-auto px-8 py-16">
                <h1 className="text-2xl font-bold text-stone-900 mb-2">
                    User not found
                </h1>
                <p className="text-sm text-stone-500">
                    No user with the handle{" "}
                    <span className="font-mono">{handle}</span>.
                </p>
            </div>
        );
    }

    const memberSince = new Date(profile.created_at).toLocaleDateString(
        undefined,
        { month: "long", year: "numeric" },
    );

    return (
        <div className="max-w-3xl mx-auto px-8 py-16">
            <header className="mb-8">
                <div className="flex items-start gap-4">
                    <Avatar
                        src={profile.avatar_url ?? undefined}
                        alt={profile.display_name}
                        sx={{ width: 72, height: 72, fontSize: 32 }}
                    >
                        {profile.display_name.charAt(0).toUpperCase()}
                    </Avatar>
                    <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 flex-wrap">
                            <h1 className="text-2xl font-bold text-stone-900">
                                {profile.display_name}
                            </h1>
                            <MemberChips roles={profile.public_roles} />
                        </div>
                        <p className="text-sm text-stone-400 mt-0.5">
                            @{profile.handle}
                        </p>
                        {profile.title && (
                            <p className="text-sm text-stone-600 mt-1">
                                {profile.title}
                            </p>
                        )}
                        <div className="text-xs text-stone-400 mt-2 flex flex-wrap gap-x-3">
                            {profile.location && (
                                <span>{profile.location}</span>
                            )}
                            {profile.website_url && (
                                <a
                                    href={profile.website_url}
                                    target="_blank"
                                    rel="noreferrer"
                                    className="text-stone-500 hover:underline"
                                >
                                    {profile.website_url.replace(
                                        /^https?:\/\//,
                                        "",
                                    )}
                                </a>
                            )}
                            <span>Member since {memberSince}</span>
                        </div>
                    </div>
                </div>
                {profile.bio && (
                    <p className="mt-4 text-sm text-stone-700 whitespace-pre-wrap">
                        {profile.bio}
                    </p>
                )}
            </header>

            <section>
                <h2 className="text-sm font-semibold text-stone-700 mb-3">
                    Articles
                    {profile.article_total > 0 && (
                        <span className="text-stone-400 font-normal">
                            {" "}
                            ({profile.article_total})
                        </span>
                    )}
                </h2>

                {profile.articles.length === 0 ? (
                    <p className="text-sm text-stone-400">
                        No published articles yet.
                    </p>
                ) : (
                    <div className="space-y-2">
                        {profile.articles.map((a) => (
                            <Link
                                key={a.id}
                                to="/articles/$slug"
                                params={{ slug: a.slug }}
                                style={{
                                    textDecoration: "none",
                                    color: "inherit",
                                }}
                            >
                                <Paper
                                    elevation={0}
                                    sx={{
                                        border: "1px solid rgb(214 211 209)",
                                        p: 1.5,
                                        transition: "box-shadow 0.15s",
                                        "&:hover": { boxShadow: 3 },
                                    }}
                                >
                                    <p className="text-sm font-medium text-stone-900">
                                        {a.title}
                                    </p>
                                    {a.description && (
                                        <p className="text-xs text-stone-500 mt-0.5 line-clamp-2">
                                            {a.description}
                                        </p>
                                    )}
                                    {a.published_at && (
                                        <p className="text-[10px] text-stone-400 mt-1">
                                            {new Date(
                                                a.published_at,
                                            ).toLocaleDateString(undefined, {
                                                month: "short",
                                                day: "numeric",
                                                year: "numeric",
                                            })}
                                        </p>
                                    )}
                                </Paper>
                            </Link>
                        ))}
                    </div>
                )}
            </section>
        </div>
    );
}
