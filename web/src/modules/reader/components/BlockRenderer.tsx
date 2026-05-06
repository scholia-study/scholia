import EditNoteOutlined from "@mui/icons-material/EditNoteOutlined";
import parse, { Element } from "html-react-parser";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
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

function MarginNotes({
    markers,
    side,
}: {
    markers: PageMarkerResponse[];
    side: "left" | "right";
}) {
    return (
        <span
            className={`absolute flex gap-1 whitespace-nowrap text-[10px] text-stone-400 select-none ${
                side === "left"
                    ? "right-full mr-2 justify-end"
                    : "left-full ml-2 justify-start"
            }`}
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
            {leftMarkers && <MarginNotes markers={leftMarkers} side="left" />}
            {rightMarkers && (
                <MarginNotes markers={rightMarkers} side="right" />
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
                <h2 className="relative text-2xl font-bold pt-8 pb-6 text-stone-900">
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
        case "paragraph":
            return (
                <p className="relative pb-4 leading-relaxed text-stone-700">
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
                </p>
            );
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
        case "separator":
            // Wrap the <hr> in a div with padding so the spacing is
            // padding-based — `<hr>` has special box behavior and
            // doesn't pad reliably on its own.
            return (
                <div className="py-8">
                    <hr className="border-stone-200" />
                </div>
            );
        default:
            return <div className="pb-4">{parse(blockHtml)}</div>;
    }
}
