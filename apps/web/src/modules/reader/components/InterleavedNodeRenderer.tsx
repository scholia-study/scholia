import type {
    ContentBlockResponse,
    NodeDetail,
    SentenceResponse,
} from "../../../api/model";
import { sentenceMatchesKey } from "../keys";
import type { MarginSettings } from "./BlockRenderer";
import { Block, Sentence, Separator } from "./BlockRenderer";

type ViewLayout = "sp" | "ss" | "bpl" | "bpr" | "bsl" | "bsr";

interface InterleavedNodeRendererProps {
    primaryNode: NodeDetail;
    companionNode: NodeDetail | undefined;
    viewLayout: ViewLayout;
    selectedSentenceId: string | null;
    showOriginal: boolean;
    onSelectSentence: (sentence: SentenceResponse, shiftKey: boolean) => void;
    marginSettings?: MarginSettings;
    primaryLabel: string;
    companionLabel: string;
}

// --- Alignment utilities ---

interface AlignedBlock {
    type: "paragraph" | "heading" | "separator" | "other";
    primary?: ContentBlockResponse;
    companion?: ContentBlockResponse;
}

function alignBlocks(
    primaryBlocks: ContentBlockResponse[],
    companionBlocks: ContentBlockResponse[],
): AlignedBlock[] {
    const result: AlignedBlock[] = [];

    // Index companion blocks by paragraph_number and position for quick lookup
    const companionByParagraph = new Map<number, ContentBlockResponse>();
    const companionByPosition = new Map<number, ContentBlockResponse>();
    const usedCompanion = new Set<string>();

    for (const b of companionBlocks) {
        if (b.paragraph_number != null) {
            companionByParagraph.set(b.paragraph_number, b);
        }
        companionByPosition.set(b.position, b);
    }

    for (const block of primaryBlocks) {
        if (block.block_type === "separator") {
            result.push({ type: "separator", primary: block });
            continue;
        }

        let companion: ContentBlockResponse | undefined;

        if (
            block.block_type === "paragraph" ||
            block.block_type === "footnote"
        ) {
            if (block.paragraph_number != null) {
                companion = companionByParagraph.get(block.paragraph_number);
            }
        } else if (block.block_type === "heading") {
            companion = companionByPosition.get(block.position);
        }

        if (companion) {
            usedCompanion.add(companion.id);
        }

        result.push({
            type: block.block_type as AlignedBlock["type"],
            primary: block,
            companion,
        });
    }

    // Add any companion blocks that weren't matched
    for (const b of companionBlocks) {
        if (!usedCompanion.has(b.id) && b.block_type !== "separator") {
            result.push({
                type: b.block_type as AlignedBlock["type"],
                companion: b,
            });
        }
    }

    return result;
}

interface AlignedSentenceGroup {
    primary: SentenceResponse[];
    companion: SentenceResponse[];
}

function alignSentences(
    primarySentences: SentenceResponse[],
    companionSentences: SentenceResponse[],
): AlignedSentenceGroup[] {
    if (companionSentences.length === 0) {
        return primarySentences.map((s) => ({ primary: [s], companion: [] }));
    }
    if (primarySentences.length === 0) {
        return companionSentences.map((s) => ({ primary: [], companion: [s] }));
    }

    // Two alignment modes:
    //
    // 1. **sentence-link**: companion sentences carry
    //    `source_sentence_start_id` pointing at primary sentence ids.
    //    Used by Kant — every English sentence is explicitly linked to
    //    its German source. This is the most precise mode.
    //
    // 2. **marker**: neither side has sentence-id linkage but both
    //    carry the same kind of reference-system markers (e.g.
    //    `verse` for the Bible). Group both sides by `ref_value`,
    //    pair groups. KJV Gen 5:1 (2 sentences) maps to WEB Gen 5:1
    //    (1 sentence) and they render together.
    //
    // Detection runs in priority order: prefer sentence-link if any
    // companion sentence has it; otherwise try marker; otherwise
    // fall back to a degenerate "every primary alone, companion
    // appended after".
    const hasSentenceLink = companionSentences.some(
        (cs) => cs.source_sentence_start_id,
    );
    if (hasSentenceLink) {
        return alignBySentenceLink(primarySentences, companionSentences);
    }

    const sharedSystem = pickSharedMarkerSystem(
        primarySentences,
        companionSentences,
    );
    if (sharedSystem) {
        return alignByMarker(
            primarySentences,
            companionSentences,
            sharedSystem,
        );
    }

    // Fallback: no alignment hints. Show primary as-is, companion
    // dumped at the end. Better than crashing or interleaving wrong.
    return [
        ...primarySentences.map((s) => ({ primary: [s], companion: [] })),
        { primary: [], companion: companionSentences },
    ];
}

