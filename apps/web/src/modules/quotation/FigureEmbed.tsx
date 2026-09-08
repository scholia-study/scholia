/** The shared figure-box visual contract: caption pinned bottom-right,
 *  muted and small — the same look the reader gives corpus figures
 *  (BlockRenderer) and figure quotations (QuotationCard). */
export const FIGURE_FRAME_CLASSES =
    "not-prose whitespace-normal [&_figure]:relative [&_figure]:pb-8 [&_figcaption]:absolute [&_figcaption]:right-2 [&_figcaption]:bottom-0 [&_figcaption]:text-right [&_figcaption]:text-sm [&_figcaption]:text-stone-400";

/** Hydrate a sanitized `div.figure-embed` (its `data-figure-*` attributes)
 *  into the real figure markup. Shared by every article-HTML renderer. */
export function figureEmbedFromAttribs(attrs: Record<string, string>) {
    const src = attrs["data-figure-src"];
    if (!src) return null;
    return (
        <FigureEmbed
            src={src}
            alt={attrs["data-figure-alt"]}
            caption={attrs["data-figure-caption"]}
            width={Number(attrs["data-figure-width"]) || undefined}
            height={Number(attrs["data-figure-height"]) || undefined}
        />
    );
}

export interface FigureEmbedProps {
    src: string;
    alt?: string;
    caption?: string;
    width?: number;
    height?: number;
}

/** Renders an article figure-embed (`::figure{}` directive output) as the
 *  real `<figure><img><figcaption>` markup. Only first-party uploads are
 *  rendered — defense in depth on top of the server-side src validation. */
export function FigureEmbed({
    src,
    alt,
    caption,
    width,
    height,
}: FigureEmbedProps) {
    if (!src.startsWith("/media/")) return null;
    return (
        <div className={`my-6 ${FIGURE_FRAME_CLASSES}`}>
            <figure className={caption ? undefined : "pb-0!"}>
                <img
                    src={src}
                    alt={alt ?? ""}
                    width={width}
                    height={height}
                    loading="lazy"
                    className="mx-auto h-auto max-w-full"
                />
                {caption ? <figcaption>{caption}</figcaption> : null}
            </figure>
        </div>
    );
}
