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
import { useGetBook } from "../../../api/books/books";
import { ArticleQuotationCard, QuotationCard } from "../../quotation";
import { type CitationEntry, CitationPopover } from "./CitationPopover";
import type { QuotationPickerResult } from "./QuotationPickerModal";

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

    // A book has a viewable source iff its translation_of source is
    // itself a hosted text — encoded in `source_book_slug`. Bibles
    // point at the canonical "The Bible" bibliographic root, which
    // has no books row → no source view possible. Suppress the mode
    // and layout controls in that case (and force display to
    // translation-only via the mode coercion below). We only fetch
    // when the popover is open to avoid book-detail queries for
    // every quotation embed in the editor.
    const { data: bookData } = useGetBook(attrs.book ?? "", {
        query: { enabled: !!attrs.book && !!anchorEl },
    });
    const hasSourceView = !!bookData?.data?.source_book_slug;
    const effectiveMode: QuotationMode = hasSourceView ? mode : "translation";

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
                mode={effectiveMode}
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
                    {hasSourceView ? (
                        <>
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
                            {mode === "source+translation" && (
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
                                                        updateAttr(
                                                            "layout",
                                                            value,
                                                        )
                                                    }
                                                    className="accent-stone-600"
                                                />
                                                {label}
                                            </label>
                                        ))}
                                    </div>
                                </fieldset>
                            )}
                        </>
                    ) : (
                        <p className="text-stone-400 italic">
                            No source-language view available for this work.
                        </p>
                    )}
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

function ArticleQuotationDirectiveEditor({
    mdastNode,
    parentEditor,
    lexicalNode,
}: DirectiveEditorProps) {
    const attrs = mdastNode.attributes ?? {};
    const id = (attrs.id as string) ?? "";

    const handleDelete = () => {
        parentEditor.update(() => {
            lexicalNode.remove();
        });
    };

    return (
        <div contentEditable={false} className="group relative">
            <ArticleQuotationCard id={id} />
            <div className="absolute -top-3 -right-3 flex flex-col gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
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
        </div>
    );
}

const articleQuotationDirectiveDescriptor: DirectiveDescriptor = {
    name: "article-quotation",
    testNode: (node) => node.name === "article-quotation",
    attributes: ["id"],
    hasChildren: false,
    type: "leafDirective",
    Editor: ArticleQuotationDirectiveEditor,
};

/** Parse the sources attribute: "uuid1:pages,uuid2:pages" */
function parseCiteSources(sourcesStr: string) {
    return sourcesStr
        .split(",")
        .filter(Boolean)
        .map((entry) => {
            const [id, pages] = entry.split(":");
            return { id: id?.trim() ?? "", pages: pages?.trim() ?? "" };
        });
}

function CitationDirectiveEditor({
    mdastNode,
    parentEditor,
    lexicalNode,
}: DirectiveEditorProps) {
    const attrs = mdastNode.attributes ?? {};
    const sourcesStr = (attrs.sources as string) ?? "";
    const label = (attrs.label as string) ?? "";
    const sourceNames = ((attrs.sourceNames as string) ?? "")
        .split("|")
        .filter(Boolean);
    const [anchorEl, setAnchorEl] = useState<HTMLElement | null>(null);

    const entries = parseCiteSources(sourcesStr);

    const displayText = label ? `(${label})` : "(cite)";

    const handleUpdate = (newEntries: CitationEntry[]) => {
        const sourcesValue = newEntries
            .map((e) => (e.pages ? `${e.sourceId}:${e.pages}` : e.sourceId))
            .join(",");
        const newLabel = buildCitationLabel(newEntries);
        const newSourceNames = newEntries.map((e) => e.sourceLabel).join("|");
        parentEditor.update(() => {
            lexicalNode.setMdastNode({
                ...mdastNode,
                attributes: {
                    ...attrs,
                    sources: sourcesValue,
                    label: newLabel,
                    sourceNames: newSourceNames,
                },
            });
        });
        setAnchorEl(null);
    };

    const handleDelete = () => {
        parentEditor.update(() => {
            lexicalNode.remove();
        });
    };

    return (
        <span contentEditable={false} className="inline">
            <span
                className="inline-flex items-center gap-0.5 bg-amber-50 border border-amber-200 rounded px-1 py-0.5 text-xs text-amber-800 cursor-pointer hover:bg-amber-100"
                onClick={(e) => setAnchorEl(e.currentTarget)}
                onKeyDown={(e) => {
                    if (e.key === "Backspace" || e.key === "Delete")
                        handleDelete();
                }}
            >
                {displayText}
            </span>
            <CitationPopover
                anchorEl={anchorEl}
                onClose={() => setAnchorEl(null)}
                onConfirm={handleUpdate}
                initialEntries={entries.map((e, i) => ({
                    sourceId: e.id,
                    sourceLabel: sourceNames[i] ?? e.id,
                    pages: e.pages,
                }))}
            />
        </span>
    );
}

