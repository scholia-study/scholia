import { createFileRoute, notFound, redirect } from "@tanstack/react-router";
import { FetchError } from "../api/fetcher";
import { getHandleById } from "../api/users/users";

/**
 * Durable redirect from a UUID-keyed profile URL to the user's current
 * handle URL. Mirrors `/articles/by-id/<id>` for articles. Resolved in
 * the loader so SSR answers with a real 301 (crawlers and link equity
 * follow it) instead of rendering a shell.
 */
export const Route = createFileRoute("/users/by-id/$id")({
    loader: async ({ params }) => {
        let handle: string | undefined;
        try {
            const res = await getHandleById(params.id);
            if (res.status === 200) handle = res.data?.handle;
        } catch (err) {
            if (!(err instanceof FetchError && err.status === 404)) {
                throw err;
            }
        }
        if (handle) {
            throw redirect({
                to: "/users/$handle",
                params: { handle },
                statusCode: 301,
                replace: true,
            });
        }
        throw notFound();
    },
    notFoundComponent: NotFound,
});

function NotFound() {
    return (
        <div className="max-w-3xl mx-auto px-8 py-16">
            <h1 className="text-2xl font-bold text-stone-900 mb-2">
                User not found
            </h1>
            <p className="text-sm text-stone-500">
                The user you're looking for doesn't exist or has been removed.
            </p>
        </div>
    );
}
