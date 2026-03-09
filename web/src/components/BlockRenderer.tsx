import parse from "html-react-parser";
import type {
    ContentBlockResponse,
    PageMarkerResponse,
    SentenceResponse,
} from "../api/model";

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
            {markers.map((pm, i) => (
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
            <span
                onClick={() => onSelect(sentence)}
                className={`cursor-pointer transition-colors rounded-sm ${
                    isSelected ? "bg-amber-200" : "hover:bg-stone-200"
                }`}
            >
                {parse(showOriginal && sentence.original_html ? sentence.original_html : sentence.html)}
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
