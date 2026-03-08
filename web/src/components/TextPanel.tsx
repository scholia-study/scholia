import { useCallback, useEffect, useRef, useState } from "react";
import type { SentenceResponse } from "../api/model";
import { useGetNode } from "../api/nodes/nodes";
import { useGetToc } from "../api/toc/toc";
import type { MarginSettings } from "./BlockRenderer";
import { PanelContent } from "./PanelContent";
import type { PanelScrollViewHandle } from "./PanelScrollView";
import { PanelScrollView } from "./PanelScrollView";
import { PanelToc } from "./PanelToc";
import { SentenceDetail } from "./SentenceDetail";

type ViewMode = "section" | "scroll";

interface TextPanelProps {
    panelIndex: number;
    bookSlug: string;
    nodeSlug: string | undefined;
    tocOpen: boolean;
    selectedSentenceId: string | undefined;
    onNavigate: (nodeSlug: string) => void;
    onSelectSentence: (sentenceId: string) => void;
    onDeselectSentence: () => void;
    onToggleToc: () => void;
    onClose: (() => void) | undefined;
    onScrollNavigate: (nodeSlug: string) => void;
    isOnly: boolean;
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
    tocOpen,
    selectedSentenceId,
    onNavigate,
    onSelectSentence,
    onDeselectSentence,
    onToggleToc,
    onClose,
    onScrollNavigate,
}: TextPanelProps) {
    const [viewMode, setViewMode] = useState<ViewMode>("section");
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
    const [refsOpen, setRefsOpen] = useState(false);

    const handleVisibleNodeChange = useCallback(
        (slug: string) => {
            setVisibleSlug(slug);
            onScrollNavigate(slug);
        },
        [onScrollNavigate],
    );

    const { data: tocData } = useGetToc(bookSlug);
    const toc = tocData?.data;

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

    const handleDeselectSentence = useCallback(() => {
        setSelectedSentence(undefined);
        onDeselectSentence();
    }, [onDeselectSentence]);

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
            {/* TOC sidebar */}
            {tocOpen && toc ? (
                <PanelToc
                    toc={toc}
                    bookSlug={bookSlug}
                    activeNodeSlug={activeNodeSlug}
                    onNavigate={handleTocNavigate}
                />
            ) : null}

            {/* Main content area */}
            <div className="flex-1 flex flex-col min-w-0">
                {/* Toolbar */}
                <div className="flex items-center gap-2 px-3 py-2 border-b border-stone-200 bg-white shrink-0">
                    <button
                        onClick={onToggleToc}
                        className="text-xs px-2 py-1 rounded border border-stone-300 text-stone-600 hover:bg-stone-100 transition-colors"
                        title={tocOpen ? "Hide TOC" : "Show TOC"}
                    >
                        {tocOpen ? "\u25C0" : "\u2630"}
                    </button>
                    <span className="text-sm text-stone-500 truncate flex-1">
                        {node?.label ?? bookSlug}
                    </span>
                    {availableSystems.length > 0 && (
                        <div className="relative">
                            <button
                                onClick={() => setRefsOpen(!refsOpen)}
                                className={`text-xs px-2 py-1 rounded border transition-colors ${
                                    marginSettings.enabledSystems.size > 0
                                        ? "border-amber-300 bg-amber-50 text-amber-700 hover:bg-amber-100"
                                        : "border-stone-300 text-stone-600 hover:bg-stone-100"
                                }`}
                                title="Reference annotations"
                            >
                                Refs
                            </button>
                            {refsOpen && (
                                <div className="absolute top-full mt-1 right-0 bg-white border border-stone-200 rounded-lg shadow-lg p-2 z-20 min-w-[10rem]">
                                    <div className="text-[10px] uppercase tracking-wider text-stone-400 px-1 pb-1 mb-1 border-b border-stone-100">
                                        Margin references
                                    </div>
                                    {availableSystems.map((slug) => (
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
                                                        handleToggleSystem(slug)
                                                    }
                                                    className="rounded border-stone-300"
                                                />
                                                {slug}
                                            </label>
                                            <button
                                                onClick={() =>
                                                    handleToggleSide(slug)
                                                }
                                                className="text-[10px] px-1.5 py-0.5 rounded border border-stone-200 text-stone-500 hover:bg-stone-50 font-mono"
                                                title={`Move to ${marginSettings.systemSides[slug] === "left" ? "right" : "left"} margin`}
                                            >
                                                {marginSettings.systemSides[
                                                    slug
                                                ] === "left"
                                                    ? "L"
                                                    : "R"}
                                            </button>
                                        </div>
                                    ))}
                                </div>
                            )}
                        </div>
                    )}
                    <button
                        onClick={handleToggleView}
                        className="text-xs px-2 py-1 rounded border border-stone-300 text-stone-600 hover:bg-stone-100 transition-colors"
                        title={
                            viewMode === "section"
                                ? "Switch to scroll view"
                                : "Switch to section view"
                        }
                    >
                        {viewMode === "section" ? "Scroll" : "Section"}
                    </button>
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

                {/* Content */}
                {viewMode === "scroll" ? (
                    <PanelScrollView
                        ref={scrollViewRef}
                        bookSlug={bookSlug}
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

            {/* Sentence detail */}
            {showSentenceDetail && (
                <SentenceDetail
                    sentence={selectedSentence}
                    onClose={handleDeselectSentence}
                />
            )}
        </div>
    );
}
