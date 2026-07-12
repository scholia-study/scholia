import { createFileRoute, Link } from "@tanstack/react-router";
import { useState } from "react";
import { useForgotPassword } from "../api/auth/auth";
import { SEO_COPY, seoHead } from "../modules/seo";

export const Route = createFileRoute("/forgot-password")({
    head: () =>
        seoHead({
            title: SEO_COPY.auth.forgotPassword,
            path: "/forgot-password",
            noindex: true,
        }),
    component: ForgotPasswordPage,
});

function ForgotPasswordPage() {
    const [email, setEmail] = useState("");
    const [sent, setSent] = useState(false);

    const forgotMutation = useForgotPassword();

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            await forgotMutation.mutateAsync({ data: { email } });
        } catch {
            // Always show success to avoid leaking email existence
        } finally {
            setSent(true);
        }
    };

    if (sent) {
        return (
            <div className="flex items-center justify-center min-h-[calc(100vh-3rem)]">
                <div className="w-full max-w-sm px-8 text-center">
                    <h1 className="text-2xl font-bold text-stone-900 mb-4">
                        Check your email
                    </h1>
                    <p className="text-stone-600 mb-4">
                        If an account with that email exists, we've sent a
                        password reset link.
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
                <h1 className="text-2xl font-bold text-stone-900 mb-2">
                    Forgot password
                </h1>
                <p className="text-stone-500 text-sm mb-6">
                    Enter your email and we'll send you a reset link.
                </p>

                <form onSubmit={handleSubmit} className="space-y-4">
                    <div>
                        <label
                            htmlFor="forgot-email"
                            className="block text-sm text-stone-600 mb-1"
                        >
                            Email
                        </label>
                        <input
                            id="forgot-email"
                            type="email"
                            value={email}
                            onChange={(e) => setEmail(e.target.value)}
                            required
                            className="w-full px-3 py-2 border border-stone-300 rounded bg-white focus:outline-none focus:ring-2 focus:ring-stone-400"
                        />
                    </div>
                    <button
                        type="submit"
                        disabled={forgotMutation.isPending}
                        className="w-full py-2 rounded bg-stone-800 text-white hover:bg-stone-700 transition-colors disabled:opacity-50"
                    >
                        {forgotMutation.isPending
                            ? "Sending..."
                            : "Send reset link"}
                    </button>
                </form>

                <p className="mt-4 text-sm text-stone-500">
                    <Link
                        to="/login"
                        className="underline hover:text-stone-700"
                    >
                        Back to login
                    </Link>
                </p>
            </div>
        </div>
    );
}
