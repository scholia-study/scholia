/**
 * Tiny badge showing which translation/edition a quotation belongs to —
 * "KJV" / "WEB" for Bible-shape works, "DE" / "EN" for Kant. The label
 * is computed server-side (sources.publisher when short, else language
 * uppercase) and exposed on QuotationResponse / QuotationWithContext /
 * NoteWithContext as `translation_label`.
 *
 * Renders nothing if the label is missing or empty so it's safe to drop
 * in unconditionally.
 */
export function TranslationBadge({
    label,
    title,
    className,
}: {
    label: string | null | undefined;
    /** Hover title — typically the full book title for context. */
    title?: string;
    className?: string;
}) {
    if (!label) return null;
    return (
        <span
            className={`inline-block text-[10px] uppercase tracking-wide px-1 py-0 rounded border border-stone-300 text-stone-500 align-middle ${className ?? ""}`}
            title={title}
        >
            {label}
        </span>
    );
}
