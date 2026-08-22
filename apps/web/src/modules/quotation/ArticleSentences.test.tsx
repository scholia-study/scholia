// @vitest-environment jsdom
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { AuthProvider } from "../../hooks/useAuth";
import { ArticleSentences, buildSentenceList } from "./ArticleSentences";

// Shape of a real rendered article: plain paragraphs, a blockquote whose
// content pulldown-cmark wraps in an inner <p>, and a quotation-embed
// placeholder. The blockquote's inner <p> is the historical trap: the
// sentence list counted it as an extra block while the render pass
// replaced the whole blockquote, shifting every key after it.
const HTML = [
    "<p>First point. Second point.</p>",
    "<blockquote>\n<p>Quoted line.</p>\n</blockquote>",
    "<p>After the quote. Final line.</p>",
    '<div class="quotation-embed" data-quotation-book="kant"><p>Embed text.</p></div>',
    "<p>Tail paragraph.</p>",
].join("\n");

// Inline markup that must survive segmentation: emphasis wholly inside
// one sentence, a link and code in another, and emphasis straddling the
// boundary between two sentences.
const INLINE_HTML = [
    '<p>A <em>bold</em> claim. See <a href="https://x.test">the docs</a> ',
    "and <code>run()</code>.</p>",
    "<p><strong>First. Second.</strong></p>",
].join("");

function renderComponent(html = HTML) {
    const qc = new QueryClient({
        defaultOptions: { queries: { retry: false } },
    });
    return render(
        <QueryClientProvider client={qc}>
            <AuthProvider>
                <ArticleSentences
                    html={html}
                    articleId="a1"
                    replaceEmbed={() => <div data-testid="embed" />}
                />
            </AuthProvider>
        </QueryClientProvider>,
    );
}

function sentenceHtml(container: HTMLElement): string[] {
    return Array.from(
        container.querySelectorAll("[data-article-sentence]"),
    ).map((el) => el.innerHTML);
}

describe("ArticleSentences", () => {
    it("renders span keys identical to the flat sentence list", () => {
        renderComponent();
        const spanKeys = Array.from(
            document.querySelectorAll("[data-article-sentence]"),
        ).map((el) => el.getAttribute("data-article-sentence"));
        expect(spanKeys.length).toBeGreaterThan(0);
        expect(spanKeys).toEqual(buildSentenceList(HTML).map((s) => s.key));
    });

    it("keeps inline markup inside a sentence", () => {
        const { container } = renderComponent(INLINE_HTML);
        const [first, second] = sentenceHtml(container);
        expect(first).toBe("A <em>bold</em> claim. ");
        expect(second).toBe(
            'See <a href="https://x.test">the docs</a> and <code>run()</code>.',
        );
    });

    it("splits an inline element that straddles a sentence boundary", () => {
        const { container } = renderComponent(INLINE_HTML);
        expect(sentenceHtml(container).slice(-2)).toEqual([
            "<strong>First. </strong>",
            "<strong>Second.</strong>",
        ]);
    });

    it("snapshots sentence html byte-identically to the rendered spans", () => {
        for (const source of [HTML, INLINE_HTML]) {
            const { container, unmount } = renderComponent(source);
            expect(sentenceHtml(container)).toEqual(
                buildSentenceList(source).map((s) => s.html),
            );
            unmount();
        }
    });

    it("counts a blockquote as one block and skips embed internals", () => {
        expect(buildSentenceList(HTML).map((s) => s.text)).toEqual([
            "First point.",
            "Second point.",
            "Quoted line.",
            "After the quote.",
            "Final line.",
            "Tail paragraph.",
        ]);
    });
});
