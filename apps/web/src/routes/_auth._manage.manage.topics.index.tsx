import { Button, TextField, Tooltip } from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute, notFound } from "@tanstack/react-router";
import { useState } from "react";
import toast from "react-hot-toast";
import {
    getAdminListTopicsQueryKey,
    useAdminListTopics,
    useCreateTopic,
    useDeleteTopic,
    useUpdateTopic,
} from "../api/admin/admin";
import { getMeQueryOptions } from "../api/auth/auth";
import { FetchError } from "../api/fetcher";
import type { TopicAdminResponse } from "../api/model";

export const Route = createFileRoute("/_auth/_manage/manage/topics/")({
    beforeLoad: async ({ context }) => {
        const me = await context.queryClient.fetchQuery(getMeQueryOptions());
        if (!me?.data?.permissions?.includes("admin_panel")) {
            throw notFound();
        }
    },
    component: TopicsPage,
});

function errorMessage(err: unknown, fallback: string) {
    return err instanceof FetchError && err.message ? err.message : fallback;
}

function TopicsPage() {
    const queryClient = useQueryClient();
    const { data, isLoading } = useAdminListTopics();
    const topics = data?.data?.topics ?? [];

    const invalidate = () =>
        queryClient.invalidateQueries({
            queryKey: getAdminListTopicsQueryKey(),
        });

    const [newName, setNewName] = useState("");
    const createMutation = useCreateTopic({
        mutation: {
            onSuccess: () => {
                setNewName("");
                invalidate();
            },
            onError: (err) =>
                toast.error(errorMessage(err, "Failed to create topic")),
        },
    });

    const handleCreate = () => {
        const name = newName.trim();
        if (!name) return;
        createMutation.mutate({ data: { name } });
    };

    return (
        <div className="max-w-3xl mx-auto px-8 py-16">
            <h1 className="text-lg font-semibold text-stone-800 mb-1">
                Topics
            </h1>
            <p className="text-sm text-stone-500 mb-6">
                The article topic vocabulary. Renaming changes the display name
                only — the slug is generated once and never changes, since it
                lives in shared URLs. Deleting requires the topic to be unused.
            </p>

            <div className="flex gap-2 mb-8">
                <TextField
                    label="New topic"
                    value={newName}
                    onChange={(e) => setNewName(e.target.value)}
                    onKeyDown={(e) => {
                        if (e.key === "Enter") handleCreate();
                    }}
                    size="small"
                    sx={{ flex: 1 }}
                />
                <Button
                    variant="contained"
                    size="small"
                    onClick={handleCreate}
                    disabled={!newName.trim() || createMutation.isPending}
                    sx={{ textTransform: "none" }}
                >
                    Add
                </Button>
            </div>

            {isLoading && <p className="text-sm text-stone-400">Loading…</p>}
            {!isLoading && topics.length === 0 && (
                <p className="text-sm text-stone-400">No topics yet.</p>
            )}
            <ul className="divide-y divide-stone-100 border border-stone-200 rounded">
                {topics.map((topic) => (
                    <TopicRow
                        key={topic.id}
                        topic={topic}
                        onChanged={invalidate}
                    />
                ))}
            </ul>
        </div>
    );
}

function TopicRow({
    topic,
    onChanged,
}: {
    topic: TopicAdminResponse;
    onChanged: () => void;
}) {
    const [editing, setEditing] = useState(false);
    const [name, setName] = useState(topic.name);

    const updateMutation = useUpdateTopic({
        mutation: {
            onSuccess: () => {
                setEditing(false);
                onChanged();
            },
            onError: (err) =>
                toast.error(errorMessage(err, "Failed to rename topic")),
        },
    });
    const deleteMutation = useDeleteTopic({
        mutation: {
            onSuccess: onChanged,
            onError: (err) =>
                toast.error(errorMessage(err, "Failed to delete topic")),
        },
    });

    const handleRename = () => {
        const trimmed = name.trim();
        if (!trimmed || trimmed === topic.name) {
            setEditing(false);
            setName(topic.name);
            return;
        }
        updateMutation.mutate({ id: topic.id, data: { name: trimmed } });
    };

    const inUse = topic.article_count > 0;

    return (
        <li className="flex items-center gap-3 px-4 py-2.5">
            <div className="flex-1 min-w-0">
                {editing ? (
                    <TextField
                        value={name}
                        onChange={(e) => setName(e.target.value)}
                        onKeyDown={(e) => {
                            if (e.key === "Enter") handleRename();
                            if (e.key === "Escape") {
                                setEditing(false);
                                setName(topic.name);
                            }
                        }}
                        size="small"
                        autoFocus
                        fullWidth
                        slotProps={{
                            input: { style: { fontSize: "0.875rem" } },
                        }}
                    />
                ) : (
                    <span className="text-sm text-stone-800">{topic.name}</span>
                )}
                <span
                    className="ml-2 text-[0.7rem] font-mono text-stone-400 bg-stone-50 border border-stone-200 rounded px-1 py-0.5"
                    title="URL slug — fixed at creation, unaffected by rename"
                >
                    {topic.slug}
                </span>
            </div>
            <span className="text-xs text-stone-400 tabular-nums shrink-0">
                {topic.article_count}{" "}
                {topic.article_count === 1 ? "article" : "articles"}
            </span>
            {editing ? (
                <Button
                    size="small"
                    onClick={handleRename}
                    disabled={updateMutation.isPending}
                    sx={{ textTransform: "none", fontSize: "0.75rem" }}
                >
                    Save
                </Button>
            ) : (
                <Button
                    size="small"
                    onClick={() => setEditing(true)}
                    sx={{ textTransform: "none", fontSize: "0.75rem" }}
                >
                    Rename
                </Button>
            )}
            <Tooltip
                title={inUse ? "Remove the topic from its articles first" : ""}
            >
                <span>
                    <Button
                        size="small"
                        color="error"
                        onClick={() => {
                            if (confirm(`Delete topic "${topic.name}"?`)) {
                                deleteMutation.mutate({ id: topic.id });
                            }
                        }}
                        disabled={inUse || deleteMutation.isPending}
                        sx={{ textTransform: "none", fontSize: "0.75rem" }}
                    >
                        Delete
                    </Button>
                </span>
            </Tooltip>
        </li>
    );
}
