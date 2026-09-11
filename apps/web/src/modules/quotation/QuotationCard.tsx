import { Paper, Skeleton } from "@mui/material";
import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import parse from "html-react-parser";
import { Fragment } from "react";
import type { SentenceData, SentenceKind } from "../../api/model";
import { batchSentences } from "../../api/sentences/sentences";
import { formatPassageCitation } from "./citation";

/** A run of consecutive quoted sentences belonging to one drama speech.
 *  Outside drama every sentence carries a null `speech_id`, so the whole
 *  passage collapses into a single unlabelled run. */
interface SpeechRun {
    key: string;
    speakerHtml: string | null;
    html: string;
}

function speechRuns(
    sentences: SentenceData[],
    layer: "translation" | "source",
): SpeechRun[] {
    const runs: SpeechRun[] = [];
    let lastSpeechId: string | null = null;
    for (const s of sentences) {
        const html = layer === "source" ? s.original_html : s.html;
        if (!html) continue;
        const speechId = s.speech_id ?? null;
        const previous = runs.at(-1);
        if (previous && speechId === lastSpeechId) {
            previous.html += ` ${html}`;
            continue;
        }
        lastSpeechId = speechId;
        runs.push({
            key: speechId ?? `run-${runs.length}`,
            speakerHtml:
                (layer === "source"
                    ? (s.speaker_original_html ?? s.speaker_html)
                    : s.speaker_html) ?? null,
            html,
        });
    }
    return runs;
}

/** The quoted passage for one layer: a figure's verbatim markup, or the
 *  speech runs with the curated speaker line heading each one — so a range
 *  crossing several speeches reads as a play excerpt, not run-on prose. */
function PassageBody({
    figureHtml,
    runs,
}: {
    figureHtml?: string | null;
    runs: SpeechRun[];
}) {
    if (figureHtml) return <>{parse(figureHtml)}</>;
    return (
        <>
            {runs.map((run, i) =>
                run.speakerHtml ? (
                    <div key={run.key} className={i > 0 ? "mt-3" : ""}>
                        <p className="mb-0.5 font-bold uppercase tracking-wide text-stone-500 leading-snug [&_i]:font-normal [&_i]:normal-case">
                            {parse(run.speakerHtml)}
                        </p>
                        {parse(run.html)}
                    </div>
                ) : (
                    <Fragment key={run.key}>{parse(run.html)}</Fragment>
                ),
            )}
        </>
    );
}

// Quotation text renders in the serif reading face regardless of the
// surrounding article typography. Polytonic Greek has no coverage in
// Libre Baskerville, so it needs Gentium Plus ahead of it — the inline
// style would otherwise beat the global `[lang="grc"]` rule.
const fontFor = (lang: string) =>
    lang === "grc"
        ? "'Gentium Plus', 'Libre Baskerville', serif"
        : "'Libre Baskerville', serif";

export interface QuotationCardProps {
    book: string;
    node: string;
    start: number;
    end?: number;
    /** Sentence-UUID addressing for anchors without a sentence number
     *  (figure captions, headings); wins over start/end. */
    sid?: string;
    sidEnd?: string;
    kind: SentenceKind;
    mode: "source" | "translation" | "source+translation";
    layout:
        | "stacked"
        | "side-by-side-source-left"
        | "side-by-side-source-right";
}

