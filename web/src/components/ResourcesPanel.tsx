import { useEffect, useRef, useState } from "react";
import type { SentenceResponse, TocNodeResponse } from "../api/model";
import { useListBooks } from "../api/books/books";
import { useGetToc } from "../api/toc/toc";
import { PanelToc } from "./PanelToc";
import { SentenceDetail } from "./SentenceDetail";

type View =
    | { kind: "sentence" }
    | { kind: "toc" }
    | { kind: "picker" }
    | { kind: "compare-toc"; compareBookSlug: string };

interface ResourcesPanelProps {
    toc: TocNodeResponse[] | undefined;
    bookSlug: string;
    activeNodeSlug: string | undefined;
    onNavigate?: (nodeSlug: string) => void;
    onAddComparisonPanel: (bookSlug: string, nodeSlug: string) => void;
    canAddPanel: boolean;
    selectedSentence: SentenceResponse | undefined;
    onClose: () => void;
}

export function ResourcesPanel({
    toc,
    bookSlug,
    activeNodeSlug,
    onNavigate,
    onAddComparisonPanel,
    canAddPanel,
    selectedSentence,
    onClose,
}: ResourcesPanelProps) {
    const [view, setView] = useState<View>(
        selectedSentence ? { kind: "sentence" } : { kind: "toc" },
    );

    // Auto-switch to sentence view when a new sentence is selected
    const prevSentenceRef = useRef(selectedSentence);
    useEffect(() => {
        if (selectedSentence && selectedSentence !== prevSentenceRef.current) {
            setView({ kind: "sentence" });
        }
        prevSentenceRef.current = selectedSentence;
    }, [selectedSentence]);

    // Fall back to toc if on sentence view but no sentence selected
    const effectiveView =
        view.kind === "sentence" && !selectedSentence
            ? { kind: "toc" as const }
            : view;

    const headerLabel =
        effectiveView.kind === "sentence"
            ? "Sentence"
            : effectiveView.kind === "toc"
              ? "Contents"
              : effectiveView.kind === "picker"
                ? "Compare"
                : "Select section";

    const canGoBack = effectiveView.kind !== "sentence";

    function handleBack() {
        if (effectiveView.kind === "compare-toc") {
            setView({ kind: "picker" });
        } else {
            setView(selectedSentence ? { kind: "sentence" } : { kind: "toc" });
        }
    }

    return (
        <aside className="w-64 border-l border-stone-200 bg-white shrink-0 flex flex-col">
            {/* Header */}
            <div className="flex items-center gap-1 px-2 py-1.5 border-b border-stone-200 shrink-0">
                {canGoBack && (
                    <button
                        onClick={handleBack}
                        className="text-xs text-stone-500 hover:text-stone-700 px-1"
                    >
                        &larr;
                    </button>
                )}
                <span className="text-xs text-stone-500 flex-1">
                    {headerLabel}
                </span>
                <button
                    onClick={onClose}
                    className="text-stone-400 hover:text-stone-600 text-lg leading-none px-1"
                    title="Close"
                >
                    &times;
                </button>
            </div>

            {/* Content */}
            {effectiveView.kind === "sentence" && selectedSentence && (
                <>
                    <SentenceDetail sentence={selectedSentence} />
                    <div className="p-2 border-t border-stone-200 space-y-1 shrink-0">
                        <button
                            onClick={() => setView({ kind: "toc" })}
                            className="w-full text-left text-sm px-2 py-1.5 rounded hover:bg-stone-100 text-stone-600 transition-colors"
                        >
                            Contents
                        </button>
                        {canAddPanel && (
                            <button
                                onClick={() => setView({ kind: "picker" })}
                                className="w-full text-left text-sm px-2 py-1.5 rounded hover:bg-stone-100 text-stone-600 transition-colors"
                            >
                                Compare text...
                            </button>
                        )}
                    </div>
                </>
            )}

            {effectiveView.kind === "toc" &&
                (toc ? (
                    <PanelToc
                        toc={toc}
                        bookSlug={bookSlug}
                        activeNodeSlug={activeNodeSlug}
                        onNavigate={onNavigate}
                    />
                ) : (
                    <div className="p-4 text-sm text-stone-400">
                        Loading...
                    </div>
                ))}

            {effectiveView.kind === "picker" && (
                <BookPickerView
                    onPickBook={(slug) =>
                        setView({ kind: "compare-toc", compareBookSlug: slug })
                    }
                />
            )}

            {effectiveView.kind === "compare-toc" && (
                <CompareTocView
                    compareBookSlug={effectiveView.compareBookSlug}
                    onSelectNode={(nodeSlug) =>
                        onAddComparisonPanel(
                            effectiveView.compareBookSlug,
                            nodeSlug,
                        )
                    }
                />
            )}
        </aside>
    );
}

function CompareTocView({
    compareBookSlug,
    onSelectNode,
}: {
    compareBookSlug: string;
    onSelectNode: (nodeSlug: string) => void;
}) {
    const { data: tocData } = useGetToc(compareBookSlug);
    const toc = tocData?.data;

    if (!toc) {
        return <div className="p-4 text-sm text-stone-400">Loading...</div>;
    }

    return (
        <PanelToc
            toc={toc}
            bookSlug={compareBookSlug}
            activeNodeSlug={undefined}
            onNavigate={onSelectNode}
        />
    );
}

function BookPickerView({
    onPickBook,
}: {
    onPickBook: (bookSlug: string) => void;
}) {
    const { data, isLoading, error } = useListBooks();
    const books = data?.data;

    return (
        <div className="flex-1 overflow-y-auto p-2">
            {isLoading && (
                <p className="text-stone-400 text-sm p-2">Loading...</p>
            )}
            {error ? (
                <p className="text-red-500 text-sm p-2">
                    Failed to load books.
                </p>
            ) : null}
            {books && (
                <ul className="space-y-1">
                    {books.map((book) => (
                        <li key={book.id}>
                            <button
                                onClick={() => onPickBook(book.slug)}
                                className="block w-full text-left px-2 py-1.5 rounded hover:bg-stone-100 transition-colors"
                            >
                                <div className="text-sm text-stone-900">
                                    {book.title}
                                </div>
                                <div className="text-xs text-stone-500">
                                    {book.author}
                                </div>
                            </button>
                        </li>
                    ))}
                </ul>
            )}
        </div>
    );
}
