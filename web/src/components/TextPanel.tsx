import CloseOutlined from "@mui/icons-material/CloseOutlined";
import TextFormatOutlined from "@mui/icons-material/TextFormatOutlined";
import {
    Checkbox,
    Divider,
    FormControlLabel,
    IconButton,
    ListItemText,
    Menu,
    MenuItem,
    Switch,
    ToggleButton,
    ToggleButtonGroup,
    Typography,
} from "@mui/material";
import { useCallback, useMemo, useRef, useState } from "react";
import { useGetBook } from "../api/books/books";
import type { SentenceResponse, TocNodeResponse } from "../api/model";
import { useGetToc } from "../api/toc/toc";
import {
    type MarginSettings,
    sentenceKey,
    sentenceMatchesKey,
} from "./BlockRenderer";
import type { PanelScrollViewHandle } from "./PanelScrollView";
import { PanelScrollView } from "./PanelScrollView";
import { ResourcesPanel } from "./ResourcesPanel";

function findNodeInToc(
    nodes: TocNodeResponse[],
    slug: string,
): TocNodeResponse | undefined {
    for (const node of nodes) {
        if (node.slug === slug) return node;
        const found = findNodeInToc(node.children, slug);
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
    showOriginal: boolean;
    onSelectSentence: (sentenceId: string) => void;
    onToggleOriginal: () => void;
    onToggleResources: () => void;
    onClose: () => void;
    onScrollNavigate: (nodeSlug: string) => void;
    onAddComparisonPanel: (bookSlug: string, nodeSlug: string) => void;
    canAddPanel: boolean;
}

export function TextPanel({
    bookSlug,
    nodeSlug,
    resourcesOpen,
    selectedSentenceId,
    showOriginal,
    onSelectSentence,
    onToggleOriginal,
    onToggleResources,
    onClose,
    onScrollNavigate,
    onAddComparisonPanel,
    canAddPanel,
}: TextPanelProps) {
    const [visibleSlug, setVisibleSlug] = useState<string | undefined>(
        nodeSlug,
    );
    const [selectedSentence, setSelectedSentence] = useState<
        SentenceResponse | undefined
    >();
    const scrollViewRef = useRef<PanelScrollViewHandle>(null);

    // Margin annotation settings
    const [marginSettings, setMarginSettings] = useState<MarginSettings>({
        enabledSystems: new Set<string>(),
        systemSides: {},
    });
    const [menuAnchor, setMenuAnchor] = useState<HTMLElement | null>(null);

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

    const activeNodeSlug = visibleSlug;
    const activeNodeLabel = useMemo(
        () =>
            activeNodeSlug && toc
                ? findNodeInToc(toc, activeNodeSlug)?.label
                : undefined,
        [activeNodeSlug, toc],
    );
    const initialSortOrder = useMemo(
        () =>
            nodeSlug && toc
                ? findNodeInToc(toc, nodeSlug)?.sort_order
                : undefined,
        [nodeSlug, toc],
    );
    const showSentenceDetail =
        selectedSentence != null &&
        sentenceMatchesKey(selectedSentence, selectedSentenceId);
    const availableSystems = Object.keys(marginSettings.systemSides);

    const handleSelectSentence = useCallback(
        (sentence: SentenceResponse) => {
            setSelectedSentence(sentence);
            onSelectSentence(sentenceKey(sentence));
        },
        [onSelectSentence],
    );

    const handleTocNavigate = useCallback(
        (slug: string) => {
            const sortOrder = toc
                ? findNodeInToc(toc, slug)?.sort_order
                : undefined;
            scrollViewRef.current?.scrollToNode(slug, sortOrder);
        },
        [toc],
    );

    return (
        <div className="flex flex-1 min-w-0 border-r border-stone-200 last:border-r-0">
            {/* Main content area */}
            <div className="flex-1 flex flex-col min-w-0">
                {/* Toolbar */}
                <div className="border-b border-stone-200 bg-stone-50 shrink-0 py-2 relative z-10">
                    <div className="relative max-w-4xl mx-auto">
                        <div className="text-center">
                            <div className="text-sm text-stone-800 truncate">
                                {activeNodeLabel ?? bookTitle}
                            </div>
                            <div className="text-xs text-stone-400 truncate">
                                {bookTitle}
                            </div>
                        </div>

                        <div
                            className="absolute top-1/2 -translate-y-1/2 flex items-center"
                            style={{ right: "calc(50% + 19rem + 0.5rem)" }}
                        >
                            <IconButton
                                size="small"
                                onClick={onClose}
                                title="Close panel"
                            >
                                <CloseOutlined fontSize="small" />
                            </IconButton>
                        </div>

                        <div
                            className="absolute top-1/2 -translate-y-1/2 flex items-center gap-1"
                            style={{ left: "calc(50% + 19rem + 0.5rem)" }}
                        >
                            <IconButton
                                size="small"
                                onClick={(e) => setMenuAnchor(e.currentTarget)}
                                title="Text display options"
                            >
                                <TextFormatOutlined fontSize="small" />
                            </IconButton>
                            <Menu
                                anchorEl={menuAnchor}
                                open={Boolean(menuAnchor)}
                                onClose={() => setMenuAnchor(null)}
                                slotProps={{
                                    paper: { sx: { minWidth: 200, py: 1 } },
                                }}
                            >
                                <MenuItem
                                    disableRipple
                                    onClick={onToggleOriginal}
                                    sx={{
                                        py: 0.5,
                                        px: 2,
                                        gap: 1,
                                        "&:hover": {
                                            backgroundColor: "transparent",
                                        },
                                    }}
                                >
                                    <Switch
                                        size="small"
                                        checked={showOriginal}
                                        tabIndex={-1}
                                    />
                                    <ListItemText primary="Original orthography" />
                                </MenuItem>
                                {availableSystems.length > 0 && [
                                    <Divider key="margin-divider" />,
                                    <Typography
                                        key="margin-label"
                                        variant="overline"
                                        sx={{
                                            px: 2,
                                            color: "text.secondary",
                                        }}
                                    >
                                        Margin references
                                    </Typography>,
                                    ...availableSystems.map((slug) => (
                                        <MenuItem
                                            key={slug}
                                            disableRipple
                                            sx={{
                                                py: 0,
                                                pl: 1.5,
                                                pr: 2,
                                                "&:hover": {
                                                    backgroundColor:
                                                        "transparent",
                                                },
                                            }}
                                        >
                                            <FormControlLabel
                                                control={
                                                    <Checkbox
                                                        size="small"
                                                        checked={marginSettings.enabledSystems.has(
                                                            slug,
                                                        )}
                                                        onChange={() =>
                                                            handleToggleSystem(
                                                                slug,
                                                            )
                                                        }
                                                    />
                                                }
                                                label={
                                                    <ListItemText
                                                        primary={slug}
                                                    />
                                                }
                                                sx={{ flex: 1, mr: 0 }}
                                            />
                                            <ToggleButtonGroup
                                                value={
                                                    marginSettings.systemSides[
                                                        slug
                                                    ]
                                                }
                                                exclusive
                                                size="small"
                                                onChange={() =>
                                                    handleToggleSide(slug)
                                                }
                                            >
                                                <ToggleButton value="left">
                                                    L
                                                </ToggleButton>
                                                <ToggleButton value="right">
                                                    R
                                                </ToggleButton>
                                            </ToggleButtonGroup>
                                        </MenuItem>
                                    )),
                                ]}
                            </Menu>
                        </div>
                    </div>
                </div>

                {/* Content */}
                <PanelScrollView
                    ref={scrollViewRef}
                    bookSlug={bookSlug}
                    initialNodeSlug={nodeSlug}
                    initialSortOrder={initialSortOrder}
                    selectedSentenceId={selectedSentenceId}
                    showOriginal={showOriginal}
                    onSelectSentence={handleSelectSentence}
                    onVisibleNodeChange={handleVisibleNodeChange}
                    onSystemsDiscovered={handleSystemsDiscovered}
                    marginSettings={marginSettings}
                />
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
                    selectedSentence={
                        showSentenceDetail ? selectedSentence : undefined
                    }
                    onClose={onToggleResources}
                />
            )}
        </div>
    );
}
