import ArrowBackOutlined from "@mui/icons-material/ArrowBackOutlined";
import CloseOutlined from "@mui/icons-material/CloseOutlined";
import CommitOutlined from "@mui/icons-material/CommitOutlined";
import CompareOutlined from "@mui/icons-material/CompareOutlined";
import FormatQuoteOutlined from "@mui/icons-material/FormatQuoteOutlined";
import ListOutlined from "@mui/icons-material/ListOutlined";
import { IconButton } from "@mui/material";
import type React from "react";
import { useEffect, useRef, useState } from "react";
import { useListBooks } from "../api/books/books";
import type { SentenceResponse, TocNodeResponse } from "../api/model";
import { useGetToc } from "../api/toc/toc";
import { FootnotesView } from "./FootnotesView";
import { PanelToc } from "./PanelToc";
import { SentenceDetail } from "./SentenceDetail";

type ViewKind = "toc" | "compare" | "sentence" | "footnotes";

interface ResourcesPanelProps {
    toc: TocNodeResponse[] | undefined;
    bookSlug: string;
    activeNodeSlug: string | undefined;
    onNavigate?: (nodeSlug: string) => void;
    onAddComparisonPanel: (bookSlug: string, nodeSlug: string) => void;
    canAddPanel: boolean;
    selectedSentence: SentenceResponse | undefined;
    onClose: () => void;
    activeView: string | undefined;
    onViewChange: (view: string | undefined) => void;
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
    activeView,
    onViewChange,
}: ResourcesPanelProps) {
    // Auto-switch to sentence view when a new sentence is selected
    const prevSentenceRef = useRef(selectedSentence);
    useEffect(() => {
        if (selectedSentence && selectedSentence !== prevSentenceRef.current) {
            onViewChange("sentence");
        }
        prevSentenceRef.current = selectedSentence;
    }, [selectedSentence, onViewChange]);

    const viewKind = activeView as ViewKind | undefined;
    const isMenu = !viewKind;

    // For compare sub-navigation (picker vs compare-toc), keep internal state
    const [compareBookSlug, setCompareBookSlug] = useState<
        string | undefined
    >();

    // Reset compare sub-state when leaving compare view
    useEffect(() => {
        if (viewKind !== "compare") {
            setCompareBookSlug(undefined);
        }
    }, [viewKind]);

    return (
        <aside className="w-80 border-l border-stone-200 bg-white shrink-0 flex flex-col">
            {/* Header - matches TextPanel toolbar height */}
            <div className="border-b border-stone-200 bg-stone-50 shrink-0 py-2 flex items-center px-3">
                <IconButton
                    size="small"
                    onClick={() => onViewChange(undefined)}
                    title="Back to menu"
                    tabIndex={isMenu ? -1 : undefined}
                    sx={{ visibility: isMenu ? "hidden" : "visible", mr: 0.5 }}
                >
                    <ArrowBackOutlined fontSize="small" />
                </IconButton>
                <div className="flex-1 min-w-0">
                    <div className="text-sm text-stone-800 truncate">
                        Resources
                    </div>
                    <div className="text-xs text-stone-400 truncate">
                        {viewKind === "toc"
                            ? "Table of Contents"
                            : viewKind === "sentence"
                              ? "Sentence Details"
                              : viewKind === "compare"
                                ? "Compare Text"
                                : viewKind === "footnotes"
                                  ? "Footnotes"
                                  : "\u00A0"}
                    </div>
                </div>
                <IconButton
                    size="small"
                    onClick={onClose}
                    title="Close"
                >
                    <CloseOutlined fontSize="small" />
                </IconButton>
            </div>

            {/* Menu */}
            {isMenu && (
                <div className="flex-1 overflow-y-auto">
                    <nav className="p-2 space-y-1">
                        <MenuButton
                            onClick={() => onViewChange("toc")}
                            label="Table of Contents"
                            icon={<ListOutlined fontSize="small" />}
                        />
                        {canAddPanel && (
                            <MenuButton
                                onClick={() => onViewChange("compare")}
                                label="Compare Text"
                                icon={<CompareOutlined fontSize="small" />}
                            />
                        )}
                        <MenuButton
                            onClick={() => onViewChange("sentence")}
                            label="Sentence Details"
                            disabled={!selectedSentence}
                            icon={<CommitOutlined fontSize="small" />}
                        />
                        <MenuButton
                            onClick={() => onViewChange("footnotes")}
                            label="Footnotes"
                            disabled={
                                !selectedSentence ||
                                !selectedSentence.footnotes?.length
                            }
                            icon={
                                <FormatQuoteOutlined fontSize="small" />
                            }
                        />
                    </nav>
                </div>
            )}

            {/* Table of Contents view */}
            {viewKind === "toc" &&
                (toc ? (
                    <PanelToc
                        toc={toc}
                        bookSlug={bookSlug}
                        activeNodeSlug={activeNodeSlug}
                        onNavigate={onNavigate}
                    />
                ) : (
                    <div className="p-4 text-sm text-stone-400">Loading...</div>
                ))}

            {/* Sentence Details view */}
            {viewKind === "sentence" &&
                (selectedSentence ? (
                    <SentenceDetail sentence={selectedSentence} />
                ) : (
                    <div className="p-4 text-sm text-stone-400">
                        Click a sentence to see its details.
                    </div>
                ))}

            {/* Footnotes view */}
            {viewKind === "footnotes" &&
                (selectedSentence?.footnotes?.length ? (
                    <FootnotesView sentence={selectedSentence} />
                ) : (
                    <div className="p-4 text-sm text-stone-400">
                        Click a sentence with a footnote to see it here.
                    </div>
                ))}

            {/* Compare Text view */}
            {viewKind === "compare" &&
                (!compareBookSlug ? (
                    <BookPickerView
                        onPickBook={setCompareBookSlug}
                    />
                ) : (
                    <CompareTocView
                        compareBookSlug={compareBookSlug}
                        onSelectNode={(nodeSlug) =>
                            onAddComparisonPanel(compareBookSlug, nodeSlug)
                        }
                        onBack={() => setCompareBookSlug(undefined)}
                    />
                ))}
        </aside>
    );
}

function MenuButton({
    onClick,
    label,
    disabled,
    icon,
}: {
    onClick: () => void;
    label: string;
    disabled?: boolean;
    icon: React.ReactNode;
}) {
    return (
        <button
            onClick={onClick}
            disabled={disabled}
            className="w-full text-left text-sm px-3 py-2 rounded hover:bg-stone-100 text-stone-700 transition-colors disabled:text-stone-300 disabled:hover:bg-transparent disabled:cursor-default flex items-center gap-2"
        >
            <span className="text-stone-400">{icon}</span>
            {label}
        </button>
    );
}

function CompareTocView({
    compareBookSlug,
    onSelectNode,
    onBack,
}: {
    compareBookSlug: string;
    onSelectNode: (nodeSlug: string) => void;
    onBack: () => void;
}) {
    const { data: tocData } = useGetToc(compareBookSlug);
    const toc = tocData?.data;

    if (!toc) {
        return <div className="p-4 text-sm text-stone-400">Loading...</div>;
    }

    return (
        <>
            <div className="px-3 py-1.5 border-b border-stone-100 flex items-center">
                <button
                    onClick={onBack}
                    className="text-stone-400 hover:text-stone-600 mr-2 text-xs"
                >
                    &larr;
                </button>
                <span className="text-xs text-stone-500">Select section</span>
            </div>
            <PanelToc
                toc={toc}
                bookSlug={compareBookSlug}
                activeNodeSlug={undefined}
                onNavigate={onSelectNode}
            />
        </>
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
