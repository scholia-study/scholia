import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { getMeQueryKey, useLogin } from "../api/auth/auth";
import { FetchError } from "../api/fetcher";
import config from "../config";
import { SEO_COPY, seoHead } from "../modules/seo";

interface LoginSearch {
    verified?: string;
    password_reset?: string;
    error?: string;
}

export const Route = createFileRoute("/login")({
    validateSearch: (search: Record<string, unknown>): LoginSearch => ({
        verified: search.verified as string | undefined,
        password_reset: search.password_reset as string | undefined,
        error: search.error as string | undefined,
    }),
    head: () =>
        seoHead({
            title: SEO_COPY.auth.login,
            path: "/login",
            noindex: true,
        }),
    component: LoginPage,
});

function LoginPage() {
    const { verified, password_reset, error: queryError } = Route.useSearch();
    const navigate = useNavigate();
    const queryClient = useQueryClient();
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [error, setError] = useState("");

    const loginMutation = useLogin();

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError("");
        try {
            await loginMutation.mutateAsync({ data: { email, password } });
            await queryClient.invalidateQueries({ queryKey: getMeQueryKey() });
            navigate({ to: "/" });
        } catch (err) {
            if (err instanceof FetchError) {
                if (err.status === 403) {
                    setError(
                        "Email not verified. Check your inbox for the verification link.",
                    );
                } else {
                    setError("Invalid email or password.");
                }
            } else {
                setError("Something went wrong. Please try again.");
            }
        }
    };

    return (
        <div className="flex items-center justify-center min-h-[calc(100vh-3rem)]">
            <div className="w-full max-w-sm px-8">
                <h1 className="text-2xl font-bold text-stone-900 mb-6">
                    Log in
                </h1>

                {verified && (
                    <div className="mb-4 p-3 rounded bg-green-50 text-green-800 text-sm border border-green-200">
                        Email verified! You can now log in.
                    </div>
                )}

                {password_reset && (
                    <div className="mb-4 p-3 rounded bg-green-50 text-green-800 text-sm border border-green-200">
                        Password changed successfully. Log in with your new
                        password.
                    </div>
                )}

                {queryError && (
                    <div className="mb-4 p-3 rounded bg-red-50 text-red-800 text-sm border border-red-200">
                        {queryError === "invalid_token"
                            ? "Invalid or expired verification link."
                            : `Authentication error: ${queryError}`}
                    </div>
                )}

                {error && (
                    <div className="mb-4 p-3 rounded bg-red-50 text-red-800 text-sm border border-red-200">
                        {error}
                    </div>
                )}

                <form onSubmit={handleSubmit} className="space-y-4">
                    <div>
                        <label
                            htmlFor="login-email"
                            className="block text-sm text-stone-600 mb-1"
                        >
                            Email
                        </label>
                        <input
                            id="login-email"
                            type="email"
                            value={email}
                            onChange={(e) => setEmail(e.target.value)}
                            required
                            className="w-full px-3 py-2 border border-stone-300 rounded bg-white focus:outline-none focus:ring-2 focus:ring-stone-400"
                        />
                    </div>
                    <div>
                        <label
                            htmlFor="login-password"
                            className="block text-sm text-stone-600 mb-1"
                        >
                            Password
                        </label>
                        <input
                            id="login-password"
                            type="password"
                            value={password}
                            onChange={(e) => setPassword(e.target.value)}
                            required
                            className="w-full px-3 py-2 border border-stone-300 rounded bg-white focus:outline-none focus:ring-2 focus:ring-stone-400"
                        />
                    </div>
                    <button
                        type="submit"
                        disabled={loginMutation.isPending}
                        className="w-full py-2 rounded bg-stone-800 text-white hover:bg-stone-700 transition-colors disabled:opacity-50"
                    >
                        {loginMutation.isPending ? "Logging in..." : "Log in"}
                    </button>
                </form>

                <div className="mt-4">
                    <a
                        href={`${config.API_BASE_URL}/api/auth/github`}
                        className="block w-full py-2 text-center rounded border border-stone-300 bg-white hover:bg-stone-50 transition-colors text-stone-800"
                    >
                        Sign in with GitHub
                    </a>
                </div>

                <div className="mt-4 text-sm text-stone-500 space-y-1">
                    <p>
                        <Link
                            to="/forgot-password"
                            className="underline hover:text-stone-700"
                        >
                            Forgot password?
                        </Link>
                    </p>
                    <p>
                        Don't have an account?{" "}
                        <Link
                            to="/register"
                            className="underline hover:text-stone-700"
                        >
                            Register
                        </Link>
                    </p>
                </div>
            </div>
        </div>
    );
}
