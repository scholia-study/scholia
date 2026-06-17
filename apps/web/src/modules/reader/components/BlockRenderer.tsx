import EditNoteOutlined from "@mui/icons-material/EditNoteOutlined";
import parse, { Element } from "html-react-parser";
import {
    Fragment,
    useCallback,
    useEffect,
    useMemo,
    useRef,
    useState,
} from "react";
import type {
    ContentBlockResponse,
    PageMarkerResponse,
    SentenceResponse,
} from "../../../api/model";
import { useQuotationContext } from "../context/Quotations";
import {
    useFootnoteActions,
    useFootnoteAnchor,
    useSentenceSelection,
} from "../context/selection";
import { sentenceKey, sentenceMatchesKey } from "../keys";
import { FootnotePopover } from "./FootnotePopover";

export interface MarginSettings {
    enabledSystems: Set<string>;
    systemSides: Record<string, "left" | "right">;
}

interface SentenceGroup {
    /** Indented-run index, or null for the normal paragraph flow. */
    segment: number | null;
    /** Sentences in this run, with their original index into block.sentences
     *  (preserved so per-sentence margin-marker lookups stay aligned). */
    items: { sentence: SentenceResponse; index: number }[];
}

/**
 * Split a block's sentences into consecutive runs sharing a `segment`. Normal
 * flow (segment null) groups together; each `+ `-authored indented run carries
 * a distinct segment value and so becomes its own group. The reader wraps each
 * indented group in one hanging-indent block — see the `paragraph` case.
 */
function groupSentencesBySegment(
    sentences: SentenceResponse[],
): SentenceGroup[] {
    const groups: SentenceGroup[] = [];
    for (let index = 0; index < sentences.length; index++) {
        const sentence = sentences[index];
        const segment = sentence.segment ?? null;
        const last = groups.at(-1);
        if (last && last.segment === segment) {
            last.items.push({ sentence, index });
        } else {
            groups.push({ segment, items: [{ sentence, index }] });
        }
    }
    return groups;
}

function MarginNotes({
    markers,
    side,
    vAlign = "top",
}: {
    markers: PageMarkerResponse[];
    side: "left" | "right";
    /**
     * "top" anchors the note at the start of its container (paragraphs, where
     * the container spans many lines). "center" vertically centres it within
     * the container — used for verse, where each line is its own one-line box,
     * so the number sits level with the line instead of riding its top edge.
     */
    vAlign?: "top" | "center";
}) {
    return (
        <span
            className={`absolute flex gap-1 whitespace-nowrap text-[10px] text-stone-400 select-none ${
                side === "left"
                    ? "right-full mr-2 justify-end"
                    : "left-full ml-2 justify-start"
            } ${vAlign === "center" ? "inset-y-0 items-center" : ""}`}
            style={{ lineHeight: "inherit" }}
        >
            {[...markers]
                .sort((a, b) => a.system_slug.localeCompare(b.system_slug))
                .map((pm, i) => (
                    <span
                        key={`${pm.system_slug}-${pm.ref_value}-${i}`}
                        title={`${pm.system_slug}: ${pm.ref_value}`}
                    >
                        {pm.ref_value}
                    </span>
                ))}
        </span>
    );
}

