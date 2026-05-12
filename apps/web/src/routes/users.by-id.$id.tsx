import { createFileRoute, redirect } from "@tanstack/react-router";
import { getHandleById } from "../api/users/users";

/**
 * Durable redirect from a UUID-keyed profile URL to the user's current
 * handle URL. Mirrors `/articles/by-id/<id>` for articles.
 *
 * Resolved during `beforeLoad` so the URL bar lands on the canonical
 * `/users/<handle>` form (good for sharing and bookmarking).
 */
export const Route = createFileRoute("/users/by-id/$id")({
    beforeLoad: async ({ params }) => {
        try {
            const res = await getHandleById(params.id);
            if (res.status === 200 && res.data?.handle) {
                throw redirect({
                    to: "/users/$handle",
                    params: { handle: res.data.handle },
                });
            }
        } catch (err) {
            // Re-throw redirects (TanStack uses thrown errors as the
            // navigation primitive). Anything else means the lookup
            // failed; let the component show the not-found state.
            if (err && typeof err === "object" && "isRedirect" in err) {
                throw err;
            }
        }
    },
    component: NotFound,
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
