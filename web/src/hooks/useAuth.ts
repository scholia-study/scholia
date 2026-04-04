import { useMe } from "../api/auth/auth";

export function useAuth() {
    const { data: meData, isLoading } = useMe({
        query: { retry: false },
    });

    const user = meData?.data ?? null;

    return {
        user,
        isLoading,
        isAuthenticated: !!user,
    };
}
