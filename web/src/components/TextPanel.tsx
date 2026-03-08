import TextFormatOutlined from "@mui/icons-material/TextFormatOutlined";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { SentenceResponse, TocNodeResponse } from "../api/model";
import { useGetBook } from "../api/books/books";
import { useGetNode } from "../api/nodes/nodes";
import { useGetToc } from "../api/toc/toc";
import type { MarginSettings } from "./BlockRenderer";
import { PanelContent } from "./PanelContent";
import type { PanelScrollViewHandle } from "./PanelScrollView";
import { PanelScrollView } from "./PanelScrollView";
import { ResourcesPanel } from "./ResourcesPanel";

type ViewMode = "section" | "scroll";

function findNodeLabel(
    nodes: TocNodeResponse[],
    slug: string,
): string | undefined {
    for (const node of nodes) {
        if (node.slug === slug) return node.label;
        const found = findNodeLabel(node.children, slug);
        if (found) return found;
    }
    return undefined;
}

interface TextPanelProps {
    panelIndex: number;
    bookSlug: string;
    nodeSlug: string | undefined;
    resourcesOpen: boolean;
    selectedSentenceId: string | undefined;
    onNavigate: (nodeSlug: string) => void;
    onSelectSentence: (sentenceId: string) => void;
    onToggleResources: () => void;
    onClose: (() => void) | undefined;
    onScrollNavigate: (nodeSlug: string) => void;
    onAddComparisonPanel: (bookSlug: string, nodeSlug: string) => void;
    canAddPanel: boolean;
}

function collectSystemsFromBlocks(
    blocks: { sentences: { page_markers: { system_slug: string }[] }[] }[],
): string[] {
    const systems = new Set<string>();
    for (const block of blocks) {
        for (const sentence of block.sentences) {
            for (const pm of sentence.page_markers) {
                systems.add(pm.system_slug);
            }
        }
    }
    return Array.from(systems);
}

