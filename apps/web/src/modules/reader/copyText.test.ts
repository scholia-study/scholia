import { describe, expect, it } from "vitest";
import type { NodeDetail } from "../../api/model";
import { rangeCopyText } from "./copyText";

function block(blockType: string, sentences: [number | null, string][]) {
    return {
        block_type: blockType,
        sentences: sentences.map(([sentence_number, text]) => ({
            sentence_number,
            text,
        })),
    };
}

function node(...blocks: ReturnType<typeof block>[]): NodeDetail {
    return { blocks } as unknown as NodeDetail;
}

const dialogue = node(
    block("speaker", [[null, "Socrates"]]),
    block("paragraph", [[1, "Is this a question you're asking?"]]),
    block("speaker", [[null, "Polus"]]),
    block("paragraph", [[2, "I'm asking, yes."]]),
    block("stage", [[3, "He pauses."]]),
    block("speaker", [[null, "Socrates"]]),
    block("paragraph", [
        [4, "It doesn't seem so to me."],
        [5, "Not at all."],
    ]),
);

describe("rangeCopyText", () => {
    it("writes a multi-speaker selection as a script", () => {
        expect(rangeCopyText([dialogue], 1, 5)).toBe(
            [
                "Socrates: Is this a question you're asking?",
                "Polus: I'm asking, yes.",
                "He pauses.",
                "Socrates: It doesn't seem so to me. Not at all.",
            ].join("\n"),
        );
    });

    it("keeps the speaker of a speech that starts before the range", () => {
        expect(rangeCopyText([dialogue], 2, 4)).toBe(
            "Polus: I'm asking, yes.\nHe pauses.\nSocrates: It doesn't seem so to me.",
        );
    });

    it("leaves a single speaker's words unlabelled", () => {
        expect(rangeCopyText([dialogue], 4, 5)).toBe(
            "It doesn't seem so to me. Not at all.",
        );
    });

    it("joins prose sentences with spaces", () => {
        const prose = node(
            block("paragraph", [[1, "One."]]),
            block("paragraph", [[2, "Two."]]),
        );
        expect(rangeCopyText([prose], 1, 2)).toBe("One. Two.");
    });
});
