import PublishOutlined from "@mui/icons-material/PublishOutlined";
import SaveOutlined from "@mui/icons-material/SaveOutlined";
import ArchiveOutlined from "@mui/icons-material/ArchiveOutlined";
import {
    Autocomplete,
    Button,
    Chip,
    Dialog,
    DialogActions,
    DialogContent,
    DialogContentText,
    DialogTitle,
    TextField,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import {
    createFileRoute,
    Link,
    redirect,
    useNavigate,
} from "@tanstack/react-router";
import { memo, useCallback, useEffect, useRef, useState } from "react";
import {
    getGetUserArticleQueryKey,
    getListUserArticlesQueryKey,
    useGetUserArticle,
    useArchiveArticle,
    usePublishArticle,
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

const MemoizedEditor = memo(
    ({
        markdown,
        onChange,
        onInsertQuotationClick,
        readOnly,
        ref,
    }: {
        markdown: string;
        onChange: (markdown: string) => void;
        onInsertQuotationClick: () => void;
        readOnly?: boolean;
        ref: React.Ref<ArticleEditorHandle>;
    }) => (
        <div>
            <ArticleEditor
                ref={ref}
                markdown={markdown}
                onChange={onChange}
                onInsertQuotationClick={onInsertQuotationClick}
                readOnly={readOnly}
            />
        </div>
    ),
);
MemoizedEditor.displayName = "MemoizedEditor";

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
    const archiveMutation = useArchiveArticle();

    // Local state for editing
    const [title, setTitle] = useState("");
    const [description, setDescription] = useState("");
    const [markdown, setMarkdown] = useState("");
    const [selectedTopics, setSelectedTopics] = useState<TopicResponse[]>([]);
    const [saveStatus, setSaveStatus] = useState<
        "saved" | "saving" | "unsaved"
    >("saved");
    const [pickerOpen, setPickerOpen] = useState(false);

    const editorRef = useRef<ArticleEditorHandle>(null);
    const initialized = useRef(false);
    const openPicker = useCallback(() => setPickerOpen(true), []);

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

    const [publishDialogOpen, setPublishDialogOpen] = useState(false);
    const [archiveDialogOpen, setArchiveDialogOpen] = useState(false);

    const handlePublishConfirm = async () => {
        setPublishDialogOpen(false);
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

    const handleArchive = async () => {
        setArchiveDialogOpen(false);
        await archiveMutation.mutateAsync({ slug: currentSlug.current });
        queryClient.invalidateQueries({
            queryKey: getGetUserArticleQueryKey(currentSlug.current),
        });
        queryClient.invalidateQueries({
            queryKey: getListUserArticlesQueryKey(),
        });
    };

    if (!article) {
        return (
            <div className="min-h-screen bg-white">
                <div className="max-w-4xl mx-auto px-8 py-16">
                    <p className="text-sm text-stone-400">Loading...</p>
                </div>
            </div>
        );
    }

    const isArchived = article.status === "archived";

    return (
        <div className="min-h-screen bg-white">
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
                                onClick={() => setPublishDialogOpen(true)}
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
                                    startIcon={<ArchiveOutlined />}
                                    onClick={() => setArchiveDialogOpen(true)}
                                    disabled={archiveMutation.isPending}
                                    sx={{ textTransform: "none" }}
                                >
                                    Archive
                                </Button>
                            </>
                        )}
                        <Button
                            size="small"
                            variant="outlined"
                            startIcon={<SaveOutlined />}
                            onClick={() =>
                                save({ title, markdown, description })
                            }
                            disabled={
                                isArchived ||
                                saveStatus === "saved" ||
                                saveStatus === "saving"
                            }
                            sx={{ textTransform: "none" }}
                        >
                            Save
                        </Button>
                    </div>
                </div>

                {isArchived && (
                    <div className="mb-4 px-3 py-2 bg-amber-50 border border-amber-200 rounded text-sm text-amber-800">
                        This article is archived and is now read-only. It
                        stays accessible via its direct link for historical
                        references, but can no longer be edited.
                    </div>
                )}

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
                    disabled={isArchived}
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
                    placeholder="Description (optional)"
                    value={description}
                    onChange={(e) => {
                        if (e.target.value.length <= 250) {
                            setDescription(e.target.value);
                            setSaveStatus("unsaved");
                        }
                    }}
                    onBlur={handleDescriptionBlur}
                    disabled={isArchived}
                    multiline
                    maxRows={3}
                    helperText={
                        description.length >= 200
                            ? `${description.length}/250`
                            : " "
                    }
                    slotProps={{
                        input: {
                            sx: {
                                fontSize: "0.875rem",
                                color: "rgb(120 113 108)",
                            },
                            disableUnderline: true,
                        },
                        formHelperText: {
                            sx: {
                                textAlign: "right",
                                color:
                                    description.length >= 230
                                        ? "rgb(239 68 68)"
                                        : undefined,
                            },
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
                    disabled={isArchived}
                    isOptionEqualToValue={(option, value) =>
                        option.id === value.id
                    }
                    renderInput={(params) => (
                        <TextField
                            {...params}
                            variant="standard"
                            placeholder={
                                selectedTopics.length === 0
                                    ? "Add topics (max 5)"
                                    : ""
                            }
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
                <MemoizedEditor
                    ref={editorRef}
                    markdown={article.markdown}
                    onChange={handleMarkdownChange}
                    onInsertQuotationClick={openPicker}
                    readOnly={isArchived}
                />

                <QuotationPickerModal
                    open={pickerOpen}
                    onClose={() => setPickerOpen(false)}
                    onSelect={handleInsertQuotation}
                />

                <Dialog
                    open={publishDialogOpen}
                    onClose={() => setPublishDialogOpen(false)}
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
                            onClick={() => setPublishDialogOpen(false)}
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

                <Dialog
                    open={archiveDialogOpen}
                    onClose={() => setArchiveDialogOpen(false)}
                    maxWidth="sm"
                >
                    <DialogTitle>Archive this article?</DialogTitle>
                    <DialogContent>
                        <DialogContentText sx={{ fontSize: "0.875rem", mb: 1.5 }}>
                            Archiving is irreversible. Before archiving, you
                            can keep editing the article as long as you like.
                            Once archived:
                        </DialogContentText>
                        <ul className="text-sm text-stone-600 list-disc pl-5 space-y-1">
                            <li>
                                The article is removed from public listings
                            </li>
                            <li>
                                It remains accessible via its direct link, so
                                historical references keep working
                            </li>
                        </ul>
                    </DialogContent>
                    <DialogActions sx={{ px: 3, pb: 2 }}>
                        <Button
                            onClick={() => setArchiveDialogOpen(false)}
                            size="small"
                        >
                            Cancel
                        </Button>
                        <Button
                            onClick={handleArchive}
                            size="small"
                            variant="contained"
                            color="warning"
                            disabled={archiveMutation.isPending}
                        >
                            Archive
                        </Button>
                    </DialogActions>
                </Dialog>
            </div>
        </div>
    );
}