function FootnoteSup({
    footnoteNumber,
    sentence,
    showOriginal,
    onSelectSentence,
}: {
    footnoteNumber: number;
    sentence: SentenceResponse;
    showOriginal?: boolean;
    onSelectSentence: (sentence: SentenceResponse, shiftKey: boolean) => void;
}) {
    const { clear: clearFootnoteSelection } = useFootnoteActions();
    const [anchorEl, setAnchorEl] = useState<HTMLElement | null>(null);
    const supRef = useRef<HTMLElement>(null);
    // Suppresses the auto-open effect during the render cycle where local
    // anchorEl has cleared but the route-state footnote selection hasn't
    // propagated yet. Resets once shouldAutoOpen settles to false.
    const suppressAutoOpenRef = useRef(false);

    const footnote = sentence.footnotes?.find(
        (fn) => fn.number === footnoteNumber,
    );

    // Auto-open popover if a footnote sentence from this footnote is selected
    const shouldAutoOpen = useFootnoteAnchor(footnote ? [footnote] : undefined);

    useEffect(() => {
        if (!shouldAutoOpen) {
            suppressAutoOpenRef.current = false;
            return;
        }
        if (suppressAutoOpenRef.current) return;
        if (!anchorEl && supRef.current) {
            setAnchorEl(supRef.current);
        }
    }, [shouldAutoOpen, anchorEl]);

    const handleClick = useCallback(
        (e: React.MouseEvent<HTMLElement>) => {
            e.stopPropagation();
            if (anchorEl) {
                suppressAutoOpenRef.current = true;
                setAnchorEl(null);
                clearFootnoteSelection();
            } else {
                suppressAutoOpenRef.current = false;
                setAnchorEl(e.currentTarget);
                // Ensure the main sentence is selected too
                onSelectSentence(sentence, false);
            }
        },
        [anchorEl, clearFootnoteSelection, onSelectSentence, sentence],
    );

    const handleClose = useCallback(() => {
        suppressAutoOpenRef.current = true;
        setAnchorEl(null);
        clearFootnoteSelection();
    }, [clearFootnoteSelection]);

    // Dismiss if anchor unmounts (virtualization)
    useEffect(() => {
        if (!anchorEl) return;
        const check = () => {
            if (!anchorEl.isConnected) {
                suppressAutoOpenRef.current = true;
                setAnchorEl(null);
                clearFootnoteSelection();
            }
        };
        const id = setInterval(check, 500);
        return () => clearInterval(id);
    }, [anchorEl, clearFootnoteSelection]);

    return (
        <>
            <sup
                ref={supRef}
                onClick={handleClick}
                className="cursor-pointer hover:bg-stone-200 rounded-sm transition-colors"
            >
                {footnoteNumber}
            </sup>
            {footnote && (
                <FootnotePopover
                    footnote={footnote}
                    anchorEl={anchorEl}
                    open={Boolean(anchorEl)}
                    onClose={handleClose}
                    showOriginal={showOriginal}
                />
            )}
        </>
    );
}

