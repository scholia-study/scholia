import { createContext, type ReactNode, useContext, useMemo } from "react";
import { useMe } from "../api/auth/auth";
import { FetchError } from "../api/fetcher";
import type { AuthResponse } from "../api/model";

/** True only for a real "not logged in" answer from the API. */
export function isUnauthorized(error: unknown): boolean {
    return error instanceof FetchError && error.status === 401;
}

/** Retry policy for the `me` query: a 401 is a definitive logged-out answer,
 *  but a network failure or 5xx (e.g. the API restarting mid-deploy) is
 *  transient — retry up to 5 times before giving up. Never retry during SSR:
 *  backoff there would stall the whole page render; the client observer
 *  refetches after hydration anyway. */
export function meRetry(failureCount: number, error: unknown): boolean {
    if (typeof window === "undefined") {
        return false;
    }
    return !isUnauthorized(error) && failureCount < 5;
}

interface AuthValue {
    user: AuthResponse | null;
    isLoading: boolean;
    isAuthenticated: boolean;
    permissions: string[];
    hasPermission: (name: string) => boolean;
}

const AuthContext = createContext<AuthValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
    const { data, isLoading } = useMe({ query: { retry: meRetry } });
    const user = data?.data ?? null;

    const value = useMemo<AuthValue>(() => {
        const permissions = user?.permissions ?? [];
        return {
            user,
            isLoading,
            isAuthenticated: !!user,
            permissions,
            hasPermission: (name: string) => permissions.includes(name),
        };
    }, [user, isLoading]);

    return (
        <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
    );
}

export function useAuth(): AuthValue {
    const ctx = useContext(AuthContext);
    if (!ctx) {
        throw new Error("useAuth must be used within an <AuthProvider>");
    }
    return ctx;
}
