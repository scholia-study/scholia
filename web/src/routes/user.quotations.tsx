import {
    FormControl,
    InputLabel,
    MenuItem,
    Select,
} from "@mui/material";
import { Link, createFileRoute, redirect } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { getGetProfileQueryOptions } from "../api/auth/auth";
import type { QuotationWithContextResponse } from "../api/model";
import { useListAllQuotations } from "../api/quotations/quotations";

export const Route = createFileRoute("/user/quotations")({
    beforeLoad: async ({ context }) => {
        const data = await context.queryClient.fetchQuery(
            getGetProfileQueryOptions(),
        );
        if (!data?.data) {
            throw redirect({ to: "/login" });
        }
    },
    component: QuotationsPage,
});

function sentenceLabel(q: QuotationWithContextResponse): string {
    const start = q.anchor_sentence_start_number;
    const end = q.anchor_sentence_end_number;
    if (end == null || end === start) return `Sentence ${start}`;
    return `Sentences ${start}–${end}`;
}

function QuotationsPage() {
    const [bookFilter, setBookFilter] = useState<string>("");

    // Fetch all quotations (unfiltered) to derive available books
    const { data: allQuotationsData, isLoading } = useListAllQuotations({});
    const allQuotations = allQuotationsData?.data?.quotations ?? [];

    // Derive book list from actual data
    const availableBooks = useMemo(() => {
        const map = new Map<string, string>();
        for (const q of allQuotations) {
            if (!map.has(q.book_slug)) {
                map.set(q.book_slug, q.book_title);
            }
        }
        return [...map.entries()].sort((a, b) => a[1].localeCompare(b[1]));
    }, [allQuotations]);

    // Apply client-side filter
    const quotations = useMemo(() => {
        if (!bookFilter) return allQuotations;
        return allQuotations.filter((q) => q.book_slug === bookFilter);
    }, [allQuotations, bookFilter]);

    // Group by book
    const grouped = useMemo(() => {
        const map = new Map<string, QuotationWithContextResponse[]>();
        for (const q of quotations) {
            const existing = map.get(q.book_slug) ?? [];
            existing.push(q);
            map.set(q.book_slug, existing);
        }
        return map;
    }, [quotations]);

    return (
        <div className="max-w-3xl mx-auto px-8 py-16">
            <div className="flex items-center justify-between mb-8">
                <h1 className="text-2xl font-bold text-stone-900">
                    My Quotations
                </h1>
                <FormControl size="small" sx={{ minWidth: 200 }}>
                    <InputLabel>Filter by book</InputLabel>
                    <Select
                        value={bookFilter}
                        label="Filter by book"
                        onChange={(e) => setBookFilter(e.target.value)}
                    >
                        <MenuItem value="">All books</MenuItem>
                        {availableBooks.map(([slug, title]) => (
                            <MenuItem key={slug} value={slug}>
                                {title}
                            </MenuItem>
                        ))}
                    </Select>
                </FormControl>
            </div>

            {isLoading && (
                <p className="text-sm text-stone-400">Loading...</p>
            )}

            {!isLoading && quotations.length === 0 && (
                <p className="text-sm text-stone-400">
                    No saved quotations yet.
                </p>
            )}

            {[...grouped.entries()].map(([bookSlug, quots]) => (
                <div key={bookSlug} className="mb-8">
                    <h2 className="text-sm font-semibold text-stone-500 uppercase tracking-wider mb-3">
                        {quots[0].book_title}
                    </h2>
                    <div className="space-y-2">
                        {quots.map((q) => (
                            <Link
                                key={q.id}
                                to="/books/$bookSlug/$nodeSlug"
                                params={{
                                    bookSlug: q.book_slug,
                                    nodeSlug: q.node_slug,
                                }}
                                search={{
                                    s: q.anchor_sentence_end_number && q.anchor_sentence_end_number !== q.anchor_sentence_start_number
                                        ? `${q.anchor_sentence_start_number}-${q.anchor_sentence_end_number}`
                                        : String(q.anchor_sentence_start_number),
                                    r: "1",
                                    rv: "notes",
                                }}
                                className="block border border-stone-300 rounded p-3 hover:shadow-md transition-all"
                            >
                                <div className="flex items-start justify-between gap-2">
                                    <div className="flex-1 min-w-0">
                                        <div className="text-xs text-stone-400 mb-1">
                                            {q.node_label} &middot;{" "}
                                            {sentenceLabel(q)}
                                        </div>
                                        {q.start_text_snippet && (
                                            <p className="text-sm text-stone-700 truncate">
                                                &ldquo;{q.start_text_snippet}
                                                &rdquo;
                                                {q.end_text_snippet && (
                                                    <span className="text-stone-400">
                                                        {" "}
                                                        &hellip; &ldquo;
                                                        {q.end_text_snippet}
                                                        &rdquo;
                                                    </span>
                                                )}
                                            </p>
                                        )}
                                    </div>
                                    <div className="text-right shrink-0">
                                        {q.note_count > 0 && (
                                            <span className="text-xs text-stone-400">
                                                {q.note_count} note
                                                {q.note_count > 1 ? "s" : ""}
                                            </span>
                                        )}
                                        <div className="text-[10px] text-stone-300 mt-0.5">
                                            {new Date(
                                                q.created_at,
                                            ).toLocaleDateString(undefined, {
                                                month: "short",
                                                day: "numeric",
                                                year: "numeric",
                                            })}
                                        </div>
                                    </div>
                                </div>
                            </Link>
                        ))}
                    </div>
                </div>
            ))}
        </div>
    );
}
