import AddOutlined from "@mui/icons-material/AddOutlined";
import StarOutlined from "@mui/icons-material/StarOutlined";
import { IconButton } from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import toast from "react-hot-toast";
import type {
    FootnoteSentenceResponse,
    ResourceResponse,
    SentenceResponse,
} from "../../../api/model";
import {
    getListResourcesQueryKey,
    useDeleteResource,
    useListResources,
} from "../../../api/resources/resources";
import { CommentaryEntry } from "./CommentaryEntry";

type ResourceType = "verbatim" | "paraphrase" | "allusion";

interface CommentaryViewProps {
    bookSlug: string;
    resourceType: ResourceType;
    selectedSentence:
        | SentenceResponse
        | FootnoteSentenceResponse
        | (SentenceResponse | FootnoteSentenceResponse)[]
        | undefined;
    isEditor: boolean;
    onAdd: (
        type: ResourceType,
        start: number,
        end: number | undefined,
        kind: string,
    ) => void;
    onEdit: (resource: ResourceResponse) => void;
}

export function getSentenceRange(
    selectedSentence: CommentaryViewProps["selectedSentence"],
): { start: number; end: number; kind: string } | null {
    if (!selectedSentence) return null;

    if (Array.isArray(selectedSentence)) {
        const numbers = selectedSentence
            .map((s) => s.sentence_number)
            .filter((n): n is number => n != null);
        if (numbers.length === 0) return null;
        const start = Math.min(...numbers);
        const end = Math.max(...numbers);
        // Determine kind from first sentence
        const first = selectedSentence[0];
        const kind = "page_markers" in first ? "body" : "footnote";
        return { start, end, kind };
    }

    const num = selectedSentence.sentence_number;
    if (num == null) return null;
    const kind = "page_markers" in selectedSentence ? "body" : "footnote";
    return { start: num, end: num, kind };
}

const TYPE_LABELS: Record<ResourceType, string> = {
    verbatim: "Verbatim Quotations",
    paraphrase: "Paraphrases",
    allusion: "Allusions",
};

export function CommentaryView({
    bookSlug,
    resourceType,
    selectedSentence,
    isEditor,
    onAdd,
    onEdit,
}: CommentaryViewProps) {
    const [searchQuery, setSearchQuery] = useState("");
    const [featuredOnly, setFeaturedOnly] = useState(false);

    const range = getSentenceRange(selectedSentence);

    const { data, isLoading } = useListResources(
        bookSlug,
        {
            start: range?.start ?? 0,
            end: range?.end ?? 0,
            kind: range?.kind ?? "body",
        },
        { query: { enabled: !!range } },
    );

    const queryClient = useQueryClient();
    const deleteMutation = useDeleteResource({
        mutation: {
            onSuccess: () => {
                toast.success("Resource archived");
                if (range) {
                    queryClient.invalidateQueries({
                        queryKey: getListResourcesQueryKey(bookSlug, {
                            start: range.start,
                            end: range.end,
                            kind: range.kind,
                        }),
                    });
                }
            },
            onError: () => {
                toast.error("Failed to delete resource");
            },
        },
    });

    const resources = useMemo(() => {
        const all = data?.data?.resources ?? [];
        // Filter by resource type
        let filtered = all.filter((r) => r.resource_type === resourceType);

        // Apply featured filter
        if (featuredOnly) {
            filtered = filtered.filter((r) => r.is_featured);
        }

        // Apply search filter
        if (searchQuery.trim()) {
            const q = searchQuery.toLowerCase();
            filtered = filtered.filter((r) => {
                const source = r.source;
                if (!source) return false;
                const titleMatch = source.title.toLowerCase().includes(q);
                const personMatch = source.persons.some((p) =>
                    p.name.toLowerCase().includes(q),
                );
                return titleMatch || personMatch;
            });
        }

        return filtered;
    }, [data, resourceType, featuredOnly, searchQuery]);

    if (!range) {
        return (
            <div className="flex-1 overflow-y-auto p-4">
                <p className="text-sm text-stone-400">
                    Select a sentence to view{" "}
                    {TYPE_LABELS[resourceType].toLowerCase()}.
                </p>
            </div>
        );
    }

    const handleDelete = (resource: ResourceResponse) => {
        if (window.confirm("Archive this resource entry?")) {
            deleteMutation.mutate({ slug: bookSlug, id: resource.id });
        }
    };

    return (
        <div className="flex-1 overflow-y-auto flex flex-col">
            {/* Toolbar */}
            <div className="px-3 py-2 border-b border-stone-100 flex items-center gap-2 shrink-0">
                <input
                    type="text"
                    placeholder="Search by author or title..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="flex-1 text-xs px-2 py-1 border border-stone-200 rounded bg-white text-stone-700 placeholder:text-stone-300 focus:outline-none focus:border-stone-400"
                />
                <IconButton
                    size="small"
                    onClick={() => setFeaturedOnly(!featuredOnly)}
                    title={featuredOnly ? "Show all" : "Show featured only"}
                    sx={{
                        color: featuredOnly
                            ? "rgb(202 138 4)"
                            : "rgb(168 162 158)",
                    }}
                >
                    <StarOutlined fontSize="small" />
                </IconButton>
                {isEditor && (
                    <IconButton
                        size="small"
                        onClick={() =>
                            onAdd(
                                resourceType,
                                range.start,
                                range.start !== range.end
                                    ? range.end
                                    : undefined,
                                range.kind,
                            )
                        }
                        title="Add entry"
                    >
                        <AddOutlined fontSize="small" />
                    </IconButton>
                )}
            </div>

            {/* List */}
            <div className="flex-1 overflow-y-auto p-2 space-y-1.5">
                {isLoading && (
                    <p className="text-sm text-stone-400 p-2">Loading...</p>
                )}

                {!isLoading && resources.length === 0 && (
                    <p className="text-sm text-stone-400 p-2">
                        {searchQuery || featuredOnly
                            ? "No matching entries."
                            : `No ${TYPE_LABELS[resourceType].toLowerCase()} for this selection.`}
                    </p>
                )}

                {resources.map((resource) => (
                    <CommentaryEntry
                        key={resource.id}
                        resource={resource}
                        isEditor={isEditor}
                        onEdit={onEdit}
                        onDelete={handleDelete}
                    />
                ))}
            </div>
        </div>
    );
}