function alignBySentenceLink(
    primarySentences: SentenceResponse[],
    companionSentences: SentenceResponse[],
): AlignedSentenceGroup[] {
    // Build mapping from primary sentence ID -> companion sentences that reference it
    const companionBySource = new Map<string, SentenceResponse[]>();
    const unmatchedCompanion: SentenceResponse[] = [];

    for (const cs of companionSentences) {
        const startId = cs.source_sentence_start_id;
        if (startId) {
            if (!companionBySource.has(startId)) {
                companionBySource.set(startId, []);
            }
            companionBySource.get(startId)!.push(cs);
        } else {
            unmatchedCompanion.push(cs);
        }
    }

    const groups: AlignedSentenceGroup[] = [];
    const usedPrimary = new Set<string>();

    for (const ps of primarySentences) {
        if (usedPrimary.has(ps.id)) continue;

        const companions = companionBySource.get(ps.id) ?? [];

        // Check if any companion references a range (merge case)
        // In a merge, a companion sentence has start_id and end_id pointing to different primary sentences
        const mergePrimary = [ps];
        for (const cs of companions) {
            if (
                cs.source_sentence_end_id &&
                cs.source_sentence_end_id !== cs.source_sentence_start_id
            ) {
                // Find all primary sentences in the range
                const startIdx = primarySentences.findIndex(
                    (s) => s.id === cs.source_sentence_start_id,
                );
                const endIdx = primarySentences.findIndex(
                    (s) => s.id === cs.source_sentence_end_id,
                );
                if (startIdx >= 0 && endIdx >= 0) {
                    mergePrimary.length = 0;
                    for (let i = startIdx; i <= endIdx; i++) {
                        mergePrimary.push(primarySentences[i]);
                        usedPrimary.add(primarySentences[i].id);
                    }
                }
                break;
            }
        }

        for (const mp of mergePrimary) {
            usedPrimary.add(mp.id);
        }

        groups.push({
            primary: mergePrimary,
            companion: companions,
        });
    }

    // Add unmatched companion sentences
    if (unmatchedCompanion.length > 0) {
        groups.push({ primary: [], companion: unmatchedCompanion });
    }

    return groups;
}

/**
 * Find a reference-system slug that's present on at least one sentence
 * in BOTH primary and companion. Bible-shape books share the `verse`
 * system; auto-detection avoids hard-coding "Bible".
 */
function pickSharedMarkerSystem(
    primary: SentenceResponse[],
    companion: SentenceResponse[],
): string | null {
    const primarySystems = new Set<string>();
    for (const s of primary) {
        for (const m of s.page_markers) primarySystems.add(m.system_slug);
    }
    if (primarySystems.size === 0) return null;
    for (const s of companion) {
        for (const m of s.page_markers) {
            if (primarySystems.has(m.system_slug)) return m.system_slug;
        }
    }
    return null;
}

/**
 * Group sentences on each side by their first marker `ref_value` for
 * the chosen reference system, then pair groups by ref_value. Verse 5:1
 * on the primary side gets all primary sentences with marker "5:1"; the
 * companion column gets all companion sentences with marker "5:1".
 *
 * Sentences without a marker for the chosen system stick with the most
 * recent group (so a stray non-verse line in the middle of a chapter
 * doesn't get lost). Companion ref_values not present in primary get
 * their own primary-empty groups appended in encountered order.
 */
