// @vitest-environment jsdom
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, render, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { BatchSentenceResponseItem, SentenceData } from "../../api/model";
import { QuotationCard } from "./QuotationCard";

vi.mock("@tanstack/react-router", () => ({
    Link: ({ children }: { children: React.ReactNode }) => (
        <span>{children}</span>
    ),
}));

const batchSentences = vi.hoisted(() => vi.fn());
vi.mock("../../api/sentences/sentences", () => ({ batchSentences }));

function itemWith(sentences: SentenceData[]): BatchSentenceResponseItem {
    return {
        book_slug: "emperor-and-galilean",
        book_title: "Emperor and Galilean",
        language: "en",
        node_slug: "foerste-handling",
        node_label: "First Act",
        citation: [],
        sentences,
    };
}

function sentence(n: number, html: string, speech?: [string, string, string]) {
    return {
        sentence_number: n,
        html,
        original_html: `<i>${html}</i>`,
        ...(speech
            ? {
                  speech_id: speech[0],
                  speaker_html: speech[1],
                  speaker_original_html: speech[2],
              }
            : {}),
    };
}

function renderCard(item: BatchSentenceResponseItem, mode = "translation") {
    batchSentences.mockResolvedValue({ data: { items: [item] } });
    const qc = new QueryClient({
        defaultOptions: { queries: { retry: false } },
    });
    return render(
        <QueryClientProvider client={qc}>
            <QuotationCard
                book="emperor-and-galilean"
                node="foerste-handling"
                start={10}
                end={13}
                kind="body"
                mode={mode as "translation" | "source" | "source+translation"}
                layout="stacked"
            />
        </QueryClientProvider>,
    );
}

/** The speaker lines the card renders, in document order. */
function speakers(container: HTMLElement): string[] {
    return Array.from(container.querySelectorAll("p.uppercase")).map(
        (el) => el.textContent ?? "",
    );
}

describe("QuotationCard drama attribution", () => {
    beforeEach(() => batchSentences.mockReset());
    afterEach(cleanup);

    it("labels each speech a range crosses", async () => {
        const { container } = renderCard(
            itemWith([
                sentence(10, "Pst, good friend?", [
                    "s1",
                    "Potamon the goldsmith.",
                    "Gullsmeden Potamon.",
                ]),
                sentence(11, "Don't know.", [
                    "s2",
                    "The soldier.",
                    "Soldaten.",
                ]),
                sentence(12, "The emperor?", [
                    "s3",
                    "Phocion the dyer.",
                    "Fargeren Fokion.",
                ]),
            ]),
        );
        await within(container).findByText("Pst, good friend?");
        expect(speakers(container)).toEqual([
            "Potamon the goldsmith.",
            "The soldier.",
            "Phocion the dyer.",
        ]);
    });

    it("labels a within-speech range once, keeping the sentences together", async () => {
        const speech: [string, string, string] = [
            "s3",
            "Phocion the dyer.",
            "Fargeren Fokion.",
        ];
        const { container } = renderCard(
            itemWith([
                sentence(12, "The emperor?", speech),
                sentence(13, "I thought someone asked.", speech),
            ]),
        );
        await within(container).findByText(/The emperor\?/);
        expect(speakers(container)).toEqual(["Phocion the dyer."]);
        expect(
            container.textContent?.includes(
                "The emperor? I thought someone asked.",
            ),
        ).toBe(true);
    });

    it("labels a single quoted sentence with its speech", async () => {
        const { container } = renderCard(
            itemWith([
                sentence(11, "Don't know.", [
                    "s2",
                    "The soldier.",
                    "Soldaten.",
                ]),
            ]),
        );
        await within(container).findByText("Don't know.");
        expect(speakers(container)).toEqual(["The soldier."]);
    });

    it("attributes the source pane in the source layer's language", async () => {
        const { container } = renderCard(
            itemWith([
                sentence(11, "Don't know.", [
                    "s2",
                    "The soldier.",
                    "Soldaten.",
                ]),
            ]),
            "source",
        );
        await within(container).findByText("Don't know.");
        expect(speakers(container)).toEqual(["Soldaten."]);
    });

    it("leaves non-drama passages unlabelled and run together", async () => {
        const { container } = renderCard(
            itemWith([
                sentence(114, "That all our cognition begins with experience."),
                sentence(115, "There is no doubt whatever."),
            ]),
        );
        await within(container).findByText(/That all our cognition/);
        expect(speakers(container)).toEqual([]);
        expect(
            container.textContent?.includes(
                "That all our cognition begins with experience. There is no doubt whatever.",
            ),
        ).toBe(true);
    });
});
