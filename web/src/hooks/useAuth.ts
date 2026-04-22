import { useMe } from "../api/auth/auth";

export function useAuth() {
    const { data: meData, isLoading } = useMe({
        query: { retry: false },
    });

    const user = meData?.data ?? null;
    const permissions = user?.permissions ?? [];
    const hasPermission = (name: string) => permissions.includes(name);

    return {
        user,
        isLoading,
        isAuthenticated: !!user,
        permissions,
        hasPermission,
    };
}