export function TextPanel({
    bookSlug,
    nodeSlug,
    resourcesOpen,
    selectedSentenceId,
    onNavigate,
    onSelectSentence,
    onToggleResources,
    onClose,
    onScrollNavigate,
    onAddComparisonPanel,
    canAddPanel,
}: TextPanelProps) {
    const [viewMode, setViewMode] = useState<ViewMode>("scroll");
    const [visibleSlug, setVisibleSlug] = useState<string | undefined>();
    const [selectedSentence, setSelectedSentence] = useState<
        SentenceResponse | undefined
    >();
    const scrollViewRef = useRef<PanelScrollViewHandle>(null);

    // Margin annotation settings
    const [marginSettings, setMarginSettings] = useState<MarginSettings>({
        enabledSystems: new Set<string>(),
        systemSides: {},
    });
    const [displayOptionsOpen, setDisplayOptionsOpen] = useState(false);

    const handleVisibleNodeChange = useCallback(
        (slug: string) => {
            setVisibleSlug(slug);
            onScrollNavigate(slug);
        },
        [onScrollNavigate],
    );

    const { data: tocData } = useGetToc(bookSlug);
    const toc = tocData?.data;

    const { data: bookData } = useGetBook(bookSlug);
    const bookTitle = bookData?.status === 200 ? bookData.data.title : bookSlug;

    // In section mode, fetch the specific node
    const {
        data: nodeData,
        isLoading,
        error,
    } = useGetNode(bookSlug, nodeSlug ?? "", {
        query: { enabled: !!nodeSlug && viewMode === "section" },
    });
    const node =
        nodeSlug && viewMode === "section" && nodeData?.status === 200
            ? nodeData.data
            : undefined;

    // Discover reference systems from section-mode node data
    useEffect(() => {
        if (!node) return;
        const systems = collectSystemsFromBlocks(node.blocks);
        if (systems.length > 0) handleSystemsDiscovered(systems);
    }, [node]);

    const handleSystemsDiscovered = useCallback((systems: string[]) => {
        setMarginSettings((prev) => {
            let changed = false;
            const newEnabled = new Set(prev.enabledSystems);
            const newSides = { ...prev.systemSides };
            for (const s of systems) {
                if (!(s in newSides)) {
                    newEnabled.add(s);
                    newSides[s] = "right";
                    changed = true;
                }
            }
            if (!changed) return prev;
            return { enabledSystems: newEnabled, systemSides: newSides };
        });
    }, []);

    const handleToggleSystem = useCallback((slug: string) => {
        setMarginSettings((prev) => {
            const newEnabled = new Set(prev.enabledSystems);
            if (newEnabled.has(slug)) newEnabled.delete(slug);
            else newEnabled.add(slug);
            return { ...prev, enabledSystems: newEnabled };
        });
    }, []);

    const handleToggleSide = useCallback((slug: string) => {
        setMarginSettings((prev) => ({
            ...prev,
            systemSides: {
                ...prev.systemSides,
                [slug]: prev.systemSides[slug] === "left" ? "right" : "left",
            },
        }));
    }, []);

    const activeNodeSlug = viewMode === "scroll" ? visibleSlug : nodeSlug;
    const activeNodeLabel = useMemo(
        () => (activeNodeSlug && toc ? findNodeLabel(toc, activeNodeSlug) : undefined),
        [activeNodeSlug, toc],
    );
    const showSentenceDetail =
        selectedSentence != null && selectedSentence.id === selectedSentenceId;
    const availableSystems = Object.keys(marginSettings.systemSides);

    const handleSelectSentence = useCallback(
        (sentence: SentenceResponse) => {
            setSelectedSentence(sentence);
            onSelectSentence(sentence.id);
        },
        [onSelectSentence],
    );

    const handleToggleView = useCallback(() => {
        setViewMode((prev) => {
            if (prev === "scroll" && visibleSlug) {
                onNavigate(visibleSlug);
            }
            return prev === "section" ? "scroll" : "section";
        });
    }, [visibleSlug, onNavigate]);

    const handleTocNavigate = useCallback(
        (slug: string) => {
            if (viewMode === "scroll") {
                scrollViewRef.current?.scrollToNode(slug);
            } else {
                onNavigate(slug);
            }
        },
        [viewMode, onNavigate],
    );

    return (
        <div className="flex flex-1 min-w-0 border-r border-stone-200 last:border-r-0">
            {/* Main content area */}
            <div className="flex-1 flex flex-col min-w-0">
                {/* Toolbar */}
                <div className="border-b border-stone-200 bg-white shrink-0 py-2 relative z-10">
                    <div className="relative max-w-4xl mx-auto">
                        {/* Centered title */}
                        <div className="text-center">
                            <div className="text-sm text-stone-800 truncate">
                                {activeNodeLabel ?? bookTitle}
                            </div>
                            <div className="text-xs text-stone-400 truncate">
                                {bookTitle}
                            </div>
                        </div>

                        {/* Display options button — aligned above right margin notes */}
                        <div className="absolute top-1/2 -translate-y-1/2 flex items-center gap-1" style={{ left: "calc(50% + 21rem + 0.5rem)" }}>
                            <div className="relative">
                                <button
                                    onClick={() =>
                                        setDisplayOptionsOpen(
                                            !displayOptionsOpen,
                                        )
                                    }
                                    className="text-stone-500 hover:text-stone-700 transition-colors p-1 rounded hover:bg-stone-100"
                                    title="Text display options"
                                >
                                    <TextFormatOutlined fontSize="small" />
                                </button>
                                {displayOptionsOpen && (
                                    <div className="absolute top-full mt-1 right-0 bg-white border border-stone-200 rounded-lg shadow-lg p-2 z-50 min-w-[12rem]">
                                        {/* View mode */}
                                        <div className="text-[10px] uppercase tracking-wider text-stone-400 px-1 pb-1 mb-1 border-b border-stone-100">
                                            View mode
                                        </div>
                                        <div className="flex items-center gap-1 mb-2 px-1">
                                            <button
                                                onClick={() => {
                                                    if (viewMode !== "scroll")
                                                        handleToggleView();
                                                }}
                                                className={`text-xs px-2 py-1 rounded transition-colors ${
                                                    viewMode === "scroll"
                                                        ? "bg-stone-200 text-stone-900 font-medium"
                                                        : "text-stone-600 hover:bg-stone-100"
                                                }`}
                                            >
                                                Scroll
                                            </button>
                                            <button
                                                onClick={() => {
                                                    if (viewMode !== "section")
                                                        handleToggleView();
                                                }}
                                                className={`text-xs px-2 py-1 rounded transition-colors ${
                                                    viewMode === "section"
                                                        ? "bg-stone-200 text-stone-900 font-medium"
                                                        : "text-stone-600 hover:bg-stone-100"
                                                }`}
                                            >
                                                Section
                                            </button>
                                        </div>

                                        {/* Margin references */}
                                        {availableSystems.length > 0 && (
                                            <>
                                                <div className="text-[10px] uppercase tracking-wider text-stone-400 px-1 pb-1 mb-1 border-b border-stone-100">
                                                    Margin references
                                                </div>
                                                {availableSystems.map(
                                                    (slug) => (
                                                        <div
                                                            key={slug}
                                                            className="flex items-center gap-2 py-1 px-1"
                                                        >
                                                            <label className="flex items-center gap-1.5 flex-1 text-xs text-stone-700 cursor-pointer">
                                                                <input
                                                                    type="checkbox"
                                                                    checked={marginSettings.enabledSystems.has(
                                                                        slug,
                                                                    )}
                                                                    onChange={() =>
                                                                        handleToggleSystem(
                                                                            slug,
                                                                        )
                                                                    }
                                                                    className="rounded border-stone-300"
                                                                />
                                                                {slug}
                                                            </label>
                                                            <button
                                                                onClick={() =>
                                                                    handleToggleSide(
                                                                        slug,
                                                                    )
                                                                }
                                                                className="text-[10px] px-1.5 py-0.5 rounded border border-stone-200 text-stone-500 hover:bg-stone-50 font-mono"
                                                                title={`Move to ${marginSettings.systemSides[slug] === "left" ? "right" : "left"} margin`}
                                                            >
                                                                {marginSettings
                                                                    .systemSides[
                                                                    slug
                                                                ] === "left"
                                                                    ? "L"
                                                                    : "R"}
                                                            </button>
                                                        </div>
                                                    ),
                                                )}
                                            </>
                                        )}
                                    </div>
                                )}
                            </div>
                            {onClose && (
                                <button
                                    onClick={onClose}
                                    className="text-stone-400 hover:text-stone-600 text-lg leading-none"
                                    title="Close panel"
                                >
                                    &times;
                                </button>
                            )}
                        </div>
                    </div>
                </div>

                {/* Content */}
                {viewMode === "scroll" ? (
                    <PanelScrollView
                        ref={scrollViewRef}
                        bookSlug={bookSlug}
                        initialNodeSlug={nodeSlug}
                        selectedSentenceId={selectedSentenceId}
                        onSelectSentence={handleSelectSentence}
                        onVisibleNodeChange={handleVisibleNodeChange}
                        onSystemsDiscovered={handleSystemsDiscovered}
                        marginSettings={marginSettings}
                    />
                ) : (
                    <div className="flex-1 overflow-y-auto">
                        {!nodeSlug ? (
                            <div className="flex items-center justify-center h-full text-stone-400">
                                <p>
                                    Select a section from the table of contents.
                                </p>
                            </div>
                        ) : isLoading ? (
                            <div className="flex items-center justify-center h-full text-stone-400">
                                <p>Loading...</p>
                            </div>
                        ) : error ? (
                            <div className="flex items-center justify-center h-full text-red-500">
                                <p>Failed to load content.</p>
                            </div>
                        ) : node ? (
                            <PanelContent
                                node={node}
                                selectedSentenceId={selectedSentenceId}
                                onSelectSentence={handleSelectSentence}
                                marginSettings={marginSettings}
                            />
                        ) : null}
                    </div>
                )}
            </div>

            {/* Resources panel */}
            {resourcesOpen && (
                <ResourcesPanel
                    toc={toc ?? undefined}
                    bookSlug={bookSlug}
                    activeNodeSlug={activeNodeSlug}
                    onNavigate={handleTocNavigate}
                    onAddComparisonPanel={onAddComparisonPanel}
                    canAddPanel={canAddPanel}
                    selectedSentence={showSentenceDetail ? selectedSentence : undefined}
                    onClose={onToggleResources}
                />
            )}
        </div>
    );
}