export function Sentence({
    sentence,
    isSelected,
    showOriginal,
    onSelect,
    marginSettings,
    nodeSourceRef,
    displayPageMarkers,
    markerVAlign = "top",
}: {
    sentence: SentenceResponse;
    isSelected: boolean;
    showOriginal?: boolean;
    onSelect: (sentence: SentenceResponse, shiftKey: boolean) => void;
    marginSettings?: MarginSettings;
    /**
     * `source_ref` of the parent toc_node — required for verse-key
     * marker projection. The QuotationContext combines it with each
     * page marker's `ref_value` to build the cross-translation lookup
     * key. Optional because not all sentence renderings (e.g. inline
     * footnote previews) sit inside a chapter context.
     */
    nodeSourceRef?: string;
    /**
     * Margin markers to render for this sentence. When omitted, falls
     * back to `sentence.page_markers`. Block uses this prop to drop
     * markers whose ref_value duplicates the previous sentence's for
     * the same system — Bible verses now segment to multiple
     * sentences sharing a verse marker, and rendering each one stacks
     * verse numbers on top of one another in the margin.
     */
    displayPageMarkers?: PageMarkerResponse[];
    /** Vertical alignment of margin notes; "center" for verse lines. */
    markerVAlign?: "top" | "center";
}) {
    const { isCorrespondent } = useSentenceSelection(sentence);
    const isFootnoteAnchor = useFootnoteAnchor(sentence.footnotes);
    const { showBookmarks, isSentenceSaved } = useQuotationContext();

    let leftMarkers: PageMarkerResponse[] | undefined;
    let rightMarkers: PageMarkerResponse[] | undefined;
    let inlineMarkers: PageMarkerResponse[] | undefined;

    const markersForRender = displayPageMarkers ?? sentence.page_markers;
    // Verse-style markers (Bible) render inline as a tiny superscript
    // before the sentence text. Margin positioning would stack different
    // verses on the same line of wrapped text on top of each other (e.g.
    // 1:2 and 1:3 both end up at the same y). Page-style markers (Kant
    // pagination) keep the existing margin layout governed by
    // marginSettings. The "verse" detection is by system slug for now —
    // ref_type is on `reference_systems` in the schema but not yet
    // surfaced through PageMarkerResponse; revisit if a second inline
    // system shows up.
    for (const pm of markersForRender) {
        if (pm.system_slug === "verse") {
            if (!inlineMarkers) inlineMarkers = [];
            inlineMarkers.push(pm);
            continue;
        }
        if (!marginSettings || marginSettings.enabledSystems.size === 0)
            continue;
        if (!marginSettings.enabledSystems.has(pm.system_slug)) continue;
        const side = marginSettings.systemSides[pm.system_slug] ?? "right";
        if (side === "left") {
            if (!leftMarkers) leftMarkers = [];
            leftMarkers.push(pm);
        } else {
            if (!rightMarkers) rightMarkers = [];
            rightMarkers.push(pm);
        }
    }

    const highlightClass = isSelected
        ? isCorrespondent
            ? "bg-amber-100"
            : isFootnoteAnchor
              ? "bg-amber-100"
              : "bg-amber-200"
        : isFootnoteAnchor
          ? "bg-amber-100"
          : "hover:bg-stone-200";

    const parsedHtml = useMemo(() => {
        const html =
            showOriginal && sentence.original_html
                ? sentence.original_html
                : sentence.html;
        return parse(html, {
            replace: (domNode) => {
                if (domNode instanceof Element && domNode.name === "sup") {
                    const textContent = domNode.children
                        .map((c) => ("data" in c ? c.data : ""))
                        .join("")
                        .trim();
                    if (!textContent || !/^\d+$/.test(textContent)) return;
                    const num = Number(textContent);
                    // Only intercept if this sentence actually has a footnote with this number
                    if (sentence.footnotes?.some((fn) => fn.number === num)) {
                        return (
                            <FootnoteSup
                                footnoteNumber={num}
                                sentence={sentence}
                                showOriginal={showOriginal}
                                onSelectSentence={onSelect}
                            />
                        );
                    }
                }
            },
        });
    }, [showOriginal, sentence, onSelect]);

    const isSaved =
        showBookmarks &&
        isSentenceSaved(
            sentence.sentence_number,
            nodeSourceRef,
            sentence.page_markers,
        );

    return (
        <>
            {isSaved && (
                <span
                    className="absolute right-full mr-1 text-stone-300 select-none pointer-events-none"
                    style={{ lineHeight: "inherit" }}
                    title="Saved quotation"
                >
                    <EditNoteOutlined sx={{ fontSize: 14 }} />
                </span>
            )}
            {leftMarkers && (
                <MarginNotes
                    markers={leftMarkers}
                    side="left"
                    vAlign={markerVAlign}
                />
            )}
            {rightMarkers && (
                <MarginNotes
                    markers={rightMarkers}
                    side="right"
                    vAlign={markerVAlign}
                />
            )}
            <span
                data-sentence-key={sentenceKey(sentence)}
                onMouseDown={(e) => {
                    if (e.shiftKey) e.preventDefault();
                }}
                onClick={(e) => onSelect(sentence, e.shiftKey)}
                className={`cursor-pointer transition-colors rounded-sm ${highlightClass}`}
            >
                {inlineMarkers?.map((pm, i) => (
                    <sup
                        key={`${pm.system_slug}-${pm.ref_value}-${i}`}
                        className="text-[0.65em] text-stone-400 mr-0.5 select-none"
                        title={pm.ref_value}
                    >
                        {pm.ref_value}
                    </sup>
                ))}
                {parsedHtml}
            </span>{" "}
        </>
    );
}

function HeadingSentence({
    sentence,
    showOriginal,
    marginSettings,
}: {
    sentence: SentenceResponse;
    showOriginal?: boolean;
    marginSettings?: MarginSettings;
}) {
    let leftMarkers: PageMarkerResponse[] | undefined;
    let rightMarkers: PageMarkerResponse[] | undefined;

    if (
        marginSettings &&
        marginSettings.enabledSystems.size > 0 &&
        sentence.page_markers.length > 0
    ) {
        for (const pm of sentence.page_markers) {
            if (!marginSettings.enabledSystems.has(pm.system_slug)) continue;
            const side = marginSettings.systemSides[pm.system_slug] ?? "right";
            if (side === "left") {
                if (!leftMarkers) leftMarkers = [];
                leftMarkers.push(pm);
            } else {
                if (!rightMarkers) rightMarkers = [];
                rightMarkers.push(pm);
            }
        }
    }

    return (
        <>
            {leftMarkers && <MarginNotes markers={leftMarkers} side="left" />}
            {rightMarkers && (
                <MarginNotes markers={rightMarkers} side="right" />
            )}
            <span>
                {parse(
                    showOriginal && sentence.original_html
                        ? sentence.original_html
                        : sentence.html,
                )}
            </span>{" "}
        </>
    );
}

/**
 * Thematic break between content blocks. `---` in the source renders as a
 * plain horizontal rule; `***` renders as a centered, bold "* * *" ornament
 * (a dinkus). The variant rides along in the block's `html` as a sentinel
 * `dinkus` class — the styling lives here, not in the stored content, so no
 * decorative text leaks into search, sentence-splitting, or alignment.
 * `className` controls the outer spacing so callers can swap padding (the main
 * reader, whose virtualizer measures padding but not margins) for margin (the
 * interleaved comparison views).
 */
