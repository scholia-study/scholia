import PublishOutlined from "@mui/icons-material/PublishOutlined";
import SaveOutlined from "@mui/icons-material/SaveOutlined";
import UnpublishedOutlined from "@mui/icons-material/UnpublishedOutlined";
import {
    Autocomplete,
    Button,
    Chip,
    TextField,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { Link, createFileRoute, redirect, useNavigate } from "@tanstack/react-router";
import { useCallback, useEffect, useRef, useState } from "react";
import {
    getGetUserArticleQueryKey,
    getListUserArticlesQueryKey,
    useGetUserArticle,
    usePublishArticle,
    useUnpublishArticle,
    useUpdateArticle,
} from "../api/articles/articles";
import { getGetProfileQueryOptions } from "../api/auth/auth";
import type { TopicResponse } from "../api/model";
import { useListTopics } from "../api/topics/topics";
import {
    ArticleEditorLazy as ArticleEditor,
    type ArticleEditorHandle,
} from "../components/editor/ArticleEditorLazy";
import {
    QuotationPickerModal,
    type QuotationPickerResult,
} from "../components/editor/QuotationPickerModal";

export const Route = createFileRoute("/user/articles/$slug")({
    beforeLoad: async ({ context }) => {
        const data = await context.queryClient.fetchQuery(
            getGetProfileQueryOptions(),
        );
        if (!data?.data) {
            throw redirect({ to: "/login" });
        }
    },
    component: ArticleEditorPage,
});

function ArticleEditorPage() {
    const { slug } = Route.useParams();
    const navigate = useNavigate();
    const queryClient = useQueryClient();

    const { data: articleData } = useGetUserArticle(slug);
    const article = articleData?.data;

    const { data: topicsData } = useListTopics();
    const allTopics = topicsData?.data?.topics ?? [];

    const updateMutation = useUpdateArticle();
    const publishMutation = usePublishArticle();
    const unpublishMutation = useUnpublishArticle();

    // Local state for editing
    const [title, setTitle] = useState("");
    const [description, setDescription] = useState("");
    const [markdown, setMarkdown] = useState("");
    const [selectedTopics, setSelectedTopics] = useState<TopicResponse[]>([]);
    const [saveStatus, setSaveStatus] = useState<"saved" | "saving" | "unsaved">("saved");
    const [pickerOpen, setPickerOpen] = useState(false);

    const editorRef = useRef<ArticleEditorHandle>(null);
    const initialized = useRef(false);

    // Initialize from article data
    useEffect(() => {
        if (article && !initialized.current) {
            setTitle(article.title);
            setDescription(article.description ?? "");
            setMarkdown(article.markdown);
            setSelectedTopics(article.topics);
            initialized.current = true;
        }
    }, [article]);

    // Auto-save debounce
    const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
    const currentSlug = useRef(slug);
    currentSlug.current = slug;

    const save = useCallback(
        async (updates: {
            title?: string;
            markdown?: string;
            description?: string;
            topic_ids?: string[];
        }) => {
            setSaveStatus("saving");
            try {
                const result = await updateMutation.mutateAsync({
                    slug: currentSlug.current,
                    data: updates,
                });
                setSaveStatus("saved");
                // If slug changed (title change), navigate to new slug
                const newSlug = result.data?.slug;
                if (newSlug && newSlug !== currentSlug.current) {
                    currentSlug.current = newSlug;
                    navigate({
                        to: "/user/articles/$slug",
                        params: { slug: newSlug },
                        replace: true,
                    });
                }
                queryClient.invalidateQueries({
                    queryKey: getGetUserArticleQueryKey(currentSlug.current),
                });
            } catch {
                setSaveStatus("unsaved");
            }
        },
        [updateMutation, navigate, queryClient],
    );

    const debouncedSave = useCallback(
        (updates: Parameters<typeof save>[0]) => {
            setSaveStatus("unsaved");
            if (saveTimer.current) clearTimeout(saveTimer.current);
            saveTimer.current = setTimeout(() => save(updates), 1500);
        },
        [save],
    );

    const handleTitleBlur = () => {
        if (title !== article?.title) {
            save({ title });
        }
    };

    const handleDescriptionBlur = () => {
        if (description !== (article?.description ?? "")) {
            save({ description });
        }
    };

    const handleMarkdownChange = (value: string) => {
        setMarkdown(value);
        debouncedSave({ markdown: value });
    };

    const handleTopicsChange = (_: unknown, value: TopicResponse[]) => {
        if (value.length > 5) return;
        setSelectedTopics(value);
        save({ topic_ids: value.map((t) => t.id) });
    };

    const handleInsertQuotation = (result: QuotationPickerResult) => {
        editorRef.current?.insertQuotation(result);
    };

    const handlePublish = async () => {
        // Save any pending changes first
        if (saveTimer.current) {
            clearTimeout(saveTimer.current);
            await save({ title, markdown, description });
        }
        await publishMutation.mutateAsync({ slug: currentSlug.current });
        queryClient.invalidateQueries({
            queryKey: getGetUserArticleQueryKey(currentSlug.current),
        });
        queryClient.invalidateQueries({
            queryKey: getListUserArticlesQueryKey(),
        });
    };

    const handleUnpublish = async () => {
        await unpublishMutation.mutateAsync({ slug: currentSlug.current });
        queryClient.invalidateQueries({
            queryKey: getGetUserArticleQueryKey(currentSlug.current),
        });
        queryClient.invalidateQueries({
            queryKey: getListUserArticlesQueryKey(),
        });
    };

    if (!article) {
        return (
            <div className="max-w-4xl mx-auto px-8 py-16">
                <p className="text-sm text-stone-400">Loading...</p>
            </div>
        );
    }

    return (
        <div className="max-w-4xl mx-auto px-8 py-16">
            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <div className="flex items-center gap-2">
                    <Chip
                        label={article.status}
                        size="small"
                        color={
                            article.status === "published"
                                ? "success"
                                : article.status === "archived"
                                  ? "warning"
                                  : "default"
                        }
                        sx={{ fontSize: "0.65rem", height: 20 }}
                    />
                    <span className="text-xs text-stone-400">
                        {saveStatus === "saving"
                            ? "Saving..."
                            : saveStatus === "unsaved"
                              ? "Unsaved changes"
                              : "Saved"}
                    </span>
                </div>
                <div className="flex items-center gap-2">
                    {article.status === "draft" && (
                        <Button
                            size="small"
                            variant="contained"
                            startIcon={<PublishOutlined />}
                            onClick={handlePublish}
                            disabled={publishMutation.isPending}
                            sx={{ textTransform: "none" }}
                        >
                            Publish
                        </Button>
                    )}
                    {article.status === "published" && (
                        <>
                            <Link
                                to="/articles/$slug"
                                params={{ slug: currentSlug.current }}
                                target="_blank"
                                className="text-xs text-blue-500 hover:text-blue-700 underline"
                            >
                                View published
                            </Link>
                            <Button
                                size="small"
                                variant="outlined"
                                startIcon={<UnpublishedOutlined />}
                                onClick={handleUnpublish}
                                disabled={unpublishMutation.isPending}
                                sx={{ textTransform: "none" }}
                            >
                                Unpublish
                            </Button>
                        </>
                    )}
                    <Button
                        size="small"
                        variant="outlined"
                        startIcon={<SaveOutlined />}
                        onClick={() => save({ title, markdown, description })}
                        disabled={saveStatus === "saved" || saveStatus === "saving"}
                        sx={{ textTransform: "none" }}
                    >
                        Save
                    </Button>
                </div>
            </div>

            {/* Title */}
            <TextField
                fullWidth
                variant="standard"
                placeholder="Article title"
                value={title}
                onChange={(e) => {
                    setTitle(e.target.value);
                    setSaveStatus("unsaved");
                }}
                onBlur={handleTitleBlur}
                slotProps={{
                    input: {
                        sx: {
                            fontSize: "1.75rem",
                            fontWeight: 700,
                            fontFamily: "'Libre Baskerville', serif",
                        },
                        disableUnderline: true,
                    },
                }}
                sx={{ mb: 2 }}
            />

            {/* Description */}
            <TextField
                fullWidth
                variant="standard"
                placeholder="Description (optional, used for listings and SEO)"
                value={description}
                onChange={(e) => {
                    setDescription(e.target.value);
                    setSaveStatus("unsaved");
                }}
                onBlur={handleDescriptionBlur}
                multiline
                maxRows={3}
                slotProps={{
                    input: {
                        sx: { fontSize: "0.875rem", color: "rgb(120 113 108)" },
                        disableUnderline: true,
                    },
                }}
                sx={{ mb: 2 }}
            />

            {/* Topics */}
            <Autocomplete
                multiple
                options={allTopics}
                getOptionLabel={(option) => option.name}
                value={selectedTopics}
                onChange={handleTopicsChange}
                isOptionEqualToValue={(option, value) => option.id === value.id}
                renderInput={(params) => (
                    <TextField
                        {...params}
                        variant="standard"
                        placeholder={selectedTopics.length === 0 ? "Add topics (max 5)" : ""}
                        slotProps={{
                            input: {
                                ...params.InputProps,
                                disableUnderline: true,
                            },
                        }}
                    />
                )}
                renderTags={(value, getTagProps) =>
                    value.map((option, index) => {
                        const { key, ...rest } = getTagProps({ index });
                        return (
                            <Chip
                                key={key}
                                label={option.name}
                                size="small"
                                {...rest}
                            />
                        );
                    })
                }
                sx={{ mb: 4 }}
            />

            {/* MDXEditor */}
            <div className="border border-stone-200 rounded-lg overflow-hidden">
                <ArticleEditor
                    ref={editorRef}
                    markdown={article.markdown}
                    onChange={handleMarkdownChange}
                    onInsertQuotationClick={() => setPickerOpen(true)}
                />
            </div>

            <QuotationPickerModal
                open={pickerOpen}
                onClose={() => setPickerOpen(false)}
                onSelect={handleInsertQuotation}
            />
        </div>
    );
}
