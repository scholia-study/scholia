import Avatar from "@mui/material/Avatar";
import { createFileRoute, notFound } from "@tanstack/react-router";
import { FetchError } from "../api/fetcher";
import {
    getGetPublicProfileSuspenseQueryOptions,
    useGetPublicProfileSuspense,
} from "../api/users/users";
import { ArticleCard } from "../modules/article";
import {
    metaDescription,
    profileJsonLd,
    SEO_COPY,
    seoHead,
} from "../modules/seo";
import { MemberChips } from "../modules/user";

export const Route = createFileRoute("/users/$handle")({
    loader: async ({ context, params }) => {
        try {
            const res = await context.queryClient.ensureQueryData(
                getGetPublicProfileSuspenseQueryOptions(params.handle, {}),
            );
            const profile = res.data;
            return {
                displayName: profile.display_name,
                bio: profile.bio ?? null,
                hasPublishedArticles: profile.article_total > 0,
            };
        } catch (err) {
            if (err instanceof FetchError && err.status === 404) {
                throw notFound();
            }
            throw err;
        }
    },
    head: ({ loaderData, params }) => {
        if (!loaderData) return {};
        const path = `/users/${params.handle}`;
        return seoHead({
            title: SEO_COPY.profile.title(
                loaderData.displayName,
                params.handle,
            ),
            description: metaDescription(
                loaderData.bio ??
                    SEO_COPY.profile.description(loaderData.displayName),
            ),
            path,
            ogType: "profile",
            // Thin-content guard: only author profiles (≥1 published
            // article) are worth a search result.
            noindex: !loaderData.hasPublishedArticles,
            jsonLd: [
                profileJsonLd(
                    {
                        displayName: loaderData.displayName,
                        handle: params.handle,
                        bio: loaderData.bio,
                    },
                    path,
                ),
            ],
        });
    },
    component: PublicProfilePage,
    notFoundComponent: UserNotFound,
});

function UserNotFound() {
    const { handle } = Route.useParams();
    return (
        <div className="w-full max-w-3xl mx-auto px-8 py-16">
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

function PublicProfilePage() {
    const { handle } = Route.useParams();
    const { data } = useGetPublicProfileSuspense(handle, {});
    const profile = data.data;

    const memberSince = new Date(profile.created_at).toLocaleDateString(
        undefined,
        { month: "long", year: "numeric" },
    );

    return (
        <div className="w-full max-w-3xl mx-auto px-8 py-16">
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
                    <div className="space-y-4">
                        {profile.articles.map((a) => (
                            <ArticleCard
                                key={a.id}
                                article={a}
                                showAuthor={false}
                            />
                        ))}
                    </div>
                )}
            </section>
        </div>
    );
}
