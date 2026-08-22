import SendOutlined from "@mui/icons-material/SendOutlined";
import { IconButton, TextField } from "@mui/material";
import { useState } from "react";
import type { ArticleReviewMessageResponse } from "#/api/model";

interface ChannelPanelProps {
    messages: ArticleReviewMessageResponse[];
    viewerUserId: string | null;
    busy: boolean;
    onSend: (body: string) => void;
    /** Who is on the other side of the channel, for the empty state. */
    audienceLabel?: string;
}

/**
 * The per-article, per-audience conversation between the author and the
 * reviewing side (editorial team or collegium), shared across review rounds.
 */
export function ChannelPanel({
    messages,
    viewerUserId,
    busy,
    onSend,
    audienceLabel = "the editorial team",
}: ChannelPanelProps) {
    const [draft, setDraft] = useState("");

    const send = () => {
        if (!draft.trim()) return;
        onSend(draft.trim());
        setDraft("");
    };

    return (
        <div className="border border-stone-200 rounded flex flex-col lg:h-full">
            <div className="shrink-0 px-3 py-2 border-b border-stone-200 text-xs uppercase tracking-wide text-stone-400">
                Conversation
            </div>
            <div className="max-h-80 lg:max-h-none lg:flex-1 min-h-0 overflow-y-auto flex flex-col gap-2 p-3">
                {messages.length === 0 && (
                    <p className="text-sm text-stone-400">
                        No messages yet. This channel is shared between the
                        author and {audienceLabel}.
                    </p>
                )}
                {messages.map((m) => {
                    const mine = m.sender?.user_id === viewerUserId;
                    return (
                        <div
                            key={m.id}
                            className={`max-w-[85%] rounded px-3 py-2 text-sm whitespace-pre-wrap ${
                                mine
                                    ? "self-end bg-sky-50 text-stone-800"
                                    : "self-start bg-stone-100 text-stone-800"
                            }`}
                        >
                            <div className="text-[0.65rem] text-stone-400 mb-0.5">
                                {m.sender?.display_name ?? "Former member"} ·{" "}
                                {new Date(m.created_at).toLocaleString(
                                    undefined,
                                    {
                                        month: "short",
                                        day: "numeric",
                                        hour: "2-digit",
                                        minute: "2-digit",
                                    },
                                )}
                            </div>
                            {m.body}
                        </div>
                    );
                })}
            </div>
            <div className="shrink-0 flex items-end gap-1 border-t border-stone-200 p-2">
                <TextField
                    value={draft}
                    onChange={(e) => setDraft(e.target.value)}
                    onKeyDown={(e) => {
                        if (e.key === "Enter" && !e.shiftKey) {
                            e.preventDefault();
                            send();
                        }
                    }}
                    placeholder="Write a message…"
                    size="small"
                    multiline
                    maxRows={4}
                    fullWidth
                />
                <IconButton
                    size="small"
                    color="primary"
                    disabled={busy || !draft.trim()}
                    onClick={send}
                >
                    <SendOutlined fontSize="small" />
                </IconButton>
            </div>
        </div>
    );
}