function alignByMarker(
    primary: SentenceResponse[],
    companion: SentenceResponse[],
    systemSlug: string,
): AlignedSentenceGroup[] {
    const refValueOf = (s: SentenceResponse): string | null => {
        for (const m of s.page_markers) {
            if (m.system_slug === systemSlug) return m.ref_value;
        }
        return null;
    };

    const order: string[] = [];
    const primaryByRef = new Map<string, SentenceResponse[]>();
    let currentRef: string | null = null;
    for (const s of primary) {
        const direct = refValueOf(s);
        const verseRef: string | null = direct ?? currentRef;
        if (!verseRef) continue; // primary preamble before any marker — skip
        currentRef = verseRef;
        if (!primaryByRef.has(verseRef)) {
            primaryByRef.set(verseRef, []);
            order.push(verseRef);
        }
        primaryByRef.get(verseRef)!.push(s);
    }

    const companionByRef = new Map<string, SentenceResponse[]>();
    currentRef = null;
    for (const s of companion) {
        const direct = refValueOf(s);
        const verseRef: string | null = direct ?? currentRef;
        if (!verseRef) continue;
        currentRef = verseRef;
        if (!companionByRef.has(verseRef)) {
            companionByRef.set(verseRef, []);
            if (!primaryByRef.has(verseRef)) order.push(verseRef);
        }
        companionByRef.get(verseRef)!.push(s);
    }

    return order.map((verseRef) => ({
        primary: primaryByRef.get(verseRef) ?? [],
        companion: companionByRef.get(verseRef) ?? [],
    }));
}

// --- Label component ---

function TextLabel({ label, color }: { label: string; color: string }) {
    return (
        <span
            className="text-[10px] uppercase tracking-wider select-none"
            style={{ color }}
        >
            {label}
        </span>
    );
}

// --- Stacked renderers ---

function StackedParagraphs({
    aligned,
    selectedSentenceId,
    showOriginal,
    onSelectSentence,
    marginSettings,
    primaryLabel,
    companionLabel,
}: {
    aligned: AlignedBlock[];
    selectedSentenceId: string | null;
    showOriginal: boolean;
    onSelectSentence: (sentence: SentenceResponse, shiftKey: boolean) => void;
    marginSettings?: MarginSettings;
    primaryLabel: string;
    companionLabel: string;
}) {
    return (
        <>
            {aligned.map((item, i) => {
                if (item.type === "separator") {
                    return (
                        <Separator
                            key={`sep-${i}`}
                            block={item.primary}
                            className="my-8"
                        />
                    );
                }
                return (
                    <div key={`aligned-${i}`}>
                        {item.primary && (
                            <div className="border-l-2 border-blue-300 pl-3 mb-1">
                                <TextLabel
                                    label={primaryLabel}
                                    color="#93c5fd"
                                />
                                <Block
                                    block={item.primary}
                                    selectedSentenceId={selectedSentenceId}
                                    showOriginal={showOriginal}
                                    onSelectSentence={onSelectSentence}
                                    marginSettings={marginSettings}
                                />
                            </div>
                        )}
                        {item.companion && (
                            <div className="border-l-2 border-amber-300 pl-3 mb-4">
                                <TextLabel
                                    label={companionLabel}
                                    color="#fcd34d"
                                />
                                <Block
                                    block={item.companion}
                                    selectedSentenceId={selectedSentenceId}
                                    showOriginal={showOriginal}
                                    onSelectSentence={onSelectSentence}
                                    marginSettings={marginSettings}
                                />
                            </div>
                        )}
                    </div>
                );
            })}
        </>
    );
}

