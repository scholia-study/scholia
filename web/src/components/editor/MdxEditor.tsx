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
import { Popover } from "@mui/material";
import { forwardRef, useImperativeHandle, useRef, useState } from "react";
import { QuotationCard } from "../QuotationCard";

// ── Quotation directive descriptor ────────────────────────

type QuotationMode = "source" | "translation" | "source+translation";
type QuotationLayout =
    | "stacked"
    | "side-by-side-source-left"
    | "side-by-side-source-right";

function QuotationDirectiveEditor({
    mdastNode,
    parentEditor,
    lexicalNode,
}: DirectiveEditorProps) {
    const attrs = mdastNode.attributes ?? {};
    const [anchorEl, setAnchorEl] = useState<HTMLElement | null>(null);

    const mode = (attrs.mode as QuotationMode) ?? "translation";
    const layout = (attrs.layout as QuotationLayout) ?? "stacked";

    const updateAttr = (key: string, value: string) => {
        parentEditor.update(() => {
            lexicalNode.setMdastNode({
                ...mdastNode,
                attributes: { ...attrs, [key]: value },
            });
        });
    };

    const handleDelete = () => {
        setAnchorEl(null);
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
                mode={mode}
                layout={layout}
            />
            <div className="absolute -top-3 -right-3 flex flex-col gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                <button
                    type="button"
                    onClick={(e) => setAnchorEl(e.currentTarget)}
                    title="Edit quotation"
                    className="bg-white rounded-full p-1.5 text-stone-400 hover:text-stone-600 shadow-sm border border-stone-200"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="20"
                        height="20"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                    >
                        <title>Edit quotation</title>
                        <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
                        <path d="m15 5 4 4" />
                    </svg>
                </button>
                <button
                    type="button"
                    onClick={handleDelete}
                    title="Remove quotation"
                    className="bg-white rounded-full p-1.5 text-stone-400 hover:text-red-500 shadow-sm border border-stone-200"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="20"
                        height="20"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                    >
                        <title>Remove quotation</title>
                        <path d="M3 6h18" />
                        <path d="M8 6V4h8v2" />
                        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
                        <path d="M10 11v6" />
                        <path d="M14 11v6" />
                    </svg>
                </button>
            </div>
            <Popover
                open={!!anchorEl}
                anchorEl={anchorEl}
                onClose={() => setAnchorEl(null)}
                anchorOrigin={{ vertical: "top", horizontal: "center" }}
                transformOrigin={{ vertical: "bottom", horizontal: "center" }}
                slotProps={{
                    paper: {
                        sx: { p: 2, mb: 1, maxWidth: 420 },
                    },
                }}
            >
                <div className="space-y-3 text-xs">
                    <fieldset>
                        <legend className="text-stone-400 font-medium mb-1">
                            Display mode
                        </legend>
                        <div className="flex gap-3">
                            {(
                                [
                                    ["source", "Source"],
                                    ["translation", "Translation"],
                                    ["source+translation", "Both"],
                                ] as const
                            ).map(([value, label]) => (
                                <label
                                    key={value}
                                    className="flex items-center gap-1 cursor-pointer"
                                >
                                    <input
                                        type="radio"
                                        name="quotation-mode"
                                        checked={mode === value}
                                        onChange={() =>
                                            updateAttr("mode", value)
                                        }
                                        className="accent-stone-600"
                                    />
                                    {label}
                                </label>
                            ))}
                        </div>
                    </fieldset>
                    <fieldset>
                        <legend className="text-stone-400 font-medium mb-1">
                            Layout
                        </legend>
                        <div className="flex gap-3">
                            {(
                                [
                                    ["stacked", "Stacked"],
                                    [
                                        "side-by-side-source-left",
                                        "Side-by-side (L)",
                                    ],
                                    [
                                        "side-by-side-source-right",
                                        "Side-by-side (R)",
                                    ],
                                ] as const
                            ).map(([value, label]) => (
                                <label
                                    key={value}
                                    className="flex items-center gap-1 cursor-pointer"
                                >
                                    <input
                                        type="radio"
                                        name="quotation-layout"
                                        checked={layout === value}
                                        onChange={() =>
                                            updateAttr("layout", value)
                                        }
                                        className="accent-stone-600"
                                    />
                                    {label}
                                </label>
                            ))}
                        </div>
                    </fieldset>
                </div>
            </Popover>
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