export function QuotationCard({
    book,
    node,
    start,
    end,
    sid,
    sidEnd,
    kind,
    mode,
    layout,
}: QuotationCardProps) {
    const { data, isPending } = useQuery({
        queryKey: ["quotation-card", book, node, start, end, sid, sidEnd, kind],
        queryFn: () =>
            batchSentences({
                items: [
                    sid
                        ? {
                              book_slug: book,
                              node_slug: node,
                              start_id: sid,
                              end_id: sidEnd,
                              kind,
                          }
                        : {
                              book_slug: book,
                              node_slug: node,
                              start_number: start,
                              end_number: end,
                              kind,
                          },
                ],
            }),
        staleTime: 5 * 60 * 1000,
    });

    const item = data?.data?.items?.[0];

    if (isPending) {
        return (
            <Paper
                variant="outlined"
                sx={{ p: 2, my: 2, borderLeft: "3px solid rgb(214 211 209)" }}
            >
                <Skeleton variant="text" width="40%" height={16} />
                <Skeleton
                    variant="text"
                    width="100%"
                    height={20}
                    sx={{ mt: 1 }}
                />
                <Skeleton variant="text" width="80%" height={20} />
            </Paper>
        );
    }

    if (!item || (item.sentences.length === 0 && !item.figure_html)) {
        return (
            <Paper
                variant="outlined"
                sx={{ p: 2, my: 2, borderLeft: "3px solid rgb(239 68 68)" }}
            >
                <p className="text-sm text-red-400 italic">
                    Quotation not found
                </p>
            </Paper>
        );
    }

    // Determine which content to show based on mode
    const showTranslation =
        mode === "translation" || mode === "source+translation";
    const showSource = mode === "source" || mode === "source+translation";

    // Build sentence HTML blocks. A figure quotation renders the block's
    // verbatim <figure> markup (caption included) instead of its anchor
    // sentence — the same markup the reader shows. `not-prose` keeps the
    // article's typography from re-adding list bullets, and the figcaption
    // is pinned bottom-right and muted exactly like the reader's figure box.
    const figureClasses = item.figure_html
        ? " not-prose whitespace-normal [&_figure]:relative [&_figure]:pb-8 [&_figcaption]:absolute [&_figcaption]:right-2 [&_figcaption]:bottom-0 [&_figcaption]:text-right [&_figcaption]:text-sm [&_figcaption]:text-stone-400 [&_ul]:list-none [&_ul]:m-0 [&_ul]:p-0 [&_li]:m-0 [&_li]:p-0"
        : "";
    const translationRuns = speechRuns(item.sentences, "translation");
    const sourceRuns = speechRuns(item.sentences, "source");
    const hasSource =
        item.figure_original_html != null || sourceRuns.length > 0;

    // Reader deep-link key: figures use `fig{N}`, sid-addressed anchors
    // fall back to the sentence UUID (the reader matches ids too), and
    // numbered ranges use `start[-end]`.
    const sentenceKey = sid
        ? item.figure_number != null
            ? `fig${item.figure_number}`
            : sid
        : end && end !== start
          ? `${start}-${end}`
          : String(start);

    const isFootnote = kind === "footnote";
    const prefix = isFootnote ? "fn. s." : "s.";
    const sentenceLabel = sid
        ? item.figure_number != null
            ? `fig. ${item.figure_number}`
            : item.figure_html
              ? "figure"
              : "heading"
        : end && end !== start
          ? `${prefix} ${start}\u2013${end}`
          : `${prefix} ${start}`;

    // Passage locator (after the book title): the book's declared citation
    // systems drive it (e.g. "Romans 13:2", "Paradise Lost \u00b7 Book I \u00b7 42"),
    // falling back to "Node \u00b7 s. N" for books with no default system.
    const passageLocation = formatPassageCitation({
        citation: item.citation,
        parentNodeLabel: item.parent_node_label,
        nodeLabel: item.node_label,
        sentenceLabel,
    });

    const srcBook = item.source ?? {
        book_slug: book,
        book_title: item.book_title,
        node_slug: node,
        node_label: item.node_label,
        language: item.language,
    };
    const sourceAttribution = showSource && (
        <div className="flex justify-end mt-1">
            <Link
                to="/books/$bookSlug/$nodeSlug"
                params={{
                    bookSlug: srcBook.book_slug,
                    nodeSlug: srcBook.node_slug,
                }}
                search={{ s: sentenceKey }}
                target="_blank"
                lang={srcBook.language}
                className="!text-xs !text-stone-400 !no-underline hover:!underline !transition-colors"
            >
                {srcBook.book_title} &middot; {srcBook.node_label} &middot;{" "}
                {sentenceLabel}
            </Link>
        </div>
    );

    const translationAttribution = showTranslation && (
        <div className="flex justify-end mt-1">
            <Link
                to="/books/$bookSlug/$nodeSlug"
                params={{ bookSlug: book, nodeSlug: node }}
                search={{ s: sentenceKey }}
                target="_blank"
                lang={item.language}
                className="!text-xs !text-stone-400 !no-underline hover:!underline !transition-colors"
            >
                {item.book_title} &middot; {passageLocation}
            </Link>
        </div>
    );

    return (
        <Paper
            variant="outlined"
            sx={{
                p: 2,
                my: 2,
                borderLeft: "3px solid rgb(168 162 158)",
                backgroundColor: "#fff",
            }}
        >
            {mode === "source+translation" &&
            layout !== "stacked" &&
            hasSource ? (
                <div
                    className="grid grid-cols-2 gap-4"
                    style={{
                        direction:
                            layout === "side-by-side-source-right"
                                ? "rtl"
                                : "ltr",
                    }}
                >
                    <div className="flex flex-col" style={{ direction: "ltr" }}>
                        <div
                            lang={srcBook.language}
                            className={`text-sm leading-relaxed text-stone-600${figureClasses}`}
                            style={{
                                fontFamily: fontFor(srcBook.language),
                            }}
                        >
                            <PassageBody
                                figureHtml={item.figure_original_html}
                                runs={sourceRuns}
                            />
                        </div>
                        <div className="mt-auto">{sourceAttribution}</div>
                    </div>
                    <div className="flex flex-col" style={{ direction: "ltr" }}>
                        <div
                            lang={item.language}
                            className={`text-sm leading-relaxed text-stone-700${figureClasses}`}
                            style={{
                                fontFamily: fontFor(item.language),
                            }}
                        >
                            <PassageBody
                                figureHtml={item.figure_html}
                                runs={translationRuns}
                            />
                        </div>
                        <div className="mt-auto">{translationAttribution}</div>
                    </div>
                </div>
            ) : (
                <div>
                    {showSource && hasSource && (
                        <>
                            <div
                                lang={srcBook.language}
                                className={`text-sm leading-relaxed ${
                                    mode === "source+translation"
                                        ? "text-stone-600"
                                        : "text-stone-700"
                                }${figureClasses}`}
                                style={{
                                    fontFamily: fontFor(srcBook.language),
                                }}
                            >
                                <PassageBody
                                    figureHtml={item.figure_original_html}
                                    runs={sourceRuns}
                                />
                            </div>
                            {sourceAttribution}
                        </>
                    )}
                    {showTranslation &&
                        mode === "source+translation" &&
                        hasSource && <hr className="my-2 border-stone-200" />}
                    {showTranslation && (
                        <>
                            <div
                                lang={item.language}
                                className={`text-sm leading-relaxed text-stone-700${figureClasses}`}
                                style={{
                                    fontFamily: fontFor(item.language),
                                }}
                            >
                                <PassageBody
                                    figureHtml={item.figure_html}
                                    runs={translationRuns}
                                />
                            </div>
                            {translationAttribution}
                        </>
                    )}
                </div>
            )}
        </Paper>
    );
}

/**
 * Boundary guard for kind strings parsed out of article HTML/markdown
 * attributes — anything unknown renders as a body anchor.
 */
export function asSentenceKind(value: string | null | undefined): SentenceKind {
    return value === "footnote" || value === "figure" ? value : "body";
}
