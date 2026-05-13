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
    Radio,
    Select,
    Switch,
    ToggleButton,
    ToggleButtonGroup,
    Typography,
} from "@mui/material";
import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useGetBook } from "../../../api/books/books";
import type {
    FootnoteSentenceResponse,
    SentenceResponse,
    TocNodeResponse,
} from "../../../api/model";
import { useListQuotations } from "../../../api/quotations/quotations";
import { getGetTocQueryOptions, useGetToc } from "../../../api/toc/toc";
import { useAuth } from "../../../hooks/useAuth";
import { QuotationProvider } from "../context/Quotations";
import { SelectionProvider } from "../context/selection";
import {
    type SelectionMode,
    useRangeSelection,
} from "../hooks/useRangeSelection";
import {
    footnoteSentenceKey,
    footnoteSentenceMatchesKey,
    parseRangeKey,
    sentenceKey,
    sentenceMatchesKey,
} from "../keys";
import type { Panel } from "../state";

const getSentenceNumber = (s: { sentence_number?: number | null }) =>
    s.sentence_number;

import type { MarginSettings } from "./BlockRenderer";
import type { PanelScrollViewHandle } from "./PanelScrollView";
import { PanelScrollView } from "./PanelScrollView";
import { ResourcesPanel } from "./ResourcesPanel";

function findNodeInTocBySourceRef(
    nodes: TocNodeResponse[],
    sourceRef: string,
): TocNodeResponse | undefined {
    for (const node of nodes) {
        if (node.source_ref === sourceRef) return node;
        const found = findNodeInTocBySourceRef(node.children, sourceRef);
        if (found) return found;
    }
}

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

/**
 * For Bible-shape books, the chapter label by itself ("Chapter 1") is
 * ambiguous — it could be Genesis 1 or John 1. Walk up the TOC and, if
 * the parent is a bibliographic anchor (`source_id` set, e.g. the
 * "Genesis" or "John" node), return "Genesis Chapter 1". For non-Bible
 * books the chapter label is already unambiguous; return null and the
 * caller falls back to just the node label.
 */
function findBookPrefixedLabel(
    nodes: TocNodeResponse[],
    slug: string,
    parent?: TocNodeResponse,
): string | null {
    for (const node of nodes) {
        if (node.slug === slug) {
            if (parent?.source_id) {
                return `${parent.label} ${node.label}`;
            }
            return null;
        }
        const found = findBookPrefixedLabel(node.children, slug, node);
        if (found) return found;
    }
    return null;
}

interface TextPanelProps {
    panel: Panel;
    panelIndex: number;
    onSelectSentence: (sentenceId: string) => void;
    onSelectFootnoteSentence: (id: string | undefined) => void;
    onToggleOriginal: () => void;
    onToggleResources: () => void;
    onResourceViewChange: (view: string | undefined) => void;
    onViewModeChange: (
        mode: string,
        companionSlug?: string,
        targetNodeSlug?: string,
    ) => void;
    onViewLayoutChange: (layout: string) => void;
    onClose: () => void;
    onScrollNavigate: (nodeSlug: string) => void;
    onAddComparisonPanel: (bookSlug: string, nodeSlug: string) => void;
    canAddPanel: boolean;
}

