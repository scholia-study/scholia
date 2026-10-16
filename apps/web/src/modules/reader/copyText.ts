import type { NodeDetail } from "../../api/model";

interface Turn {
    speaker: string | null;
    lines: string[];
}

/** Plain text of sentences `start..=end` for the clipboard. A selection that
 *  spans more than one speaker in a play is written as a script, one
 *  `Speaker: …` line per speech, since the text alone loses who says what. */
export function rangeCopyText(
    nodes: NodeDetail[],
    start: number,
    end: number,
): string {
    const turns: Turn[] = [];
    for (const node of nodes) {
        let speaker: string | null = null;
        for (const block of node.blocks) {
            if (block.block_type === "speaker") {
                speaker = block.sentences
                    .map((s) => s.text.trim())
                    .filter(Boolean)
                    .join(" ");
                continue;
            }
            const lines = block.sentences
                .filter(
                    (s) =>
                        s.sentence_number != null &&
                        s.sentence_number >= start &&
                        s.sentence_number <= end,
                )
                .map((s) => s.text.trim())
                .filter(Boolean);
            if (lines.length === 0) continue;
            const who = block.block_type === "stage" ? null : speaker;
            const last = turns.at(-1);
            if (last && last.speaker === who) last.lines.push(...lines);
            else turns.push({ speaker: who, lines });
        }
    }

    const speakers = new Set(turns.map((t) => t.speaker).filter(Boolean));
    if (speakers.size < 2) return turns.flatMap((t) => t.lines).join(" ");
    return turns
        .map((t) =>
            t.speaker
                ? `${t.speaker}: ${t.lines.join(" ")}`
                : t.lines.join(" "),
        )
        .join("\n");
}
