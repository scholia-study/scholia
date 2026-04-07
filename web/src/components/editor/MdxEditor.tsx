import "@mdxeditor/editor/style.css";
import {
    BlockTypeSelect,
    BoldItalicUnderlineToggles,
    CodeToggle,
    DiffSourceToggleWrapper,
    type DirectiveDescriptor,
    type DirectiveEditorProps,
    diffSourcePlugin,
    directivesPlugin,
    headingsPlugin,
    InsertThematicBreak,
    ListsToggle,
    listsPlugin,
    MDXEditor,
    type MDXEditorMethods,
    markdownShortcutPlugin,
    quotePlugin,
    Separator,
    thematicBreakPlugin,
    toolbarPlugin,
} from "@mdxeditor/editor";
import { forwardRef, useImperativeHandle, useRef } from "react";
import { QuotationCard } from "../QuotationCard";

// ── Quotation directive descriptor ────────────────────────

function QuotationDirectiveEditor({
    mdastNode,
    parentEditor,
    lexicalNode,
}: DirectiveEditorProps) {
    const attrs = mdastNode.attributes ?? {};

    const handleDelete = () => {
        parentEditor.update(() => {
            lexicalNode.remove();
        });
    };

    return (
        <div contentEditable={false} className="group relative">
            <QuotationCard
                book={attrs.book ?? ""}
                node={attrs.node ?? ""}
                start={Number(attrs.start) || 0}
                end={attrs.end ? Number(attrs.end) : undefined}
                kind={attrs.kind ?? "body"}
                mode={
                    (attrs.mode as
                        | "source"
                        | "translation"
                        | "source+translation") ?? "translation"
                }
                layout={
                    (attrs.layout as
                        | "stacked"
                        | "side-by-side-source-left"
                        | "side-by-side-source-right") ?? "stacked"
                }
            />
            <button
                type="button"
                onClick={handleDelete}
                title="Remove quotation"
                className="absolute top-4 right-10 opacity-0 group-hover:opacity-100 transition-opacity bg-white/80 rounded p-1 text-stone-400 hover:text-stone-600"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                >
                    <title>Remove quote</title>
                    <path d="M3 6h18" />
                    <path d="M8 6V4h8v2" />
                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
                    <path d="M10 11v6" />
                    <path d="M14 11v6" />
                </svg>
            </button>
        </div>
    );
}

const quotationDirectiveDescriptor: DirectiveDescriptor = {
    name: "quotation",
    testNode: (node) => node.name === "quotation",
    attributes: ["book", "node", "start", "end", "kind", "mode", "layout"],
    hasChildren: false,
    type: "leafDirective",
    Editor: QuotationDirectiveEditor,
};

// ── Toolbar quotation button ──────────────────────────────

function InsertQuotationButton({ onClick }: { onClick: () => void }) {
    return (
        <button
            type="button"
            onClick={onClick}
            className="px-2 py-1 text-xs rounded hover:bg-stone-100 text-stone-600"
            title="Insert Quotation"
        >
            Quotation
        </button>
    );
}

// ── Editor component ──────────────────────────────────────

export interface ArticleEditorHandle {
    insertQuotation: (attrs: {
        book: string;
        node: string;
        start: number;
        end?: number;
        kind: string;
        mode: string;
        layout: string;
    }) => void;
}

interface ArticleEditorProps {
    markdown: string;
    onChange: (markdown: string) => void;
    onInsertQuotationClick: () => void;
}

export const ArticleEditor = forwardRef<
    ArticleEditorHandle,
    ArticleEditorProps
>(({ markdown, onChange, onInsertQuotationClick }, ref) => {
    const editorRef = useRef<MDXEditorMethods>(null);

    useImperativeHandle(
        ref,
        () => ({
            insertQuotation: (attrs) => {
                if (!editorRef.current) return;
                const parts = [
                    `book="${attrs.book}"`,
                    `node="${attrs.node}"`,
                    `start="${attrs.start}"`,
                ];
                if (attrs.end != null) {
                    parts.push(`end="${attrs.end}"`);
                }
                parts.push(
                    `kind="${attrs.kind}"`,
                    `mode="${attrs.mode}"`,
                    `layout="${attrs.layout}"`,
                );
                const directive = `\n::quotation{${parts.join(" ")}}\n`;
                editorRef.current.insertMarkdown(directive);
            },
        }),
        [],
    );

    return (
        <MDXEditor
            ref={editorRef}
            markdown={markdown}
            onChange={onChange}
            contentEditableClassName="!prose !prose-stone max-w-none min-h-[400px] font-serif"
            plugins={[
                headingsPlugin(),
                listsPlugin(),
                quotePlugin(),
                thematicBreakPlugin(),
                markdownShortcutPlugin(),
                directivesPlugin({
                    directiveDescriptors: [quotationDirectiveDescriptor],
                }),
                diffSourcePlugin({
                    diffMarkdown:
                        markdown ?? "No differences to show. Ignore this.",
                    readOnlyDiff: true,
                    viewMode: "rich-text",
                }),
                toolbarPlugin({
                    toolbarContents: () => (
                        <DiffSourceToggleWrapper>
                            <BoldItalicUnderlineToggles />
                            <CodeToggle />
                            <Separator />
                            <BlockTypeSelect />
                            <Separator />
                            <ListsToggle />
                            <InsertThematicBreak />
                            <Separator />
                            <InsertQuotationButton
                                onClick={onInsertQuotationClick}
                            />
                        </DiffSourceToggleWrapper>
                    ),
                }),
            ]}
        />
    );
});

ArticleEditor.displayName = "ArticleEditor";
