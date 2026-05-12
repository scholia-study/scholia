import { createFileRoute, Link } from "@tanstack/react-router";

interface VerifyEmailSearch {
    success?: string;
}

export const Route = createFileRoute("/verify-email")({
    validateSearch: (search: Record<string, unknown>): VerifyEmailSearch => ({
        success: search.success as string | undefined,
    }),
    component: VerifyEmailPage,
});

function VerifyEmailPage() {
    const { success } = Route.useSearch();

    if (success) {
        return (
            <div className="flex items-center justify-center min-h-[calc(100vh-3rem)]">
                <div className="w-full max-w-sm px-8 text-center">
                    <h1 className="text-2xl font-bold text-stone-900 mb-4">
                        Email verified
                    </h1>
                    <p className="text-stone-600 mb-4">
                        Your email has been verified and you're now logged in.
                    </p>
                    <Link
                        to="/"
                        className="inline-block px-4 py-2 rounded bg-stone-800 text-white hover:bg-stone-700 transition-colors"
                    >
                        Get started
                    </Link>
                </div>
            </div>
        );
    }

    return (
        <div className="flex items-center justify-center min-h-[calc(100vh-3rem)]">
            <div className="w-full max-w-sm px-8 text-center">
                <h1 className="text-2xl font-bold text-stone-900 mb-4">
                    Email verification
                </h1>
                <p className="text-stone-600 mb-4">
                    Check your inbox for the verification link. If you've
                    already verified, you can log in.
                </p>
                <Link
                    to="/login"
                    className="text-sm underline text-stone-500 hover:text-stone-700"
                >
                    Go to login
                </Link>
            </div>
        </div>
    );
}
