import { $node, $remark } from "@milkdown/kit/utils";
import type { Node } from "@milkdown/kit/prose/model";
import directive from "remark-directive";

/**
 * Remark plugin to enable directive syntax (:quotation{...})
 */
export const remarkDirective = $remark("remarkDirective", () => directive);

/**
 * Custom Milkdown node for :quotation{book="..." node="..." start=123 ...} directives.
 * Renders as an atom (non-editable) block node.
 */
export const quotationDirectiveNode = $node("quotationDirective", () => ({
    group: "block",
    atom: true,
    isolating: true,
    marks: "",
    attrs: {
        book: { default: "" },
        node: { default: "" },
        start: { default: 0 },
        end: { default: null },
        kind: { default: "body" },
        mode: { default: "translation" },
        layout: { default: "stacked" },
    },
    parseDOM: [
        {
            tag: 'div[data-quotation-directive]',
            getAttrs: (dom) => {
                const el = dom as HTMLElement;
                return {
                    book: el.getAttribute("data-book") ?? "",
                    node: el.getAttribute("data-node") ?? "",
                    start: Number(el.getAttribute("data-start")) || 0,
                    end: el.getAttribute("data-end")
                        ? Number(el.getAttribute("data-end"))
                        : null,
                    kind: el.getAttribute("data-kind") ?? "body",
                    mode: el.getAttribute("data-mode") ?? "translation",
                    layout: el.getAttribute("data-layout") ?? "stacked",
                };
            },
        },
    ],
    toDOM: (pmNode: Node) => [
        "div",
        {
            "data-quotation-directive": "true",
            "data-book": pmNode.attrs.book,
            "data-node": pmNode.attrs.node,
            "data-start": String(pmNode.attrs.start),
            "data-end": pmNode.attrs.end != null ? String(pmNode.attrs.end) : "",
            "data-kind": pmNode.attrs.kind,
            "data-mode": pmNode.attrs.mode,
            "data-layout": pmNode.attrs.layout,
            contenteditable: "false",
        },
    ],
    parseMarkdown: {
        match: (mdNode) =>
            mdNode.type === "leafDirective" &&
            (mdNode as { name?: string }).name === "quotation",
        runner: (state, mdNode, type) => {
            const attrs = (mdNode as { attributes?: Record<string, string> })
                .attributes ?? {};
            state.addNode(type, {
                book: attrs.book ?? "",
                node: attrs.node ?? "",
                start: Number(attrs.start) || 0,
                end: attrs.end ? Number(attrs.end) : null,
                kind: attrs.kind ?? "body",
                mode: attrs.mode ?? "translation",
                layout: attrs.layout ?? "stacked",
            });
        },
    },
    toMarkdown: {
        match: (pmNode) => pmNode.type.name === "quotationDirective",
        runner: (state, pmNode) => {
            const attrs: Record<string, string> = {
                book: pmNode.attrs.book,
                node: pmNode.attrs.node,
                start: String(pmNode.attrs.start),
                kind: pmNode.attrs.kind,
                mode: pmNode.attrs.mode,
                layout: pmNode.attrs.layout,
            };
            if (pmNode.attrs.end != null) {
                attrs.end = String(pmNode.attrs.end);
            }
            state.addNode("leafDirective", undefined, undefined, {
                name: "quotation",
                attributes: attrs,
            });
        },
    },
}));
