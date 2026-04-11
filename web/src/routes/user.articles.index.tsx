import AddOutlined from "@mui/icons-material/AddOutlined";
import ArchiveOutlined from "@mui/icons-material/ArchiveOutlined";
import EditOutlined from "@mui/icons-material/EditOutlined";
import PublishOutlined from "@mui/icons-material/PublishOutlined";
import {
    Button,
    Chip,
    Dialog,
    DialogActions,
    DialogContent,
    DialogContentText,
    DialogTitle,
    IconButton,
    Paper,
    Tab,
    Tabs,
    TextField,
    Tooltip,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { Link, createFileRoute, redirect, useNavigate } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import {
    getListUserArticlesQueryKey,
    useArchiveArticle,
    useCreateArticle,
    useListUserArticles,
    usePublishArticle,
} from "../api/articles/articles";
import { getGetProfileQueryOptions } from "../api/auth/auth";
import type { ArticleResponse } from "../api/model";

export const Route = createFileRoute("/user/articles/")({
    beforeLoad: async ({ context }) => {
        const data = await context.queryClient.fetchQuery(
            getGetProfileQueryOptions(),
        );
        if (!data?.data) {
            throw redirect({ to: "/login" });
        }
    },
    component: ArticlesPage,
});

const STATUS_TABS = ["all", "draft", "published", "archived"] as const;

function statusColor(status: string) {
    switch (status) {
        case "published":
            return "success";
        case "draft":
            return "default";
        case "archived":
            return "warning";
        default:
            return "default";
    }
}

function ArticlesPage() {
    const navigate = useNavigate();
    const queryClient = useQueryClient();
    const [statusTab, setStatusTab] = useState<string>("all");
    const [createDialogOpen, setCreateDialogOpen] = useState(false);
    const [newTitle, setNewTitle] = useState("");

    const { data: articlesData, isLoading } = useListUserArticles({});
    const articles = articlesData?.data?.articles ?? [];
    const limits = articlesData?.data?.limits;

    const filtered = useMemo(() => {
        const list =
            statusTab === "all"
                ? articles
                : articles.filter((a) => a.status === statusTab);
        return [...list].sort((a, b) => {
            if (statusTab === "all") {
                const aArch = a.status === "archived" ? 1 : 0;
                const bArch = b.status === "archived" ? 1 : 0;
                if (aArch !== bArch) return aArch - bArch;
            }
            return (
                new Date(b.created_at).getTime() -
                new Date(a.created_at).getTime()
            );
        });
    }, [articles, statusTab]);

    const [publishSlug, setPublishSlug] = useState<string | null>(null);

    const createMutation = useCreateArticle();
    const publishMutation = usePublishArticle();
    const archiveMutation = useArchiveArticle();

    const handleCreate = async () => {
        if (!newTitle.trim()) return;
        const result = await createMutation.mutateAsync({ data: { title: newTitle.trim() } });
        setCreateDialogOpen(false);
        setNewTitle("");
        queryClient.invalidateQueries({ queryKey: getListUserArticlesQueryKey() });
        if (result.data?.slug) {
            navigate({ to: "/user/articles/$slug", params: { slug: result.data.slug } });
        }
    };

    const handlePublishConfirm = async () => {
        if (!publishSlug) return;
        setPublishSlug(null);
        await publishMutation.mutateAsync({ slug: publishSlug });
        queryClient.invalidateQueries({ queryKey: getListUserArticlesQueryKey() });
    };

    const handleArchive = async (slug: string) => {
        await archiveMutation.mutateAsync({ slug });
        queryClient.invalidateQueries({ queryKey: getListUserArticlesQueryKey() });
    };

    const canCreate = limits ? limits.current_total < limits.max_total : true;
    const canPublish = limits ? limits.current_published < limits.max_published : true;

    return (
        <div className="max-w-3xl mx-auto px-8 py-16">
            <div className="flex items-center justify-between mb-6">
                <h1 className="text-2xl font-bold text-stone-900">
                    My Articles
                </h1>
                <div className="flex items-center gap-3">
                    {limits && (
                        <span className="text-xs text-stone-400">
                            {limits.current_published}/{limits.max_published} published
                            {" \u00B7 "}
                            {limits.current_total}/{limits.max_total} total
                        </span>
                    )}
                    <Button
                        variant="contained"
                        size="small"
                        startIcon={<AddOutlined />}
                        disabled={!canCreate}
                        onClick={() => setCreateDialogOpen(true)}
                        sx={{ textTransform: "none" }}
                    >
                        New Article
                    </Button>
                </div>
            </div>

            <Tabs
                value={statusTab}
                onChange={(_, v) => setStatusTab(v)}
                sx={{ mb: 3, minHeight: 36 }}
            >
                {STATUS_TABS.map((tab) => (
                    <Tab
                        key={tab}
                        value={tab}
                        label={tab === "all" ? "All" : tab.charAt(0).toUpperCase() + tab.slice(1)}
                        sx={{ minHeight: 36, textTransform: "none", fontSize: "0.875rem" }}
                    />
                ))}
            </Tabs>

            {isLoading && (
                <p className="text-sm text-stone-400">Loading...</p>
            )}

            {!isLoading && filtered.length === 0 && (
                <p className="text-sm text-stone-400">
                    {statusTab === "all"
                        ? "No articles yet. Create your first one!"
                        : `No ${statusTab} articles.`}
                </p>
            )}

            <div className="space-y-2">
                {filtered.map((article) => (
                    <ArticleRow
                        key={article.id}
                        article={article}
                        canPublish={canPublish}
                        onPublish={(slug) => setPublishSlug(slug)}
                        onArchive={handleArchive}
                    />
                ))}
            </div>

            <Dialog
                open={createDialogOpen}
                onClose={() => setCreateDialogOpen(false)}
                maxWidth="sm"
                fullWidth
            >
                <DialogTitle>New Article</DialogTitle>
                <DialogContent>
                    <TextField
                        autoFocus
                        fullWidth
                        label="Title"
                        value={newTitle}
                        onChange={(e) => setNewTitle(e.target.value)}
                        onKeyDown={(e) => {
                            if (e.key === "Enter") handleCreate();
                        }}
                        sx={{ mt: 1 }}
                    />
                </DialogContent>
                <DialogActions>
                    <Button onClick={() => setCreateDialogOpen(false)}>Cancel</Button>
                    <Button
                        onClick={handleCreate}
                        variant="contained"
                        disabled={!newTitle.trim() || createMutation.isPending}
                    >
                        Create
                    </Button>
                </DialogActions>
            </Dialog>

            <Dialog
                open={publishSlug != null}
                onClose={() => setPublishSlug(null)}
                maxWidth="sm"
            >
                <DialogTitle>Publish this article?</DialogTitle>
                <DialogContent>
                    <DialogContentText sx={{ fontSize: "0.875rem", mb: 1.5 }}>
                        Once published, this article becomes public and
                        cannot be reverted to a draft. You can:
                    </DialogContentText>
                    <ul className="text-sm text-stone-600 list-disc pl-5 space-y-1">
                        <li>Continue editing the article at any time</li>
                        <li>
                            Archive it later, which removes it from
                            listings but keeps it accessible via direct
                            link for historical references
                        </li>
                    </ul>
                </DialogContent>
                <DialogActions sx={{ px: 3, pb: 2 }}>
                    <Button
                        onClick={() => setPublishSlug(null)}
                        size="small"
                    >
                        Cancel
                    </Button>
                    <Button
                        onClick={handlePublishConfirm}
                        size="small"
                        variant="contained"
                        disabled={publishMutation.isPending}
                    >
                        Publish
                    </Button>
                </DialogActions>
            </Dialog>
        </div>
    );
}

function ArticleRow({
    article,
    canPublish,
    onPublish,
    onArchive,
}: {
    article: ArticleResponse;
    canPublish: boolean;
    onPublish: (slug: string) => void;
    onArchive: (slug: string) => void;
}) {
    return (
        <Paper
            elevation={0}
            sx={{
                border: "1px solid rgb(214 211 209)",
                p: 1.5,
                display: "flex",
                alignItems: "center",
                gap: 1.5,
                transition: "box-shadow 0.15s",
                "&:hover": { boxShadow: 3 },
            }}
        >
            <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-0.5">
                    {article.status === "published" ? (
                        <Link
                            to="/articles/$slug"
                            params={{ slug: article.slug }}
                            target="_blank"
                            className="text-sm font-medium text-stone-900 hover:underline truncate"
                        >
                            {article.title}
                        </Link>
                    ) : (
                        <span className="text-sm font-medium text-stone-500 truncate">
                            {article.title}
                        </span>
                    )}
                    <Chip
                        label={article.status}
                        size="small"
                        color={statusColor(article.status) as "default"}
                        sx={{ fontSize: "0.65rem", height: 20 }}
                    />
                </div>
                <div className="flex items-center gap-2">
                    {article.topics.map((t) => (
                        <span key={t.id} className="text-[10px] text-stone-400">
                            {t.name}
                        </span>
                    ))}
                    <span className="text-[10px] text-stone-300">
                        {new Date(article.updated_at).toLocaleDateString(undefined, {
                            month: "short",
                            day: "numeric",
                            year: "numeric",
                        })}
                    </span>
                </div>
            </div>

            <div className="flex items-center gap-0.5 shrink-0">
                <Tooltip title="Edit">
                    <Link to="/user/articles/$slug" params={{ slug: article.slug }}>
                        <IconButton size="small">
                            <EditOutlined fontSize="small" />
                        </IconButton>
                    </Link>
                </Tooltip>

                {article.status === "draft" && (
                    <Tooltip title={canPublish ? "Publish" : "Publish limit reached"}>
                        <span>
                            <IconButton
                                size="small"
                                disabled={!canPublish}
                                onClick={() => onPublish(article.slug)}
                            >
                                <PublishOutlined fontSize="small" />
                            </IconButton>
                        </span>
                    </Tooltip>
                )}

                {article.status === "published" && (
                    <Tooltip title="Archive">
                        <IconButton
                            size="small"
                            onClick={() => onArchive(article.slug)}
                        >
                            <ArchiveOutlined fontSize="small" />
                        </IconButton>
                    </Tooltip>
                )}
            </div>
        </Paper>
    );
}
