import { createFileRoute, Link } from "@tanstack/react-router";
import { useState } from "react";
import { useRegister } from "../api/auth/auth";
import { FetchError } from "../api/fetcher";

export const Route = createFileRoute("/register")({
    component: RegisterPage,
});

function RegisterPage() {
    const [email, setEmail] = useState("");
    const [displayName, setDisplayName] = useState("");
    const [password, setPassword] = useState("");
    const [error, setError] = useState("");
    const [success, setSuccess] = useState(false);

    const registerMutation = useRegister();

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError("");
        try {
            await registerMutation.mutateAsync({
                data: { email, display_name: displayName, password },
            });
            setSuccess(true);
        } catch (err) {
            if (err instanceof FetchError) {
                if (err.status === 409) {
                    setError("An account with this email already exists.");
                } else {
                    setError(err.message);
                }
            } else {
                setError("Something went wrong. Please try again.");
            }
        }
    };

    if (success) {
        return (
            <div className="flex items-center justify-center min-h-[calc(100vh-3rem)]">
                <div className="w-full max-w-sm px-8 text-center">
                    <h1 className="text-2xl font-bold text-stone-900 mb-4">
                        Check your email
                    </h1>
                    <p className="text-stone-600 mb-4">
                        We've sent a verification link to{" "}
                        <strong>{email}</strong>. Click the link to activate
                        your account.
                    </p>
                    <Link
                        to="/login"
                        className="text-sm underline text-stone-500 hover:text-stone-700"
                    >
                        Back to login
                    </Link>
                </div>
            </div>
        );
    }

    return (
        <div className="flex items-center justify-center min-h-[calc(100vh-3rem)]">
            <div className="w-full max-w-sm px-8">
                <h1 className="text-2xl font-bold text-stone-900 mb-6">
                    Create an account
                </h1>

                {error && (
                    <div className="mb-4 p-3 rounded bg-red-50 text-red-800 text-sm border border-red-200">
                        {error}
                    </div>
                )}

                <form onSubmit={handleSubmit} className="space-y-4">
                    <div>
                        <label
                            htmlFor="register-display-name"
                            className="block text-sm text-stone-600 mb-1"
                        >
                            Display name
                        </label>
                        <input
                            id="register-display-name"
                            type="text"
                            value={displayName}
                            onChange={(e) => setDisplayName(e.target.value)}
                            required
                            className="w-full px-3 py-2 border border-stone-300 rounded bg-white focus:outline-none focus:ring-2 focus:ring-stone-400"
                        />
                    </div>
                    <div>
                        <label
                            htmlFor="register-email"
                            className="block text-sm text-stone-600 mb-1"
                        >
                            Email
                        </label>
                        <input
                            id="register-email"
                            type="email"
                            value={email}
                            onChange={(e) => setEmail(e.target.value)}
                            required
                            className="w-full px-3 py-2 border border-stone-300 rounded bg-white focus:outline-none focus:ring-2 focus:ring-stone-400"
                        />
                    </div>
                    <div>
                        <label
                            htmlFor="register-password"
                            className="block text-sm text-stone-600 mb-1"
                        >
                            Password
                        </label>
                        <input
                            id="register-password"
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
                    <button
                        type="submit"
                        disabled={registerMutation.isPending}
                        className="w-full py-2 rounded bg-stone-800 text-white hover:bg-stone-700 transition-colors disabled:opacity-50"
                    >
                        {registerMutation.isPending
                            ? "Creating account..."
                            : "Create account"}
                    </button>
                </form>

                <div className="mt-4">
                    <a
                        href="http://localhost:4000/auth/github"
                        className="block w-full py-2 text-center rounded border border-stone-300 bg-white hover:bg-stone-50 transition-colors text-stone-800"
                    >
                        Sign up with GitHub
                    </a>
                </div>

                <p className="mt-4 text-sm text-stone-500">
                    Already have an account?{" "}
                    <Link
                        to="/login"
                        className="underline hover:text-stone-700"
                    >
                        Log in
                    </Link>
                </p>
            </div>
        </div>
    );
}
