import { createContext, type ReactNode, useContext, useMemo } from "react";
import { useMe } from "../api/auth/auth";
import type { AuthResponse } from "../api/model";

interface AuthValue {
    user: AuthResponse | null;
    isLoading: boolean;
    isAuthenticated: boolean;
    permissions: string[];
    hasPermission: (name: string) => boolean;
}

const AuthContext = createContext<AuthValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
    const { data, isLoading } = useMe({ query: { retry: false } });
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
