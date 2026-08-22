import { useQueryClient } from "@tanstack/react-query";
import { Link, useNavigate } from "@tanstack/react-router";
import { useEffect, useRef, useState } from "react";
import toast from "react-hot-toast";
import {
    getListMyCollegiaQueryKey,
    useJoinByToken,
} from "#/api/collegia/collegia";
import { FetchError } from "#/api/fetcher";

/**
 * The `/user/collegia/join/$token` landing page: redeems the invite link once
 * and forwards to the collegium page.
 */
export function JoinCollegiumPage({ token }: { token: string }) {
    const navigate = useNavigate();
    const queryClient = useQueryClient();
    const joinMutation = useJoinByToken();
    const [error, setError] = useState<string | null>(null);
    const attempted = useRef(false);

    useEffect(() => {
        if (attempted.current) return;
        attempted.current = true;
        joinMutation
            .mutateAsync({ token })
            .then((result) => {
                queryClient.invalidateQueries({
                    queryKey: getListMyCollegiaQueryKey(),
                });
                toast.success(
                    result.data.already_member
                        ? `You're already a member of ${result.data.name}.`
                        : `Welcome to ${result.data.name}!`,
                );
                navigate({
                    to: "/user/collegia/$slug",
                    params: { slug: result.data.slug },
                    replace: true,
                });
            })
            .catch((err) => {
                setError(
                    err instanceof FetchError && err.message
                        ? err.message
                        : "This invite link is invalid or has been revoked.",
                );
            });
    }, [joinMutation, navigate, queryClient, token]);

    return (
        <div className="w-full max-w-3xl mx-auto px-8 py-16">
            {error ? (
                <>
                    <h1 className="text-xl font-bold text-stone-900 mb-2">
                        Invite link not valid
                    </h1>
                    <p className="text-sm text-stone-500">
                        {error}{" "}
                        <Link to="/user/collegia" className="underline">
                            Back to Collegia
                        </Link>
                    </p>
                </>
            ) : (
                <p className="text-sm text-stone-400">Joining collegium...</p>
            )}
        </div>
    );
}
