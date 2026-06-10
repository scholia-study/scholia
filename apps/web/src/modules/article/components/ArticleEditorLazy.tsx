import { forwardRef, lazy, Suspense } from "react";
import type { ArticleEditorHandle } from "./MdxEditor";

const MdxEditorModule = lazy(async () => {
    // @lexical/code (via MdxEditor's markdownShortcutPlugin) loads prismjs
    // language files that reference a bare global `Prism`. Set it from the
    // separate prism chunk before MdxEditor's graph evaluates, else prod
    // builds throw "Prism is not defined". Loads only when the editor opens.
    const Prism = (await import("prismjs")).default;
    if (typeof window !== "undefined") {
        (window as Window & { Prism?: typeof Prism }).Prism ??= Prism;
    }
    const mod = await import("./MdxEditor");
    return { default: mod.ArticleEditor };
});

interface ArticleEditorLazyProps {
    markdown: string;
    onChange: (markdown: string) => void;
    onInsertQuotationClick: () => void;
    readOnly?: boolean;
}

export const ArticleEditorLazy = forwardRef<
    ArticleEditorHandle,
    ArticleEditorLazyProps
>((props, ref) => {
    return (
        <Suspense
            fallback={
                <div className="p-4 text-sm text-stone-400 min-h-[400px] flex items-center justify-center">
                    Loading editor...
                </div>
            }
        >
            <MdxEditorModule ref={ref} {...props} />
        </Suspense>
    );
});

ArticleEditorLazy.displayName = "ArticleEditorLazy";

export type { ArticleEditorHandle } from "./MdxEditor";
