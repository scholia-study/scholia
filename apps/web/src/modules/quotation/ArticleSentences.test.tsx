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

function renderComponent() {
    const qc = new QueryClient({
        defaultOptions: { queries: { retry: false } },
    });
    return render(
        <QueryClientProvider client={qc}>
            <AuthProvider>
                <ArticleSentences
                    html={HTML}
                    articleId="a1"
                    replaceEmbed={() => <div data-testid="embed" />}
                />
            </AuthProvider>
        </QueryClientProvider>,
    );
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
