import parse, { Element } from "html-react-parser";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type {
    ContentBlockResponse,
    FootnoteSentenceResponse,
    PageMarkerResponse,
    SentenceResponse,
} from "../api/model";
import { FootnotePopover } from "./FootnotePopover";
import { useFootnoteSelection } from "./FootnoteSelectionContext";
import { useSentenceSelection } from "./SentenceSelectionContext";

/** URL-friendly key for a sentence: sentence_number if available, otherwise ID. */
export function sentenceKey(s: SentenceResponse): string {
    return s.sentence_number != null ? String(s.sentence_number) : s.id;
}

/** Check if a sentence matches a URL key (sentence_number or ID). */
export function sentenceMatchesKey(
    s: SentenceResponse,
    key: string | undefined | null,
): boolean {
    if (!key) return false;
    return s.id === key || (s.sentence_number != null && String(s.sentence_number) === key);
}

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
            {[...markers].sort((a, b) => a.system_slug.localeCompare(b.system_slug)).map((pm, i) => (
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
    onSelectSentence: (sentence: SentenceResponse) => void;
}) {
    const { selectedFootnoteSentenceId, onSelectFootnoteSentence, onClearFootnoteSentence } =
        useFootnoteSelection();
    const [anchorEl, setAnchorEl] = useState<HTMLElement | null>(null);
    const supRef = useRef<HTMLElement>(null);

    const footnote = sentence.footnotes?.find((fn) => fn.number === footnoteNumber);

    // Auto-open popover if a footnote sentence from this footnote is selected
    const shouldAutoOpen = useMemo(() => {
        if (!selectedFootnoteSentenceId || !footnote) return false;
        return footnote.sentences.some((s) => s.id === selectedFootnoteSentenceId);
    }, [selectedFootnoteSentenceId, footnote]);

    useEffect(() => {
        if (shouldAutoOpen && !anchorEl && supRef.current) {
            setAnchorEl(supRef.current);
        }
    }, [shouldAutoOpen, anchorEl]);

    const handleClick = useCallback(
        (e: React.MouseEvent<HTMLElement>) => {
            e.stopPropagation();
            if (anchorEl) {
                setAnchorEl(null);
                onClearFootnoteSentence();
            } else {
                setAnchorEl(e.currentTarget);
                // Ensure the main sentence is selected too
                onSelectSentence(sentence);
            }
        },
        [anchorEl, onClearFootnoteSentence, onSelectSentence, sentence],
    );

    const handleClose = useCallback(() => {
        setAnchorEl(null);
        onClearFootnoteSentence();
    }, [onClearFootnoteSentence]);

    // Dismiss if anchor unmounts (virtualization)
    useEffect(() => {
        if (!anchorEl) return;
        const check = () => {
            if (!anchorEl.isConnected) {
                setAnchorEl(null);
                onClearFootnoteSentence();
            }
        };
        const id = setInterval(check, 500);
        return () => clearInterval(id);
    }, [anchorEl, onClearFootnoteSentence]);

    const handleSelectFootnoteSentence = useCallback(
        (fsSentence: FootnoteSentenceResponse) => {
            onSelectFootnoteSentence(fsSentence);
        },
        [onSelectFootnoteSentence],
    );

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
                    selectedFootnoteSentenceId={selectedFootnoteSentenceId}
                    onSelectFootnoteSentence={handleSelectFootnoteSentence}
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
}: {
    sentence: SentenceResponse;
    isSelected: boolean;
    showOriginal?: boolean;
    onSelect: (sentence: SentenceResponse) => void;
    marginSettings?: MarginSettings;
}) {
    const { isCorrespondent } = useSentenceSelection(sentence);
    const { selectedFootnoteSentenceId } = useFootnoteSelection();

    // Check if this sentence is the anchor for a selected footnote sentence
    const isFootnoteAnchor = useMemo(() => {
        if (!selectedFootnoteSentenceId || !sentence.footnotes) return false;
        return sentence.footnotes.some((fn) =>
            fn.sentences.some((s) => s.id === selectedFootnoteSentenceId),
        );
    }, [selectedFootnoteSentenceId, sentence.footnotes]);

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
        const html = showOriginal && sentence.original_html ? sentence.original_html : sentence.html;
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

    return (
        <>
            {leftMarkers && <MarginNotes markers={leftMarkers} side="left" />}
            {rightMarkers && (
                <MarginNotes markers={rightMarkers} side="right" />
            )}
            <span
                onClick={() => onSelect(sentence)}
                className={`cursor-pointer transition-colors rounded-sm ${highlightClass}`}
            >
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
            <span>{parse(showOriginal && sentence.original_html ? sentence.original_html : sentence.html)}</span>{" "}
        </>
    );
}

export function Block({
    block,
    selectedSentenceId,
    showOriginal,
    onSelectSentence,
    marginSettings,
}: {
    block: ContentBlockResponse;
    selectedSentenceId: string | null;
    showOriginal?: boolean;
    onSelectSentence: (sentence: SentenceResponse) => void;
    marginSettings?: MarginSettings;
}) {
    const blockHtml = showOriginal && block.original_html ? block.original_html : block.html;

    switch (block.block_type) {
        case "heading":
            return (
                <h2 className="relative text-2xl font-bold mt-8 mb-6 text-stone-900">
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
                <p className="relative mb-4 leading-relaxed text-stone-700">
                    {block.sentences.map((s) => (
                        <Sentence
                            key={s.id}
                            sentence={s}
                            isSelected={sentenceMatchesKey(s, selectedSentenceId)}
                            showOriginal={showOriginal}
                            onSelect={onSelectSentence}
                            marginSettings={marginSettings}
                        />
                    ))}
                </p>
            );
        case "footnote":
            return (
                <div className="relative mb-4 ml-8 text-sm text-stone-500 italic border-l-2 border-stone-200 pl-4">
                    {block.sentences.map((s) => (
                        <Sentence
                            key={s.id}
                            sentence={s}
                            isSelected={sentenceMatchesKey(s, selectedSentenceId)}
                            showOriginal={showOriginal}
                            onSelect={onSelectSentence}
                            marginSettings={marginSettings}
                        />
                    ))}
                </div>
            );
        case "separator":
            return <hr className="my-8 border-stone-200" />;
        default:
            return <div className="mb-4">{parse(blockHtml)}</div>;
    }
}
