import parse from "html-react-parser";
import { marked } from "marked";
import { useMemo } from "react";

interface StaticMDRendererProps {
    source: string;
    className?: string;
}

/**
 * Render a static markdown string as HTML, parsed into React elements via
 * html-react-parser. Use with Vite's `?raw` import for build-time inlined
 * content:
 *
 *   import aboutMd from "../content/about.md?raw";
 *   <StaticMDRenderer source={aboutMd} />
 *
 * Named to distinguish it from runtime markdown editing (Milkdown / MDXEditor)
 * and from server-rendered article markdown.
 */
export function StaticMDRenderer({ source, className }: StaticMDRendererProps) {
    const html = useMemo(
        () => marked.parse(source, { async: false }) as string,
        [source],
    );
    return (
        <div className={`prose prose-stone max-w-none ${className ?? ""}`}>
            {parse(html)}
        </div>
    );
}
