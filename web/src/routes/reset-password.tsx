import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { useResetPassword } from "../api/auth/auth";
import { FetchError } from "../api/fetcher";

interface ResetSearch {
    token?: string;
}

export const Route = createFileRoute("/reset-password")({
    validateSearch: (search: Record<string, unknown>): ResetSearch => ({
        token: search.token as string | undefined,
    }),
    component: ResetPasswordPage,
});

function ResetPasswordPage() {
    const { token } = Route.useSearch();
    const navigate = useNavigate();
    const [password, setPassword] = useState("");
    const [confirm, setConfirm] = useState("");
    const [error, setError] = useState("");

    const resetMutation = useResetPassword();

    if (!token) {
        return (
            <div className="flex items-center justify-center min-h-[calc(100vh-3rem)]">
                <div className="w-full max-w-sm px-8 text-center">
                    <h1 className="text-2xl font-bold text-stone-900 mb-4">
                        Invalid link
                    </h1>
                    <p className="text-stone-600 mb-4">
                        This password reset link is invalid or missing a token.
                    </p>
                    <Link
                        to="/forgot-password"
                        className="text-sm underline text-stone-500 hover:text-stone-700"
                    >
                        Request a new reset link
                    </Link>
                </div>
            </div>
        );
    }

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError("");

        if (password !== confirm) {
            setError("Passwords do not match.");
            return;
        }

        try {
            await resetMutation.mutateAsync({ data: { token, password } });
            navigate({ to: "/login", search: { password_reset: "true" } });
        } catch (err) {
            if (err instanceof FetchError) {
                setError(err.message);
            } else {
                setError("Something went wrong. Please try again.");
            }
        }
    };

    return (
        <div className="flex items-center justify-center min-h-[calc(100vh-3rem)]">
            <div className="w-full max-w-sm px-8">
                <h1 className="text-2xl font-bold text-stone-900 mb-6">
                    Reset password
                </h1>

                {error && (
                    <div className="mb-4 p-3 rounded bg-red-50 text-red-800 text-sm border border-red-200">
                        {error}
                    </div>
                )}

                <form onSubmit={handleSubmit} className="space-y-4">
                    <div>
                        <label className="block text-sm text-stone-600 mb-1">
                            New password
                        </label>
                        <input
                            type="password"
                            value={password}
                            onChange={(e) => setPassword(e.target.value)}
                            required
                            minLength={8}
                            className="w-full px-3 py-2 border border-stone-300 rounded bg-white focus:outline-none focus:ring-2 focus:ring-stone-400"
                        />
                        <p className="text-xs text-stone-400 mt-1">
                            At least 8 characters
                        </p>
                    </div>
                    <div>
                        <label className="block text-sm text-stone-600 mb-1">
                            Confirm password
                        </label>
                        <input
                            type="password"
                            value={confirm}
                            onChange={(e) => setConfirm(e.target.value)}
                            required
                            className="w-full px-3 py-2 border border-stone-300 rounded bg-white focus:outline-none focus:ring-2 focus:ring-stone-400"
                        />
                    </div>
                    <button
                        type="submit"
                        disabled={resetMutation.isPending}
                        className="w-full py-2 rounded bg-stone-800 text-white hover:bg-stone-700 transition-colors disabled:opacity-50"
                    >
                        {resetMutation.isPending
                            ? "Resetting..."
                            : "Reset password"}
                    </button>
                </form>
            </div>
        </div>
    );
}