export function Separator({
    block,
    className = "py-8",
}: {
    block?: ContentBlockResponse;
    className?: string;
}) {
    if (block?.html?.includes("dinkus")) {
        return (
            <div
                className={`text-center font-bold text-stone-400 select-none ${className}`}
            >
                * * *
            </div>
        );
    }
    // `<hr>` has special box behavior and doesn't pad reliably on its own, so
    // the wrapper carries the spacing.
    return (
        <div className={className}>
            <hr className="border-stone-200" />
        </div>
    );
}

export function Block({
    block,
    selectedSentenceId,
    showOriginal,
    onSelectSentence,
    marginSettings,
    nodeSourceRef,
}: {
    block: ContentBlockResponse;
    selectedSentenceId: string | null;
    showOriginal?: boolean;
    onSelectSentence: (sentence: SentenceResponse, shiftKey: boolean) => void;
    marginSettings?: MarginSettings;
    /** Forwarded to `Sentence` for verse-key marker projection. */
    nodeSourceRef?: string;
}) {
    const blockHtml =
        showOriginal && block.original_html ? block.original_html : block.html;

    // Per-sentence display-marker lists: drop a page_marker whose
    // ref_value matches the previous sentence's for the same system.
    // Bible verses now segment to multiple sentences sharing a verse
    // marker; without dedup the verse number stacks at the same y in
    // the margin and blurs into itself. Computed once per render.
    const sentenceDisplayMarkers = useMemo(() => {
        const lastRefBySystem = new Map<string, string>();
        return block.sentences.map((s) => {
            const result: PageMarkerResponse[] = [];
            for (const m of s.page_markers) {
                if (lastRefBySystem.get(m.system_slug) !== m.ref_value) {
                    result.push(m);
                    lastRefBySystem.set(m.system_slug, m.ref_value);
                }
            }
            return result;
        });
    }, [block.sentences]);

    // Block elements use padding (not margin) for vertical spacing —
    // Virtuoso measures item heights via ResizeObserver's `contentRect`,
    // which excludes margins. Padding stays inside the box and is
    // always counted, so item heights stay accurate even when an inner
    // wrapper drops its margin-containment in some future refactor.
    // (See virtuoso troubleshooting §2.)
    switch (block.block_type) {
        case "heading":
            return (
                <h2 className="relative text-[1.5em] font-bold pt-8 pb-6 text-stone-900">
                    {block.sentences.length > 0
                        ? block.sentences.map((s) => (
                              <HeadingSentence
                                  key={s.id}
                                  sentence={s}
                                  showOriginal={showOriginal}
                                  marginSettings={marginSettings}
                              />
                          ))
                        : parse(blockHtml)}
                </h2>
            );
        case "paragraph": {
            // Sentences split into runs: normal flow renders inline; each
            // `+ `-authored indented run renders as one hanging-indent block
            // (e.g. Kant's `1) 2) 3)` enumerations). The paragraph stays a
            // single <p> with one paragraph_number.
            const groups = groupSentencesBySegment(block.sentences);
            const renderSentence = (s: SentenceResponse, i: number) => (
                <Sentence
                    key={s.id}
                    sentence={s}
                    isSelected={sentenceMatchesKey(s, selectedSentenceId)}
                    showOriginal={showOriginal}
                    onSelect={onSelectSentence}
                    marginSettings={marginSettings}
                    nodeSourceRef={nodeSourceRef}
                    displayPageMarkers={sentenceDisplayMarkers[i]}
                />
            );
            return (
                <p className="relative pb-4 leading-[var(--reader-line-height)] text-stone-700">
                    {groups.map((group, gi) =>
                        group.segment == null ? (
                            <Fragment key={`flow-${gi}`}>
                                {group.items.map(({ sentence, index }) =>
                                    renderSentence(sentence, index),
                                )}
                            </Fragment>
                        ) : (
                            // Hanging indent, no list marker — Kant's own
                            // `1)`/`2)` is the only visible label. A <span>
                            // (display:block), not a <div>, so it stays valid
                            // phrasing content inside <p> and survives SSR
                            // hydration.
                            <span
                                key={`seg-${gi}`}
                                className="block [padding-left:2em] [text-indent:-1em]"
                            >
                                {group.items.map(({ sentence, index }) =>
                                    renderSentence(sentence, index),
                                )}
                            </span>
                        ),
                    )}
                </p>
            );
        }
        case "verse": {
            // One <Sentence> per verse line, each on its own line (no reflow)
            // with a hanging indent so a line too long for the column wraps
            // under itself. Reuses the paragraph selection wiring, so lines
            // are individually clickable and shift-click range-selects.
            return (
                <div className="relative pb-4 leading-[var(--reader-line-height)] text-stone-700">
                    {block.sentences.map((s, i) => (
                        <span
                            key={s.id}
                            className="block relative [padding-left:1.5em] [text-indent:-1.5em]"
                        >
                            <Sentence
                                sentence={s}
                                isSelected={sentenceMatchesKey(
                                    s,
                                    selectedSentenceId,
                                )}
                                showOriginal={showOriginal}
                                onSelect={onSelectSentence}
                                marginSettings={marginSettings}
                                nodeSourceRef={nodeSourceRef}
                                displayPageMarkers={sentenceDisplayMarkers[i]}
                                markerVAlign="center"
                            />
                        </span>
                    ))}
                </div>
            );
        }
        case "footnote":
            return (
                <div className="relative pb-4 ml-8 text-sm text-stone-500 italic border-l-2 border-stone-200 pl-4">
                    {block.sentences.map((s, i) => (
                        <Sentence
                            key={s.id}
                            sentence={s}
                            isSelected={sentenceMatchesKey(
                                s,
                                selectedSentenceId,
                            )}
                            showOriginal={showOriginal}
                            onSelect={onSelectSentence}
                            marginSettings={marginSettings}
                            nodeSourceRef={nodeSourceRef}
                            displayPageMarkers={sentenceDisplayMarkers[i]}
                        />
                    ))}
                </div>
            );
        case "figure": {
            // Diagram-like insertion (e.g. the table of judgments): the
            // block's html is verbatim editor-authored `<figure>` markup,
            // rendered as-is. The whole figure is one selectable/quotable
            // unit, anchored to its single sentence (the figcaption label).
            const anchor = block.sentences[0];
            const figureHtml =
                showOriginal && block.original_html
                    ? block.original_html
                    : block.html;

            let leftMarkers: PageMarkerResponse[] | undefined;
            let rightMarkers: PageMarkerResponse[] | undefined;
            if (
                anchor &&
                marginSettings &&
                marginSettings.enabledSystems.size > 0
            ) {
                for (const pm of anchor.page_markers) {
                    if (!marginSettings.enabledSystems.has(pm.system_slug))
                        continue;
                    const side =
                        marginSettings.systemSides[pm.system_slug] ?? "right";
                    if (side === "left") {
                        if (!leftMarkers) leftMarkers = [];
                        leftMarkers.push(pm);
                    } else {
                        if (!rightMarkers) rightMarkers = [];
                        rightMarkers.push(pm);
                    }
                }
            }

            const isSelected = anchor
                ? sentenceMatchesKey(anchor, selectedSentenceId)
                : false;

            return (
                <div className="relative py-4">
                    {leftMarkers && (
                        <MarginNotes markers={leftMarkers} side="left" />
                    )}
                    {rightMarkers && (
                        <MarginNotes markers={rightMarkers} side="right" />
                    )}
                    <div
                        data-sentence-key={
                            anchor ? sentenceKey(anchor) : undefined
                        }
                        onMouseDown={(e) => {
                            if (e.shiftKey) e.preventDefault();
                        }}
                        onClick={(e) =>
                            anchor && onSelectSentence(anchor, e.shiftKey)
                        }
                        // The figcaption is editorial apparatus (the figure's
                        // label), not part of the diagram — pin it to the
                        // bottom-right of the figure box and mute it so it
                        // doesn't read as a title. The figure is the
                        // positioning context.
                        className={`cursor-pointer rounded-sm transition-colors [&_figure]:relative [&_figcaption]:absolute [&_figcaption]:right-6 [&_figcaption]:bottom-4 [&_figcaption]:text-right [&_figcaption]:text-sm [&_figcaption]:text-stone-400 ${
                            isSelected
                                ? "ring-2 ring-amber-300 bg-amber-50"
                                : "hover:bg-stone-100"
                        }`}
                    >
                        {parse(figureHtml)}
                    </div>
                </div>
            );
        }
        case "separator":
            return <Separator block={block} />;
        default:
            return <div className="pb-4">{parse(blockHtml)}</div>;
    }
}