function StackedSentences({
    aligned,
    selectedSentenceId,
    showOriginal,
    onSelectSentence,
    marginSettings,
    primaryLabel,
    companionLabel,
}: {
    aligned: AlignedBlock[];
    selectedSentenceId: string | null;
    showOriginal: boolean;
    onSelectSentence: (sentence: SentenceResponse, shiftKey: boolean) => void;
    marginSettings?: MarginSettings;
    primaryLabel: string;
    companionLabel: string;
}) {
    return (
        <>
            {aligned.map((item, i) => {
                if (item.type === "separator") {
                    return (
                        <Separator
                            key={`sep-${i}`}
                            block={item.primary}
                            className="my-8"
                        />
                    );
                }

                if (item.type === "heading") {
                    return (
                        <div key={`aligned-${i}`}>
                            {item.primary && (
                                <div className="border-l-2 border-blue-300 pl-3">
                                    <Block
                                        block={item.primary}
                                        selectedSentenceId={selectedSentenceId}
                                        showOriginal={showOriginal}
                                        onSelectSentence={onSelectSentence}
                                        marginSettings={marginSettings}
                                    />
                                </div>
                            )}
                            {item.companion && (
                                <div className="border-l-2 border-amber-300 pl-3">
                                    <Block
                                        block={item.companion}
                                        selectedSentenceId={selectedSentenceId}
                                        showOriginal={showOriginal}
                                        onSelectSentence={onSelectSentence}
                                        marginSettings={marginSettings}
                                    />
                                </div>
                            )}
                        </div>
                    );
                }

                // Sentence-level interleaving for paragraphs
                const groups = alignSentences(
                    item.primary?.sentences ?? [],
                    item.companion?.sentences ?? [],
                );

                return (
                    <div key={`aligned-${i}`} className="mb-4">
                        {groups.map((group, gi) => (
                            <div key={gi} className="mb-2">
                                {group.primary.length > 0 && (
                                    <p className="relative border-l-2 border-blue-300 pl-3 leading-[var(--reader-line-height)] text-stone-700">
                                        {gi === 0 && (
                                            <TextLabel
                                                label={primaryLabel}
                                                color="#93c5fd"
                                            />
                                        )}{" "}
                                        {group.primary.map((s) => (
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
                                            />
                                        ))}
                                    </p>
                                )}
                                {group.companion.length > 0 && (
                                    <p className="relative border-l-2 border-amber-300 pl-3 leading-[var(--reader-line-height)] text-stone-700">
                                        {gi === 0 && (
                                            <TextLabel
                                                label={companionLabel}
                                                color="#fcd34d"
                                            />
                                        )}{" "}
                                        {group.companion.map((s) => (
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
                                            />
                                        ))}
                                    </p>
                                )}
                            </div>
                        ))}
                    </div>
                );
            })}
        </>
    );
}

/** Force all margin systems to a specific side (for side-by-side to avoid gutter collisions). */
function forceMarginSide(
    settings: MarginSettings | undefined,
    side: "left" | "right",
): MarginSettings | undefined {
    if (!settings || settings.enabledSystems.size === 0) return settings;
    const forcedSides: Record<string, "left" | "right"> = {};
    for (const slug of Object.keys(settings.systemSides)) {
        forcedSides[slug] = side;
    }
    return {
        enabledSystems: settings.enabledSystems,
        systemSides: forcedSides,
    };
}

// --- Side-by-side renderers ---

