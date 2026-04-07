import {
    Editor,
    defaultValueCtx,
    editorViewCtx,
    rootCtx,
} from "@milkdown/kit/core";
import { commonmark } from "@milkdown/kit/preset/commonmark";
import { history } from "@milkdown/kit/plugin/history";
import { listener, listenerCtx } from "@milkdown/kit/plugin/listener";
import { $prose, $view } from "@milkdown/kit/utils";
import { Plugin, PluginKey } from "@milkdown/kit/prose/state";
import { Milkdown, MilkdownProvider, useEditor, useInstance } from "@milkdown/react";
import {
    ProsemirrorAdapterProvider,
    useNodeViewFactory,
} from "@prosemirror-adapter/react";
import { forwardRef, useImperativeHandle } from "react";
import {
    quotationDirectiveNode,
    remarkDirective,
} from "./quotation-plugin";
import { EditorToolbar } from "./EditorToolbar";
import { QuotationNodeView } from "./QuotationNodeView";

/**
 * Plugin that ensures the document always ends with a paragraph,
 * so the user can always click/type below atom nodes like quotation cards.
 */
const trailingParagraph = $prose(() => {
    const key = new PluginKey("trailingParagraph");
    return new Plugin({
        key,
        appendTransaction: (_transactions, _oldState, newState) => {
            const { doc, schema, tr } = newState;
            const lastChild = doc.lastChild;
            if (!lastChild || lastChild.type.name !== "paragraph") {
                return tr.insert(doc.content.size, schema.nodes.paragraph.create());
            }
            return null;
        },
    });
});

export interface MilkdownEditorHandle {
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

interface EditorInnerProps {
    defaultValue: string;
    onChange: (markdown: string) => void;
}

const EditorInner = forwardRef<MilkdownEditorHandle, EditorInnerProps>(
    ({ defaultValue, onChange }, ref) => {
        const nodeViewFactory = useNodeViewFactory();

        useEditor(
            (root) => {
                return Editor.make()
                    .config((ctx) => {
                        ctx.set(rootCtx, root);
                        ctx.set(defaultValueCtx, defaultValue);
                        ctx.get(listenerCtx).markdownUpdated(
                            (_ctx, markdown, prevMarkdown) => {
                                if (markdown !== prevMarkdown) {
                                    onChange(markdown);
                                }
                            },
                        );
                    })
                    .use(commonmark)
                    .use(history)
                    .use(listener)
                    .use(trailingParagraph)
                    .use(remarkDirective)
                    .use(quotationDirectiveNode)
                    .use(
                        $view(quotationDirectiveNode, () =>
                            nodeViewFactory({
                                component: QuotationNodeView,
                                as: "div",
                            }),
                        ),
                    );
            },
            [],
        );

        const [loading, getInstance] = useInstance();

        useImperativeHandle(
            ref,
            () => ({
                insertQuotation: (attrs) => {
                    if (loading) return;
                    const editor = getInstance();
                    editor.action((ctx) => {
                        const view = ctx.get(editorViewCtx);
                        const { state, dispatch } = view;
                        const nodeType =
                            state.schema.nodes.quotationDirective;
                        if (!nodeType) return;

                        const node = nodeType.create({
                            book: attrs.book,
                            node: attrs.node,
                            start: attrs.start,
                            end: attrs.end ?? null,
                            kind: attrs.kind,
                            mode: attrs.mode,
                            layout: attrs.layout,
                        });

                        const { from } = state.selection;
                        const paragraph = state.schema.nodes.paragraph.create();
                        const tr = state.tr
                            .insert(from, node)
                            .insert(from + node.nodeSize, paragraph);
                        dispatch(tr);
                        view.focus();
                    });
                },
            }),
            [loading, getInstance],
        );

        return (
            <>
                <style>{`
                    .ProseMirror {
                        white-space: pre-wrap;
                        word-wrap: break-word;
                        outline: none;
                        padding: 1rem;
                        min-height: 400px;
                    }
                    .ProseMirror > * + * {
                        margin-top: 1em;
                    }
                    .ProseMirror p {
                        margin: 0;
                    }
                    .ProseMirror h1, .ProseMirror h2, .ProseMirror h3 {
                        margin: 0;
                        font-weight: 700;
                    }
                    .ProseMirror h1 { font-size: 1.5em; }
                    .ProseMirror h2 { font-size: 1.25em; }
                    .ProseMirror h3 { font-size: 1.1em; }
                    .ProseMirror blockquote {
                        border-left: 3px solid #d6d3d1;
                        padding-left: 1em;
                        margin: 0;
                        color: #78716c;
                    }
                    .ProseMirror ul, .ProseMirror ol {
                        padding-left: 1.5em;
                        margin: 0;
                    }
                    .ProseMirror code {
                        background: #f5f5f4;
                        padding: 0.15em 0.3em;
                        border-radius: 3px;
                        font-size: 0.9em;
                    }
                    .ProseMirror hr {
                        border: none;
                        border-top: 1px solid #d6d3d1;
                        margin: 1em 0;
                    }
                `}</style>
                <EditorToolbar />
                <Milkdown />
            </>
        );
    },
);

EditorInner.displayName = "EditorInner";

export const MilkdownEditorComponent = forwardRef<
    MilkdownEditorHandle,
    EditorInnerProps
>(({ defaultValue, onChange }, ref) => {
    return (
        <MilkdownProvider>
            <ProsemirrorAdapterProvider>
                <EditorInner
                    ref={ref}
                    defaultValue={defaultValue}
                    onChange={onChange}
                />
            </ProsemirrorAdapterProvider>
        </MilkdownProvider>
    );
});

MilkdownEditorComponent.displayName = "MilkdownEditorComponent";