/**
 * Build a Chicago author-date label from popover entries.
 * Each entry has sourceLabel like "Immanuel Kant — Kritik der reinen Vernunft"
 * We extract the author last name and year from it.
 */
function buildCitationLabel(
    entries: {
        sourceId: string;
        sourceLabel: string;
        pages: string;
        year?: string;
        authorLastName?: string;
    }[],
): string {
    return entries
        .map((e) => {
            const author = e.authorLastName ?? "Unknown";
            const year = e.year ?? "n.d.";
            return e.pages
                ? `${author} ${year}, ${e.pages}`
                : `${author} ${year}`;
        })
        .join("; ");
}

const citationDirectiveDescriptor: DirectiveDescriptor = {
    name: "cite",
    testNode: (node) => node.name === "cite",
    attributes: ["sources", "label", "sourceNames"],
    hasChildren: false,
    type: "textDirective",
    Editor: CitationDirectiveEditor,
};

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

function InsertCitationButton({
    onClick,
}: {
    onClick: (e: React.MouseEvent<HTMLButtonElement>) => void;
}) {
    return (
        <button
            type="button"
            onClick={onClick}
            className="px-2 py-1 text-xs rounded hover:bg-stone-100 text-stone-600"
            title="Insert Citation"
        >
            Cite
        </button>
    );
}

export interface ArticleEditorHandle {
    insertQuotation: (result: QuotationPickerResult) => void;
    insertCitation: (entries: { sourceId: string; pages: string }[]) => void;
}

interface ArticleEditorProps {
    markdown: string;
    onChange: (markdown: string) => void;
    onInsertQuotationClick: () => void;
    readOnly?: boolean;
}

export const ArticleEditor = forwardRef<
    ArticleEditorHandle,
    ArticleEditorProps
>(({ markdown, onChange, onInsertQuotationClick, readOnly }, ref) => {
    const editorRef = useRef<MDXEditorMethods>(null);
    const [citeAnchorEl, setCiteAnchorEl] = useState<HTMLElement | null>(null);

    useImperativeHandle(
        ref,
        () => ({
            insertQuotation: (result) => {
                if (!editorRef.current) return;
                if (result.source_type === "article") {
                    const directive = `\n::article-quotation{id="${result.id}"}\n`;
                    editorRef.current.insertMarkdown(directive);
                    return;
                }
                const parts = [
                    `book="${result.book}"`,
                    `node="${result.node}"`,
                    `start="${result.start}"`,
                ];
                if (result.end != null) {
                    parts.push(`end="${result.end}"`);
                }
                parts.push(
                    `kind="${result.kind}"`,
                    `mode="${result.mode}"`,
                    `layout="${result.layout}"`,
                );
                const directive = `\n::quotation{${parts.join(" ")}}\n`;
                editorRef.current.insertMarkdown(directive);
            },
            insertCitation: (entries) => {
                if (!editorRef.current) return;
                const sourcesValue = entries
                    .map((e) =>
                        e.pages ? `${e.sourceId}:${e.pages}` : e.sourceId,
                    )
                    .join(",");
                const directive = `:cite{sources="${sourcesValue}"}`;
                editorRef.current.insertMarkdown(directive);
            },
        }),
        [],
    );

    const handleCiteConfirm = (entries: CitationEntry[]) => {
        setCiteAnchorEl(null);
        if (!editorRef.current) return;
        const sourcesValue = entries
            .map((e) => (e.pages ? `${e.sourceId}:${e.pages}` : e.sourceId))
            .join(",");
        const label = buildCitationLabel(entries);
        const names = entries.map((e) => e.sourceLabel).join("|");
        const directive = `:cite{sources="${sourcesValue}" label="${label}" sourceNames="${names}"}`;
        editorRef.current.insertMarkdown(directive);
    };

    return (
        <>
            <MDXEditor
                ref={editorRef}
                markdown={markdown}
                onChange={onChange}
                readOnly={readOnly}
                contentEditableClassName="!prose !prose-stone max-w-none min-h-[400px] font-serif"
                plugins={[
                    headingsPlugin(),
                    listsPlugin(),
                    quotePlugin(),
                    thematicBreakPlugin(),
                    markdownShortcutPlugin(),
                    directivesPlugin({
                        directiveDescriptors: [
                            quotationDirectiveDescriptor,
                            articleQuotationDirectiveDescriptor,
                            citationDirectiveDescriptor,
                        ],
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
                                <InsertCitationButton
                                    onClick={(e) =>
                                        setCiteAnchorEl(
                                            e.currentTarget as HTMLElement,
                                        )
                                    }
                                />
                            </DiffSourceToggleWrapper>
                        ),
                    }),
                ]}
            />
            <CitationPopover
                anchorEl={citeAnchorEl}
                onClose={() => setCiteAnchorEl(null)}
                onConfirm={handleCiteConfirm}
            />
        </>
    );
});

ArticleEditor.displayName = "ArticleEditor";