export function TextPanel({
    panel,
    onSelectSentence,
    onSelectFootnoteSentence,
    onToggleOriginal,
    onToggleResources,
    onResourceViewChange,
    onViewModeChange,
    onViewLayoutChange,
    onClose,
    onScrollNavigate,
    onAddComparisonPanel,
    canAddPanel,
}: TextPanelProps) {
    const {
        bookSlug,
        nodeSlug,
        resourcesOpen,
        resourceView,
        selectedSentenceId,
        showOriginal,
        viewMode,
        viewLayout,
        companionSlug,
        footnoteSentenceId,
    } = panel;
    const [visibleSlug, setVisibleSlug] = useState<string | undefined>(
        nodeSlug,
    );
    // Sync visibleSlug when nodeSlug prop changes (e.g. navigating to a different book)
    const [prevNodeSlug, setPrevNodeSlug] = useState(nodeSlug);
    if (nodeSlug !== prevNodeSlug) {
        setPrevNodeSlug(nodeSlug);
        setVisibleSlug(nodeSlug);
    }
    const [selectedSentence, setSelectedSentence] = useState<
        SentenceResponse | undefined
    >();

    // Resolve footnote sentence(s) from the selected main sentence's footnotes
    const selectedFootnoteSentences =
        useMemo((): FootnoteSentenceResponse[] => {
            if (!footnoteSentenceId || !selectedSentence?.footnotes) return [];
            const result: FootnoteSentenceResponse[] = [];
            for (const fn of selectedSentence.footnotes) {
                for (const s of fn.sentences) {
                    if (footnoteSentenceMatchesKey(s, footnoteSentenceId)) {
                        result.push(s);
                    }
                }
            }
            return result;
        }, [footnoteSentenceId, selectedSentence]);

    const scrollViewRef = useRef<PanelScrollViewHandle>(null);

    // Margin annotation settings
    const [marginSettings, setMarginSettings] = useState<MarginSettings>({
        enabledSystems: new Set<string>(),
        systemSides: {},
    });
    const [menuAnchor, setMenuAnchor] = useState<HTMLElement | null>(null);

    // Quotation bookmarks
    const { user } = useAuth();
    const [showBookmarks, setShowBookmarks] = useState(true);

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
    const bookDetail = bookData?.status === 200 ? bookData.data : undefined;
    const bookTitle = bookDetail?.title ?? bookSlug;

    // Determine translation relationships
    const isTranslation = !!bookDetail?.source_book_slug;
    const hasTranslations = (bookDetail?.translations?.length ?? 0) > 0;
    const hasRelationship = isTranslation || hasTranslations;
    // Bible-shape: no hosted source language and no children, but
    // sibling translations exist (peers under the same translation root).
    // Triggers the flat translation picker in the view-mode menu
    // instead of the Kant-style Source/Translation/Both menu.
    const siblingTranslations = bookDetail?.sibling_translations ?? [];
    const isBibleShape = !hasRelationship && siblingTranslations.length > 0;

    const queryClient = useQueryClient();

    // Fetch companion book title for labels
    const { data: companionBookData } = useGetBook(companionSlug ?? "", {
        query: { enabled: !!companionSlug && viewMode === "st" },
    });
    const companionBookTitle =
        companionBookData?.status === 200
            ? companionBookData.data.title
            : companionSlug;

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
    const activeNodeLabel = useMemo(() => {
        if (!activeNodeSlug || !toc) return undefined;
        // Bible-shape: prepend the book name ("John Chapter 1") so the
        // header bar isn't ambiguously "Chapter 1" between Genesis
        // and John. For other books the chapter/section label is
        // already self-identifying.
        const prefixed = findBookPrefixedLabel(toc, activeNodeSlug);
        if (prefixed) return prefixed;
        return findNodeInToc(toc, activeNodeSlug)?.label;
    }, [activeNodeSlug, toc]);
    const activeNodeId = useMemo(
        () =>
            activeNodeSlug && toc
                ? findNodeInToc(toc, activeNodeSlug)?.id
                : undefined,
        [activeNodeSlug, toc],
    );

    // Fetch quotations for margin bookmark icons (lazy, non-blocking)
    const { data: quotationsData } = useListQuotations(
        bookSlug,
        { node_id: activeNodeId ?? "" },
        { query: { enabled: !!activeNodeId && !!user } },
    );
    const quotations = quotationsData?.data?.quotations ?? [];

    const showSentenceDetail =
        selectedSentence != null &&
        sentenceMatchesKey(selectedSentence, selectedSentenceId);
    const availableSystems = Object.keys(marginSettings.systemSides);

    const onMainSelect = useCallback(
        (key: string, sentence: SentenceResponse, mode: SelectionMode) => {
            setSelectedSentence(sentence);
            // Anchor branch (regular click) guards against re-emit on initial
            // scroll-to-sentence — re-selecting the same key would otherwise
            // round-trip to ReaderLayout and deselect. Range branch is
            // explicit user intent and always emits.
            if (mode === "range" || key !== selectedSentenceId) {
                onSelectSentence(key);
            }
        },
        [onSelectSentence, selectedSentenceId],
    );
    const { select: handleSelectSentence } =
        useRangeSelection<SentenceResponse>({
            keyOf: sentenceKey,
            sentenceNumberOf: getSentenceNumber,
            onSelect: onMainSelect,
        });

    // Collect sentences for range display in ResourcesPanel
    const [selectedSentences, setSelectedSentences] = useState<
        SentenceResponse[]
    >([]);
    const rangeKey = selectedSentenceId
        ? parseRangeKey(selectedSentenceId)
        : null;

    useEffect(() => {
        if (rangeKey && scrollViewRef.current) {
            setSelectedSentences(
                scrollViewRef.current.getSentencesInRange(
                    rangeKey[0],
                    rangeKey[1],
                ),
            );
        } else {
            setSelectedSentences([]);
        }
    }, [rangeKey?.[0], rangeKey?.[1]]);

    const onFnSelect = useCallback(
        (key: string) => onSelectFootnoteSentence(key),
        [onSelectFootnoteSentence],
    );
    const { select: handleSelectFootnoteSentence, clear: clearFootnoteAnchor } =
        useRangeSelection<FootnoteSentenceResponse>({
            keyOf: footnoteSentenceKey,
            sentenceNumberOf: getSentenceNumber,
            onSelect: onFnSelect,
        });

    const handleClearFootnoteSentence = useCallback(() => {
        clearFootnoteAnchor();
        onSelectFootnoteSentence(undefined);
    }, [clearFootnoteAnchor, onSelectFootnoteSentence]);

    const selectionCtx = useMemo(
        () => ({
            main: {
                key: selectedSentenceId ?? null,
                clickedId: rangeKey ? undefined : selectedSentence?.id,
            },
            footnote: {
                key: footnoteSentenceId,
                select: handleSelectFootnoteSentence,
                clear: handleClearFootnoteSentence,
            },
        }),
        [
            selectedSentenceId,
            selectedSentence,
            rangeKey,
            footnoteSentenceId,
            handleSelectFootnoteSentence,
            handleClearFootnoteSentence,
        ],
    );

    /** Find the companion node slug corresponding to the current active node.
     *  Uses source_ref (shared between source and translation) as the primary lookup.
     *  Reads the companion TOC from the query cache (eagerly fetching if needed). */
    const resolveCompanionNodeSlug = useCallback(
        async (companionBookSlug: string): Promise<string | undefined> => {
            if (!activeNodeSlug || !toc) return undefined;

            const activeNode = findNodeInToc(toc, activeNodeSlug);
            if (!activeNode) return undefined;

            // Fetch companion TOC from cache or network
            const companionTocData = await queryClient.ensureQueryData(
                getGetTocQueryOptions(companionBookSlug),
            );
            const companionToc = companionTocData?.data;
            if (!companionToc) return undefined;

            // source_ref is shared between source and translation nodes
            const companion = findNodeInTocBySourceRef(
                companionToc,
                activeNode.source_ref,
            );
            return companion?.slug;
        },
        [activeNodeSlug, toc, queryClient],
    );

    const handleTocNavigate = useCallback(
        (slug: string) => {
            // Drive navigation through the URL/visibleSlug path (same as
            // the IntersectionObserver-driven scroll sync). This makes
            // PanelScrollView's initialNodeSlug-watcher pick the right
            // strategy: in-window scrollToIndex for short jumps, or a
            // query restart + scroll for far ones (e.g. Bible cross-book
            // navigation, where sort_order can differ by 1000+). The
            // earlier imperative `scrollToNode` shortcut bypassed the
            // URL update and lost the target on far jumps.
            setVisibleSlug(slug);
            onScrollNavigate(slug);
        },
        [onScrollNavigate],
    );

    // Determine what to show in the resource panel: footnote sentences > range > single sentence
    const resourcePanelSentence = useMemo(():
        | SentenceResponse
        | FootnoteSentenceResponse
        | (SentenceResponse | FootnoteSentenceResponse)[]
        | undefined => {
        if (selectedFootnoteSentences.length > 1)
            return selectedFootnoteSentences;
        if (selectedFootnoteSentences.length === 1)
            return selectedFootnoteSentences[0];
        if (selectedSentences.length > 0) return selectedSentences;
        if (showSentenceDetail) return selectedSentence;
        return undefined;
    }, [
        selectedFootnoteSentences,
        selectedSentences,
        showSentenceDetail,
        selectedSentence,
    ]);

    return (
        <SelectionProvider value={selectionCtx}>
            <div className="flex flex-1 min-w-0 border-r border-stone-200 last:border-r-0">
                {/* Main content area */}
                <div className="flex-1 flex flex-col min-w-0">
                    {/* Toolbar */}
                    <div className="border-b border-stone-200 bg-stone-50 shrink-0 py-2 relative z-10">
                        <div className="flex items-center max-w-2xl mx-auto px-2">
                            <div className="flex items-center shrink-0">
                                <IconButton
                                    size="small"
                                    onClick={onClose}
                                    title="Close panel"
                                >
                                    <CloseOutlined fontSize="small" />
                                </IconButton>
                            </div>

                            <div className="flex-1 min-w-0 text-center">
                                <div className="text-sm text-stone-800 truncate">
                                    {activeNodeLabel ?? bookTitle}
                                </div>
                                <div className="text-xs text-stone-400 truncate">
                                    {bookTitle}
                                </div>
                            </div>

                            <div className="flex items-center gap-1 shrink-0">
                                <IconButton
                                    size="small"
                                    onClick={(e) =>
                                        setMenuAnchor(e.currentTarget)
                                    }
                                    title="Text display options"
                                >
                                    <TextFormatOutlined fontSize="small" />
                                </IconButton>
                                <Menu
                                    anchorEl={menuAnchor}
                                    open={Boolean(menuAnchor)}
                                    onClose={() => setMenuAnchor(null)}
                                    slotProps={{
                                        paper: {
                                            sx: { minWidth: 240, py: 1 },
                                        },
                                    }}
                                >
                                    {hasRelationship && [
                                        <Typography
                                            key="vm-label"
                                            variant="overline"
                                            sx={{
                                                px: 2,
                                                color: "text.secondary",
                                            }}
                                        >
                                            View mode
                                        </Typography>,
                                        <MenuItem
                                            key="vm-source"
                                            disabled={
                                                !isTranslation &&
                                                !hasTranslations
                                            }
                                            onClick={async () => {
                                                if (
                                                    isTranslation &&
                                                    bookDetail?.source_book_slug
                                                ) {
                                                    const targetSlug =
                                                        await resolveCompanionNodeSlug(
                                                            bookDetail.source_book_slug,
                                                        );
                                                    onViewModeChange(
                                                        "s",
                                                        bookDetail.source_book_slug,
                                                        targetSlug,
                                                    );
                                                } else if (viewMode === "st") {
                                                    onViewModeChange("s");
                                                }
                                                setMenuAnchor(null);
                                            }}
                                            sx={{ py: 0.5, px: 2 }}
                                        >
                                            <Radio
                                                size="small"
                                                checked={
                                                    !viewMode ||
                                                    viewMode === "s"
                                                        ? !isTranslation
                                                        : false
                                                }
                                                tabIndex={-1}
                                                sx={{ p: 0.5, mr: 1 }}
                                            />
                                            <ListItemText primary="Source" />
                                        </MenuItem>,
                                        <MenuItem
                                            key="vm-translation"
                                            disabled={
                                                !isTranslation &&
                                                !hasTranslations
                                            }
                                            onClick={async () => {
                                                if (
                                                    !isTranslation &&
                                                    hasTranslations
                                                ) {
                                                    const translationSlug =
                                                        bookDetail!
                                                            .translations[0]
                                                            .slug;
                                                    const targetSlug =
                                                        await resolveCompanionNodeSlug(
                                                            translationSlug,
                                                        );
                                                    onViewModeChange(
                                                        "t",
                                                        translationSlug,
                                                        targetSlug,
                                                    );
                                                } else if (
                                                    isTranslation &&
                                                    viewMode === "st"
                                                ) {
                                                    onViewModeChange("t");
                                                }
                                                setMenuAnchor(null);
                                            }}
                                            sx={{ py: 0.5, px: 2 }}
                                        >
                                            <Radio
                                                size="small"
                                                checked={
                                                    !viewMode ||
                                                    viewMode === "t"
                                                        ? isTranslation
                                                        : false
                                                }
                                                tabIndex={-1}
                                                sx={{ p: 0.5, mr: 1 }}
                                            />
                                            <ListItemText primary="Translation" />
                                        </MenuItem>,
                                        <MenuItem
                                            key="vm-both"
                                            disabled={!hasRelationship}
                                            onClick={() => {
                                                const companion = isTranslation
                                                    ? bookDetail!
                                                          .source_book_slug!
                                                    : bookDetail!
                                                          .translations[0]
                                                          ?.slug;
                                                if (companion) {
                                                    onViewModeChange(
                                                        "st",
                                                        companion,
                                                    );
                                                }
                                                setMenuAnchor(null);
                                            }}
                                            sx={{ py: 0.5, px: 2 }}
                                        >
                                            <Radio
                                                size="small"
                                                checked={viewMode === "st"}
                                                tabIndex={-1}
                                                sx={{ p: 0.5, mr: 1 }}
                                            />
                                            <ListItemText primary="Source with Translation" />
                                        </MenuItem>,
                                        ...(viewMode === "st" &&
                                        !isTranslation &&
                                        bookDetail!.translations.length > 1
                                            ? [
                                                  <MenuItem
                                                      key="vm-picker"
                                                      disableRipple
                                                      sx={{
                                                          py: 0.5,
                                                          px: 2,
                                                          "&:hover": {
                                                              backgroundColor:
                                                                  "transparent",
                                                          },
                                                      }}
                                                  >
                                                      <Select
                                                          size="small"
                                                          value={
                                                              companionSlug ??
                                                              ""
                                                          }
                                                          onChange={(e) => {
                                                              onViewModeChange(
                                                                  "st",
                                                                  e.target
                                                                      .value,
                                                              );
                                                              setMenuAnchor(
                                                                  null,
                                                              );
                                                          }}
                                                          fullWidth
                                                          sx={{
                                                              fontSize:
                                                                  "0.875rem",
                                                          }}
                                                      >
                                                          {bookDetail!.translations.map(
                                                              (t) => (
                                                                  <MenuItem
                                                                      key={
                                                                          t.slug
                                                                      }
                                                                      value={
                                                                          t.slug
                                                                      }
                                                                  >
                                                                      {t.title}
                                                                  </MenuItem>
                                                              ),
                                                          )}
                                                      </Select>
                                                  </MenuItem>,
                                              ]
                                            : []),
                                        ...(viewMode === "st"
                                            ? [
                                                  <Divider key="vl-divider" />,
                                                  <Typography
                                                      key="vl-label"
                                                      variant="overline"
                                                      sx={{
                                                          px: 2,
                                                          color: "text.secondary",
                                                      }}
                                                  >
                                                      Layout
                                                  </Typography>,
                                                  ...(
                                                      [
                                                          [
                                                              "sp",
                                                              "Stacked paragraphs",
                                                          ],
                                                          [
                                                              "ss",
                                                              "Stacked sentences",
                                                          ],
                                                          [
                                                              "bpl",
                                                              "Side-by-side (source left)",
                                                          ],
                                                          [
                                                              "bpr",
                                                              "Side-by-side (source right)",
                                                          ],
                                                          [
                                                              "bsl",
                                                              "Side-by-side sentences (source left)",
                                                          ],
                                                          [
                                                              "bsr",
                                                              "Side-by-side sentences (source right)",
                                                          ],
                                                      ] as const
                                                  ).map(([code, label]) => (
                                                      <MenuItem
                                                          key={`vl-${code}`}
                                                          onClick={() => {
                                                              onViewLayoutChange(
                                                                  code,
                                                              );
                                                              setMenuAnchor(
                                                                  null,
                                                              );
                                                          }}
                                                          sx={{
                                                              py: 0.5,
                                                              px: 2,
                                                          }}
                                                      >
                                                          <Radio
                                                              size="small"
                                                              checked={
                                                                  (viewLayout ??
                                                                      "sp") ===
                                                                  code
                                                              }
                                                              tabIndex={-1}
                                                              sx={{
                                                                  p: 0.5,
                                                                  mr: 1,
                                                              }}
                                                          />
                                                          <ListItemText
                                                              primary={label}
                                                          />
                                                      </MenuItem>
                                                  )),
                                              ]
                                            : []),
                                        <Divider key="vm-bottom-divider" />,
                                    ]}
                                    {isBibleShape && [
                                        <Typography
                                            key="bs-label"
                                            variant="overline"
                                            sx={{
                                                px: 2,
                                                color: "text.secondary",
                                            }}
                                        >
                                            Translation
                                        </Typography>,
                                        // Active translation (radio-checked)
                                        <MenuItem
                                            key={`bs-self-${bookSlug}`}
                                            onClick={() => setMenuAnchor(null)}
                                            sx={{ py: 0.5, px: 2 }}
                                        >
                                            <Radio
                                                size="small"
                                                checked={
                                                    !viewMode ||
                                                    viewMode !== "st"
                                                }
                                                tabIndex={-1}
                                                sx={{ p: 0.5, mr: 1 }}
                                            />
                                            <ListItemText
                                                primary={
                                                    bookDetail?.publisher ??
                                                    bookTitle
                                                }
                                            />
                                        </MenuItem>,
                                        // Sibling translations: navigate the
                                        // panel to the equivalent node in that
                                        // translation. Q9 selection-carry
                                        // happens via the URL preserving
                                        // selectedSentenceId; the equivalent
                                        // sentence resolves on the other side
                                        // (P6 wires that explicitly).
                                        ...siblingTranslations.map((s) => (
                                            <MenuItem
                                                key={`bs-sibling-${s.slug}`}
                                                onClick={async () => {
                                                    const targetSlug =
                                                        await resolveCompanionNodeSlug(
                                                            s.slug,
                                                        );
                                                    onViewModeChange(
                                                        "t",
                                                        s.slug,
                                                        targetSlug,
                                                    );
                                                    setMenuAnchor(null);
                                                }}
                                                sx={{ py: 0.5, px: 2 }}
                                            >
                                                <Radio
                                                    size="small"
                                                    checked={false}
                                                    tabIndex={-1}
                                                    sx={{ p: 0.5, mr: 1 }}
                                                />
                                                <ListItemText
                                                    primary={s.title}
                                                />
                                            </MenuItem>
                                        )),
                                        <Divider key="bs-divider-sxs" />,
                                        <MenuItem
                                            key="bs-side-by-side"
                                            disabled={
                                                siblingTranslations.length === 0
                                            }
                                            onClick={() => {
                                                const companion =
                                                    siblingTranslations[0]
                                                        ?.slug;
                                                if (companion) {
                                                    onViewModeChange(
                                                        "st",
                                                        companion,
                                                    );
                                                }
                                                setMenuAnchor(null);
                                            }}
                                            sx={{ py: 0.5, px: 2 }}
                                        >
                                            <Radio
                                                size="small"
                                                checked={viewMode === "st"}
                                                tabIndex={-1}
                                                sx={{ p: 0.5, mr: 1 }}
                                            />
                                            <ListItemText primary="Side-by-side" />
                                        </MenuItem>,
                                        // Companion picker visible only when
                                        // there are 2+ siblings (3-way Bible
                                        // setup) and side-by-side is on.
                                        ...(viewMode === "st" &&
                                        siblingTranslations.length > 1
                                            ? [
                                                  <MenuItem
                                                      key="bs-picker"
                                                      disableRipple
                                                      sx={{
                                                          py: 0.5,
                                                          px: 2,
                                                          "&:hover": {
                                                              backgroundColor:
                                                                  "transparent",
                                                          },
                                                      }}
                                                  >
                                                      <Select
                                                          size="small"
                                                          value={
                                                              companionSlug ??
                                                              ""
                                                          }
                                                          onChange={(e) => {
                                                              onViewModeChange(
                                                                  "st",
                                                                  e.target
                                                                      .value,
                                                              );
                                                              setMenuAnchor(
                                                                  null,
                                                              );
                                                          }}
                                                          fullWidth
                                                          sx={{
                                                              fontSize:
                                                                  "0.875rem",
                                                          }}
                                                      >
                                                          {siblingTranslations.map(
                                                              (s) => (
                                                                  <MenuItem
                                                                      key={
                                                                          s.slug
                                                                      }
                                                                      value={
                                                                          s.slug
                                                                      }
                                                                  >
                                                                      {s.title}
                                                                  </MenuItem>
                                                              ),
                                                          )}
                                                      </Select>
                                                  </MenuItem>,
                                              ]
                                            : []),
                                        <Divider key="bs-bottom-divider" />,
                                    ]}
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
                                                        marginSettings
                                                            .systemSides[slug]
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
                                    {user && [
                                        <Divider key="bookmarks-divider" />,
                                        <Typography
                                            key="bookmarks-label"
                                            variant="overline"
                                            sx={{
                                                px: 2,
                                                color: "text.secondary",
                                            }}
                                        >
                                            Saved quotations
                                        </Typography>,
                                        <MenuItem
                                            key="bookmarks-toggle"
                                            disableRipple
                                            sx={{
                                                py: 0,
                                                pl: 1.5,
                                                "&:hover": {
                                                    backgroundColor:
                                                        "transparent",
                                                },
                                            }}
                                        >
                                            <FormControlLabel
                                                control={
                                                    <Switch
                                                        size="small"
                                                        checked={showBookmarks}
                                                        onChange={() =>
                                                            setShowBookmarks(
                                                                !showBookmarks,
                                                            )
                                                        }
                                                    />
                                                }
                                                label={
                                                    <ListItemText primary="Show notes icons" />
                                                }
                                            />
                                        </MenuItem>,
                                    ]}
                                </Menu>
                            </div>
                        </div>
                    </div>

                    {/* Content */}
                    <QuotationProvider
                        quotations={quotations}
                        showBookmarks={showBookmarks}
                    >
                        <PanelScrollView
                            ref={scrollViewRef}
                            bookSlug={bookSlug}
                            initialNodeSlug={nodeSlug}
                            selectedSentenceId={selectedSentenceId}
                            showOriginal={showOriginal}
                            viewMode={viewMode}
                            viewLayout={viewLayout}
                            companionSlug={companionSlug}
                            primaryLabel={bookTitle}
                            companionLabel={companionBookTitle}
                            onSelectSentence={handleSelectSentence}
                            onVisibleNodeChange={handleVisibleNodeChange}
                            onSystemsDiscovered={handleSystemsDiscovered}
                            marginSettings={marginSettings}
                        />
                    </QuotationProvider>
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
                        selectedSentence={resourcePanelSentence}
                        selectedSentenceId={selectedSentenceId}
                        onClose={onToggleResources}
                        activeView={resourceView}
                        onViewChange={onResourceViewChange}
                    />
                )}
            </div>
        </SelectionProvider>
    );
}
