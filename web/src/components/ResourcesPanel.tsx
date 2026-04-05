import ArrowBackOutlined from "@mui/icons-material/ArrowBackOutlined";
import CloseOutlined from "@mui/icons-material/CloseOutlined";
import CommitOutlined from "@mui/icons-material/CommitOutlined";
import CompareOutlined from "@mui/icons-material/CompareOutlined";
import MenuBookOutlined from "@mui/icons-material/MenuBookOutlined";
import ListOutlined from "@mui/icons-material/ListOutlined";
import { IconButton } from "@mui/material";
import type React from "react";
import { useEffect, useMemo, useState } from "react";
import { useListBooks } from "../api/books/books";
import type { FootnoteSentenceResponse, ResourceResponse, SentenceResponse, TocNodeResponse } from "../api/model";
import { useGetToc } from "../api/toc/toc";
import { useListResources } from "../api/resources/resources";
import { useAuth } from "../hooks/useAuth";
import { CommentaryView, getSentenceRange } from "./CommentaryView";
import { PanelToc } from "./PanelToc";
import { ResourceFormModal } from "./ResourceFormModal";
import { SentenceDetail } from "./SentenceDetail";

type ViewKind = "toc" | "compare" | "verbatim" | "paraphrase" | "allusion" | "sentence";

interface ResourcesPanelProps {
    toc: TocNodeResponse[] | undefined;
    bookSlug: string;
    activeNodeSlug: string | undefined;
    onNavigate?: (nodeSlug: string) => void;
    onAddComparisonPanel: (bookSlug: string, nodeSlug: string) => void;
    canAddPanel: boolean;
    selectedSentence: SentenceResponse | FootnoteSentenceResponse | (SentenceResponse | FootnoteSentenceResponse)[] | undefined;
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
    const { user } = useAuth();
    const isEditor =
        user?.roles?.includes("editor") || user?.roles?.includes("admin") || false;

    // Fetch resource counts for menu badges
    const sentenceRange = useMemo(
        () => getSentenceRange(selectedSentence),
        [selectedSentence],
    );
    const { data: resourcesData } = useListResources(
        bookSlug,
        {
            start: sentenceRange?.start ?? 0,
            end: sentenceRange?.end ?? 0,
            kind: sentenceRange?.kind ?? "body",
        },
        { query: { enabled: !!sentenceRange } },
    );
    const commentaryCounts = useMemo(() => {
        const all = resourcesData?.data?.resources ?? [];
        return {
            verbatim: all.filter((r) => r.resource_type === "verbatim").length,
            paraphrase: all.filter((r) => r.resource_type === "paraphrase").length,
            allusion: all.filter((r) => r.resource_type === "allusion").length,
        };
    }, [resourcesData]);

    // Modal state for resource create/edit (used by ResourceFormModal)
    const [resourceModalOpen, setResourceModalOpen] = useState(false);
    const [editingResource, setEditingResource] = useState<
        ResourceResponse | undefined
    >();
    const [modalDefaults, setModalDefaults] = useState<{
        type: "verbatim" | "paraphrase" | "allusion";
        start: number;
        end: number | undefined;
        kind: string;
    } | null>(null);

    const handleAddResource = (
        type: "verbatim" | "paraphrase" | "allusion",
        start: number,
        end: number | undefined,
        kind: string,
    ) => {
        setEditingResource(undefined);
        setModalDefaults({ type, start, end, kind });
        setResourceModalOpen(true);
    };

    const handleEditResource = (resource: ResourceResponse) => {
        setEditingResource(resource);
        setModalDefaults(null);
        setResourceModalOpen(true);
    };


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
                                : viewKind === "verbatim"
                                  ? "Verbatim Quotations"
                                  : viewKind === "paraphrase"
                                    ? "Paraphrases"
                                    : viewKind === "allusion"
                                      ? "Allusions"
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
                        <MenuButton
                            onClick={() => onViewChange("sentence")}
                            label="Sentence Details"
                            disabled={!selectedSentence}
                            icon={<CommitOutlined fontSize="small" />}
                        />
                        {canAddPanel && (
                            <MenuButton
                                onClick={() => onViewChange("compare")}
                                label="Compare Text"
                                icon={<CompareOutlined fontSize="small" />}
                            />
                        )}
                        <div className="text-[11px] uppercase tracking-wider text-stone-400 font-medium px-3 pt-3 pb-1">
                            Commentary
                        </div>
                        <MenuButton
                            onClick={() => onViewChange("verbatim")}
                            label={`Verbatim${commentaryCounts.verbatim ? ` (${commentaryCounts.verbatim})` : ""}`}
                            disabled={!selectedSentence}
                            icon={<MenuBookOutlined fontSize="small" sx={{ color: "#722f37" }} />}
                        />
                        <MenuButton
                            onClick={() => onViewChange("paraphrase")}
                            label={`Paraphrase${commentaryCounts.paraphrase ? ` (${commentaryCounts.paraphrase})` : ""}`}
                            disabled={!selectedSentence}
                            icon={<MenuBookOutlined fontSize="small" sx={{ color: "#5c6b8b" }} />}
                        />
                        <MenuButton
                            onClick={() => onViewChange("allusion")}
                            label={`Allusion${commentaryCounts.allusion ? ` (${commentaryCounts.allusion})` : ""}`}
                            disabled={!selectedSentence}
                            icon={<MenuBookOutlined fontSize="small" sx={{ color: "#5c7a5c" }} />}
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

            {/* Commentary views */}
            {(viewKind === "verbatim" ||
                viewKind === "paraphrase" ||
                viewKind === "allusion") && (
                <CommentaryView
                    bookSlug={bookSlug}
                    resourceType={viewKind}
                    selectedSentence={selectedSentence}
                    isEditor={isEditor}
                    onAdd={handleAddResource}
                    onEdit={handleEditResource}
                />
            )}

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

            {/* Resource create/edit modal */}
            <ResourceFormModal
                key={editingResource?.id ?? `${modalDefaults?.type}-${modalDefaults?.start}-${modalDefaults?.end}-${modalDefaults?.kind}`}
                open={resourceModalOpen}
                onClose={() => setResourceModalOpen(false)}
                bookSlug={bookSlug}
                mode={editingResource ? "edit" : "create"}
                initialData={editingResource}
                defaultType={modalDefaults?.type}
                defaultSentenceStart={modalDefaults?.start}
                defaultSentenceEnd={modalDefaults?.end}
                defaultSentenceKind={modalDefaults?.kind}
                isAdmin={user?.roles?.includes("admin") ?? false}
            />
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
