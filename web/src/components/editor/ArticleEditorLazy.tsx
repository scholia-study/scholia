import { lazy, Suspense, forwardRef } from "react";
import type { ArticleEditorHandle } from "./MdxEditor";

const MdxEditorModule = lazy(() =>
    import("./MdxEditor").then((mod) => ({ default: mod.ArticleEditor })),
);

interface ArticleEditorLazyProps {
    markdown: string;
    onChange: (markdown: string) => void;
    onInsertQuotationClick: () => void;
    readOnly?: boolean;
}

export const ArticleEditorLazy = forwardRef<ArticleEditorHandle, ArticleEditorLazyProps>(
    (props, ref) => {
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
    },
);

ArticleEditorLazy.displayName = "ArticleEditorLazy";

export type { ArticleEditorHandle } from "./MdxEditor";
