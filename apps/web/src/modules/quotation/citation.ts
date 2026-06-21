import type { CitationPart } from "../../api/model";

const EN_DASH = "–";
const MIDDOT = "·";
/** Separator between multiple citation systems (e.g. Kant A/B). */
const CITATION_COMBINER = ` ${MIDDOT} `;

/** Render one citation part's template, substituting node labels and the
 *  `{ref}` range (`first` or `first–last`). */
function renderPart(
    part: CitationPart,
    parentNodeLabel: string | null | undefined,
    nodeLabel: string,
): string {
    const ref =
        part.last_ref && part.last_ref !== part.first_ref
            ? `${part.first_ref}${EN_DASH}${part.last_ref}`
            : part.first_ref;
    return part.template
        .replaceAll("{parent}", parentNodeLabel ?? "")
        .replaceAll("{self}", nodeLabel)
        .replaceAll("{ref}", ref)
        .trim();
}

/** The passage locator shown after the book title in a citation. Driven by the
 *  book's declared citation systems (`item.citation`); falls back to
 *  `Node · s. N` when the book has no default citation system. */
export function formatPassageCitation(args: {
    citation: CitationPart[];
    parentNodeLabel?: string | null;
    nodeLabel: string;
    sentenceLabel: string;
}): string {
    const { citation, parentNodeLabel, nodeLabel, sentenceLabel } = args;
    if (citation.length > 0) {
        return citation
            .map((part) => renderPart(part, parentNodeLabel, nodeLabel))
            .join(CITATION_COMBINER);
    }
    return `${nodeLabel} ${MIDDOT} ${sentenceLabel}`;
}