function SideBySideParagraphs({
    aligned,
    selectedSentenceId,
    showOriginal,
    onSelectSentence,
    marginSettings,
    primaryLabel,
    companionLabel,
    primaryLeft,
}: {
    aligned: AlignedBlock[];
    selectedSentenceId: string | null;
    showOriginal: boolean;
    onSelectSentence: (sentence: SentenceResponse, shiftKey: boolean) => void;
    marginSettings?: MarginSettings;
    primaryLabel: string;
    companionLabel: string;
    primaryLeft: boolean;
}) {
    const leftLabel = primaryLeft ? primaryLabel : companionLabel;
    const rightLabel = primaryLeft ? companionLabel : primaryLabel;

    return (
        <div>
            <div className="grid grid-cols-2 gap-4 mb-4 border-b border-stone-200 pb-2">
                <div className="text-xs font-medium text-stone-400 uppercase tracking-wider">
                    {leftLabel}
                </div>
                <div className="text-xs font-medium text-stone-400 uppercase tracking-wider">
                    {rightLabel}
                </div>
            </div>
            {aligned.map((item, i) => {
                if (item.type === "separator") {
                    return (
                        <Separator
                            key={`sep-${i}`}
                            block={item.primary}
                            className="my-8"
                        />
                    );
                }

                const leftBlock = primaryLeft ? item.primary : item.companion;
                const rightBlock = primaryLeft ? item.companion : item.primary;
                // Primary gets margins forced to outer edge, companion gets none
                const leftMargins = primaryLeft
                    ? forceMarginSide(marginSettings, "left")
                    : undefined;
                const rightMargins = primaryLeft
                    ? undefined
                    : forceMarginSide(marginSettings, "right");

                return (
                    <div
                        key={`aligned-${i}`}
                        className="grid grid-cols-2 gap-4"
                    >
                        <div>
                            {leftBlock && (
                                <Block
                                    block={leftBlock}
                                    selectedSentenceId={selectedSentenceId}
                                    showOriginal={showOriginal}
                                    onSelectSentence={onSelectSentence}
                                    marginSettings={leftMargins}
                                />
                            )}
                        </div>
                        <div>
                            {rightBlock && (
                                <Block
                                    block={rightBlock}
                                    selectedSentenceId={selectedSentenceId}
                                    showOriginal={showOriginal}
                                    onSelectSentence={onSelectSentence}
                                    marginSettings={rightMargins}
                                />
                            )}
                        </div>
                    </div>
                );
            })}
        </div>
    );
}

function SideBySideSentences({
    aligned,
    selectedSentenceId,
    showOriginal,
    onSelectSentence,
    marginSettings,
    primaryLabel,
    companionLabel,
    primaryLeft,
}: {
    aligned: AlignedBlock[];
    selectedSentenceId: string | null;
    showOriginal: boolean;
    onSelectSentence: (sentence: SentenceResponse, shiftKey: boolean) => void;
    marginSettings?: MarginSettings;
    primaryLabel: string;
    companionLabel: string;
    primaryLeft: boolean;
}) {
    const leftLabel = primaryLeft ? primaryLabel : companionLabel;
    const rightLabel = primaryLeft ? companionLabel : primaryLabel;

    return (
        <div>
            <div className="grid grid-cols-2 gap-4 mb-4 border-b border-stone-200 pb-2">
                <div className="text-xs font-medium text-stone-400 uppercase tracking-wider">
                    {leftLabel}
                </div>
                <div className="text-xs font-medium text-stone-400 uppercase tracking-wider">
                    {rightLabel}
                </div>
            </div>
            {aligned.map((item, i) => {
                if (item.type === "separator") {
                    return (
                        <Separator
                            key={`sep-${i}`}
                            block={item.primary}
                            className="my-8"
                        />
                    );
                }

                if (item.type === "heading") {
                    const leftBlock = primaryLeft
                        ? item.primary
                        : item.companion;
                    const rightBlock = primaryLeft
                        ? item.companion
                        : item.primary;
                    return (
                        <div
                            key={`aligned-${i}`}
                            className="grid grid-cols-2 gap-4"
                        >
                            <div>
                                {leftBlock && (
                                    <Block
                                        block={leftBlock}
                                        selectedSentenceId={selectedSentenceId}
                                        showOriginal={showOriginal}
                                        onSelectSentence={onSelectSentence}
                                    />
                                )}
                            </div>
                            <div>
                                {rightBlock && (
                                    <Block
                                        block={rightBlock}
                                        selectedSentenceId={selectedSentenceId}
                                        showOriginal={showOriginal}
                                        onSelectSentence={onSelectSentence}
                                    />
                                )}
                            </div>
                        </div>
                    );
                }

                const groups = alignSentences(
                    item.primary?.sentences ?? [],
                    item.companion?.sentences ?? [],
                );

                const leftMargins = primaryLeft
                    ? forceMarginSide(marginSettings, "left")
                    : undefined;
                const rightMargins = primaryLeft
                    ? undefined
                    : forceMarginSide(marginSettings, "right");

                return (
                    <div key={`aligned-${i}`} className="mb-4">
                        {groups.map((group, gi) => {
                            const leftSentences = primaryLeft
                                ? group.primary
                                : group.companion;
                            const rightSentences = primaryLeft
                                ? group.companion
                                : group.primary;
                            return (
                                <div
                                    key={gi}
                                    className="grid grid-cols-2 gap-4 mb-1"
                                >
                                    <p className="relative leading-[var(--reader-line-height)] text-stone-700">
                                        {leftSentences.map((s) => (
                                            <Sentence
                                                key={s.id}
                                                sentence={s}
                                                isSelected={sentenceMatchesKey(
                                                    s,
                                                    selectedSentenceId,
                                                )}
                                                showOriginal={showOriginal}
                                                onSelect={onSelectSentence}
                                                marginSettings={leftMargins}
                                            />
                                        ))}
                                    </p>
                                    <p className="relative leading-[var(--reader-line-height)] text-stone-700">
                                        {rightSentences.map((s) => (
                                            <Sentence
                                                key={s.id}
                                                sentence={s}
                                                isSelected={sentenceMatchesKey(
                                                    s,
                                                    selectedSentenceId,
                                                )}
                                                showOriginal={showOriginal}
                                                onSelect={onSelectSentence}
                                                marginSettings={rightMargins}
                                            />
                                        ))}
                                    </p>
                                </div>
                            );
                        })}
                    </div>
                );
            })}
        </div>
    );
}

