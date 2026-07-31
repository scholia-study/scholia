import { createFileRoute, Link } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { useAdminListTopics } from "../api/admin/admin";
import { useListUsers } from "../api/admin-users/admin-users";
import { useListArticleReviewQueue } from "../api/article-reviews/article-reviews";
import { useListFeedback } from "../api/feedback/feedback";
import { useListResourceSubmissions } from "../api/resources/resources";
import { useAuth } from "../hooks/useAuth";
import { BRIDGE_SECTIONS } from "./_auth._manage";

export const Route = createFileRoute("/_auth/_manage/manage/")({
    component: BridgeDashboard,
});

function BridgeDashboard() {
    const { hasPermission } = useAuth();
    const counts = {
        "/manage/feedback": <FeedbackCount />,
        "/manage/users": <UsersCount />,
        "/manage/topics": <TopicsCount />,
        "/manage/resource-submissions": <SubmissionsCount />,
        "/manage/article-reviews": <ReviewsCount />,
    } as const;

    return (
        <div className="max-w-3xl mx-auto px-8 py-16">
            <h1 className="text-lg font-semibold text-stone-800 mb-1">
                The Bridge
            </h1>
            <p className="text-sm text-stone-500 mb-8">
                Editorial and administrative tools.
            </p>
            <div className="grid sm:grid-cols-2 gap-4">
                {BRIDGE_SECTIONS.filter((s) => hasPermission(s.permission)).map(
                    (section) => (
                        <Link
                            key={section.to}
                            to={section.to}
                            className="border border-stone-200 rounded-lg p-4 no-underline hover:border-stone-400 hover:bg-stone-50 transition-colors"
                        >
                            <div className="flex items-baseline justify-between">
                                <span className="text-sm font-medium text-stone-800">
                                    {section.label}
                                </span>
                                {counts[section.to]}
                            </div>
                            <p className="text-xs text-stone-500 mt-1">
                                {section.description}
                            </p>
                        </Link>
                    ),
                )}
            </div>
        </div>
    );
}

function CountBadge({ children }: { children: ReactNode }) {
    return (
        <span className="text-xs text-stone-400 tabular-nums">{children}</span>
    );
}

function FeedbackCount() {
    const { data } = useListFeedback({
        filter: "active",
        page: 1,
        per_page: 1,
    });
    const total = data?.data?.total;
    return <CountBadge>{total != null ? `${total} active` : ""}</CountBadge>;
}

function UsersCount() {
    const { data } = useListUsers({ page: 1, per_page: 1 });
    const total = data?.data?.total;
    return <CountBadge>{total ?? ""}</CountBadge>;
}

function TopicsCount() {
    const { data } = useAdminListTopics();
    const total = data?.data?.topics?.length;
    return <CountBadge>{total ?? ""}</CountBadge>;
}

function SubmissionsCount() {
    const { data } = useListResourceSubmissions({
        filter: "pending",
        page: 1,
        per_page: 1,
    });
    const total = data?.data?.total;
    return <CountBadge>{total != null ? `${total} pending` : ""}</CountBadge>;
}

function ReviewsCount() {
    const { data } = useListArticleReviewQueue({
        filter: "pending",
        page: 1,
        per_page: 1,
    });
    const total = data?.data?.total;
    return <CountBadge>{total != null ? `${total} pending` : ""}</CountBadge>;
}