// --- Main component ---

export function InterleavedNodeRenderer({
    primaryNode,
    companionNode,
    viewLayout,
    selectedSentenceId,
    showOriginal,
    onSelectSentence,
    marginSettings,
    primaryLabel,
    companionLabel,
}: InterleavedNodeRendererProps) {
    const aligned = alignBlocks(
        primaryNode.blocks,
        companionNode?.blocks ?? [],
    );

    const isStacked = viewLayout === "sp" || viewLayout === "ss";
    const isSentenceLevel =
        viewLayout === "ss" || viewLayout === "bsl" || viewLayout === "bsr";
    const primaryLeft = viewLayout === "bpl" || viewLayout === "bsl";

    if (isStacked) {
        if (isSentenceLevel) {
            return (
                <StackedSentences
                    aligned={aligned}
                    selectedSentenceId={selectedSentenceId}
                    showOriginal={showOriginal}
                    onSelectSentence={onSelectSentence}
                    marginSettings={marginSettings}
                    primaryLabel={primaryLabel}
                    companionLabel={companionLabel}
                />
            );
        }
        return (
            <StackedParagraphs
                aligned={aligned}
                selectedSentenceId={selectedSentenceId}
                showOriginal={showOriginal}
                onSelectSentence={onSelectSentence}
                marginSettings={marginSettings}
                primaryLabel={primaryLabel}
                companionLabel={companionLabel}
            />
        );
    }

    if (isSentenceLevel) {
        return (
            <SideBySideSentences
                aligned={aligned}
                selectedSentenceId={selectedSentenceId}
                showOriginal={showOriginal}
                onSelectSentence={onSelectSentence}
                marginSettings={marginSettings}
                primaryLabel={primaryLabel}
                companionLabel={companionLabel}
                primaryLeft={primaryLeft}
            />
        );
    }

    return (
        <SideBySideParagraphs
            aligned={aligned}
            selectedSentenceId={selectedSentenceId}
            showOriginal={showOriginal}
            onSelectSentence={onSelectSentence}
            marginSettings={marginSettings}
            primaryLabel={primaryLabel}
            companionLabel={companionLabel}
            primaryLeft={primaryLeft}
        />
    );
}
