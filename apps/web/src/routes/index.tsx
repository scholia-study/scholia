import PlayCircleOutlined from "@mui/icons-material/PlayCircleOutlined";
import { Dialog } from "@mui/material";
import { createFileRoute, Link } from "@tanstack/react-router";
import {
    type CSSProperties,
    type ReactNode,
    useCallback,
    useEffect,
    useMemo,
    useState,
} from "react";
import {
    getGetLibrarySuspenseQueryOptions,
    useGetLibrarySuspense,
} from "../api/books/books";
import type {
    LibraryGroup,
    LibraryStats,
    LibraryVersion,
    LibraryWork,
} from "../api/model";
import { DevServerNotice } from "../components/DevServerNotice";
import { InfoLinks } from "../components/InfoLinks";
import {
    getLocalStorage,
    LOC_STORAGE_KEYS,
    setLocalStorage,
} from "../hooks/local-storage";
import { SEO_COPY, seoHead } from "../modules/seo";
import { libraryHasBook, TOUR_BOOK_SLUG, useReaderTour } from "../modules/tour";

export const Route = createFileRoute("/")({
    loader: ({ context }) => {
        context.queryClient.prefetchQuery(getGetLibrarySuspenseQueryOptions());
    },
    head: () =>
        seoHead({
            title: SEO_COPY.library.title,
            description: SEO_COPY.library.description,
            path: "/",
        }),
    component: IndexPage,
});

function IndexPage() {
    const { data, isLoading } = useGetLibrarySuspense();
    const library = data.data;

    const { startReaderTour, maybeWelcome } = useReaderTour();
    const canTour = !!library && libraryHasBook(library, TOUR_BOOK_SLUG);

    // First-visit welcome prompt (shown once; replayable from the reader after).
    useEffect(() => {
        if (canTour) maybeWelcome();
    }, [canTour, maybeWelcome]);

    const onTakeTour = canTour ? startReaderTour : undefined;

    const groups = useMemo(() => library?.groups ?? [], [library]);
    const bible = useBibleTranslations(groups);
    const [hoveredKey, setHoveredKey] = useState<string | null>(null);

    return (
        <div className="flex-1 bg-white">
            {/* An empty column mirrors the banner's width so the shelf and
                catalogue centre on the viewport rather than on the space left
                over beside the banner. Both side columns appear together at
                1400px — narrower than that the centre column would be squeezed
                hard, and the about text drops to its box under the catalogue
                instead. The centre reaches its full 48rem at about 1520px. */}
            <div className="mx-auto flex w-full max-w-[96rem] items-start gap-8 px-6">
                <div
                    aria-hidden
                    className="hidden w-80 shrink-0 min-[1400px]:block"
                />

                <div className="min-w-0 flex-1 py-10 md:pt-16 md:pb-14">
                    <div className="mx-auto max-w-3xl">
                        <h1 className="sr-only">Library</h1>
                        <DevServerNotice />
                        {!isLoading && groups.length === 0 && (
                            <p className="text-sm text-stone-400">
                                No books in the library yet.
                            </p>
                        )}
                        {!isLoading && groups.length > 0 && (
                            <>
                                <BookShelf
                                    groups={groups}
                                    bibleSlugFor={bible.activeSlug}
                                    hoveredKey={hoveredKey}
                                />
                                <div className="space-y-8">
                                    {groups.map((group) => (
                                        <GroupSection
                                            key={group.id}
                                            group={group}
                                            bible={bible}
                                            onHover={setHoveredKey}
                                        />
                                    ))}
                                </div>
                            </>
                        )}

                        <div className="mt-10 border border-stone-200 bg-stone-100 p-5 min-[1400px]:hidden">
                            <AboutPanel
                                stats={library?.stats}
                                onTakeTour={onTakeTour}
                            />
                        </div>
                    </div>
                </div>

                <aside className="hidden w-80 shrink-0 pt-10 min-[1400px]:block">
                    <AboutBanner
                        stats={library?.stats}
                        onTakeTour={onTakeTour}
                    />
                </aside>
            </div>
        </div>
    );
}

type BibleTranslations = {
    activeSlug: (group: LibraryGroup) => string;
    setActive: (group: LibraryGroup, slug: string) => void;
};

/** Versions live on the (single) work for Bible-shape groups. */
function compilationVersions(group: LibraryGroup): LibraryVersion[] {
    return group.books[0]?.versions ?? [];
}

function defaultTranslationSlug(group: LibraryGroup): string {
    const versions = compilationVersions(group);
    // WEB is the v1 default. The publisher field carries the short label.
    return (
        versions.find((v) => v.publisher === "WEB")?.book_slug ??
        versions[0]?.book_slug ??
        ""
    );
}

/**
 * Persisted translation choice for every compilation-shape group, lifted to
 * the page so the shelf spine and the catalogue row stay in sync. Initial
 * state matches the SSR default to avoid hydration mismatch; the stored
 * value is applied after hydration via useEffect.
 */
function useBibleTranslations(groups: LibraryGroup[]): BibleTranslations {
    const [chosen, setChosen] = useState<Record<string, string>>({});

    useEffect(() => {
        const stored: Record<string, string> = {};
        for (const group of groups) {
            if (group.book_pills.length === 0) continue;
            const slug = getLocalStorage(
                LOC_STORAGE_KEYS.bibleTranslation(group.id),
            );
            if (
                slug &&
                compilationVersions(group).some((v) => v.book_slug === slug)
            ) {
                stored[group.id] = slug;
            }
        }
        setChosen((prev) => ({ ...stored, ...prev }));
    }, [groups]);

    const activeSlug = useCallback(
        (group: LibraryGroup) =>
            chosen[group.id] ?? defaultTranslationSlug(group),
        [chosen],
    );

    const setActive = useCallback((group: LibraryGroup, slug: string) => {
        setChosen((prev) => ({ ...prev, [group.id]: slug }));
        setLocalStorage(LOC_STORAGE_KEYS.bibleTranslation(group.id), slug);
    }, []);

    return { activeSlug, setActive };
}

type HoverHandler = (key: string | null) => void;

/**
 * Spread onto any container of links that stand for one shelf volume, so
 * pointing at them raises it. onFocus/onBlur bubble in React, so tabbing
 * through the links inside does the same.
 */
function raisesSpine(onHover: HoverHandler, key: string) {
    return {
        onMouseEnter: () => onHover(key),
        onMouseLeave: () => onHover(null),
        onFocus: () => onHover(key),
        onBlur: () => onHover(null),
    };
}

type WorkTarget = { bookSlug: string; nodeSlug?: string };

/**
 * The edition a shelf spine stands for. English first: a spine is lettered
 * with the work's English title, so opening the German or Norwegian original
 * would send the reader somewhere the spine never named. Falls back to the
 * original when a work has no English edition.
 */
function spineEdition(work: LibraryWork): LibraryVersion | undefined {
    return (
        work.versions.find((v) => v.language === "en") ??
        work.versions.find((v) => v.is_original) ??
        work.versions[0]
    );
}

function targetOf(version: LibraryVersion | undefined): WorkTarget | null {
    if (!version) return null;
    return {
        bookSlug: version.book_slug,
        nodeSlug: version.node_slug ?? undefined,
    };
}

function stripBooksPrefix(slug: string): string {
    return slug.replace(/^\/books\//, "");
}

type SpineEdition = { label: string; target: WorkTarget | null };

type Spine = {
    key: string;
    title: string;
    author: string;
    year: string;
    /** Each edition the work exists in, so the hover card can link them. */
    editions: SpineEdition[];
    accent: string;
    /** Body-text characters of the edition shown; 0 when unmeasured. */
    length: number;
    height: number;
    target: WorkTarget | null;
};

/**
 * Spine thickness runs on a square-root scale between fixed bounds. Raw
 * character counts span a 42:1 range (the Bible against the Sonnets), so a
 * linear map would leave every philosophical work an indistinguishable
 * sliver beside one enormous Bible; the root compresses that to roughly 6:1
 * and the bounds keep the thinnest volume legible and the thickest from
 * dominating the shelf. Ordering is preserved, so the Bible is still
 * visibly the largest book on the shelf.
 */
const SPINE_MIN_WIDTH = 30;
const SPINE_MAX_WIDTH = 90;

function spineWidths(lengths: number[]): number[] {
    const measured = lengths.filter((n) => n > 0).map(Math.sqrt);
    const mid = (SPINE_MIN_WIDTH + SPINE_MAX_WIDTH) / 2;
    if (measured.length === 0) return lengths.map(() => mid);
    const min = Math.min(...measured);
    const max = Math.max(...measured);
    return lengths.map((raw) => {
        // Unmeasured editions (nested versions) sit mid-shelf rather than
        // claiming to be the slimmest volume.
        if (raw <= 0) return mid;
        if (max === min) return mid;
        const t = (Math.sqrt(raw) - min) / (max - min);
        return Math.round(
            SPINE_MIN_WIDTH + t * (SPINE_MAX_WIDTH - SPINE_MIN_WIDTH),
        );
    });
}

/**
 * A spine is as tall as its lettering needs. Height follows the title's
 * length so a long title is set in full rather than truncated — the reason
 * it can't simply be a hash, which knows nothing about what it has to carry.
 *
 * The per-character figure is deliberately generous: the browser does the
 * real typesetting and we only get to estimate it here, and the widest real
 * title in Libre Baskerville measures 7.85px per character at this size, so
 * 8.6 leaves roughly a tenth in hand.
 *
 * Short titles would leave stubby books, so the id-derived jitter varies the
 * floor rather than the final height: a briefly-titled volume stands
 * somewhere in the min band instead of every one of them landing on the same
 * line, while a long title still takes exactly the height it needs. The
 * ceiling is a true cap — past it a title ellipses rather than growing a
 * comically tall book.
 */
const SPINE_PX_PER_CHAR = 8.6;
/** Head padding above the title plus the clearance kept above the tail rules. */
const SPINE_CHROME = 76;
const SPINE_MIN_HEIGHT = 285;
const SPINE_MAX_HEIGHT = 390;

function spineHeight(id: string, title: string): number {
    let hash = 0;
    for (let i = 0; i < id.length; i++) {
        hash = (hash * 31 + id.charCodeAt(i)) | 0;
    }
    const needed = title.length * SPINE_PX_PER_CHAR + SPINE_CHROME;
    const floor = SPINE_MIN_HEIGHT + (Math.abs(hash) % 6) * 5;
    const clamped = Math.min(SPINE_MAX_HEIGHT, Math.max(floor, needed));
    return Math.round(clamped);
}

function buildSpines(
    groups: LibraryGroup[],
    bibleSlugFor: BibleTranslations["activeSlug"],
): Spine[] {
    const spines: Spine[] = [];
    for (const group of groups) {
        const accent = accentColorFor(group.primary_label, group.id);
        if (group.book_pills.length > 0) {
            // One volume for the whole compilation, opening the reader's
            // chosen translation and sized by that translation's bulk.
            const activeSlug = bibleSlugFor(group);
            const versions = compilationVersions(group);
            const active = versions.find((v) => v.book_slug === activeSlug);
            const slug =
                activeSlug || stripBooksPrefix(group.primary_slug ?? "");
            spines.push({
                key: group.id,
                // The compilation's own name. A Bible-shape work carries the
                // title of one representative translation ("American
                // Standard Version"), which is never what the shelf means.
                title: group.primary_label,
                author: "",
                year: "",
                editions: versions.map((v) => ({
                    label: v.publisher ?? v.language.toUpperCase(),
                    target: targetOf(v),
                })),
                accent,
                length: active?.text_length ?? versions[0]?.text_length ?? 0,
                height: spineHeight(group.id, group.primary_label),
                target: slug ? { bookSlug: slug } : null,
            });
            continue;
        }
        if (group.primary_kind === "self" && group.books.length === 0) {
            const slug = group.primary_slug
                ? stripBooksPrefix(group.primary_slug)
                : "";
            spines.push({
                key: group.id,
                title: group.primary_label,
                author: "",
                year: "",
                editions: [],
                accent,
                length: 0,
                height: spineHeight(group.id, group.primary_label),
                target: slug ? { bookSlug: slug } : null,
            });
            continue;
        }
        for (const work of group.books) {
            // One edition drives both the link and the thickness, so a spine
            // is as thick as the text it actually opens.
            const edition = spineEdition(work);
            const labels = labelVersions(work.versions);
            spines.push({
                key: work.work_id,
                title: work.title,
                author: group.primary_label,
                year: work.publication_year
                    ? String(work.publication_year)
                    : "",
                editions: work.versions.map((v, i) => ({
                    label: labels[i] ?? v.language.toUpperCase(),
                    target: targetOf(v),
                })),
                accent,
                length: edition?.text_length ?? 0,
                height: spineHeight(work.work_id, work.title),
                target: targetOf(edition),
            });
        }
    }
    return spines;
}

function BookShelf({
    groups,
    bibleSlugFor,
    hoveredKey,
}: {
    groups: LibraryGroup[];
    bibleSlugFor: BibleTranslations["activeSlug"];
    /** Work whose catalogue row is hovered; its volume rises off the shelf. */
    hoveredKey: string | null;
}) {
    const spines = useMemo(
        () => buildSpines(groups, bibleSlugFor),
        [groups, bibleSlugFor],
    );
    const widths = useMemo(
        () => spineWidths(spines.map((s) => s.length)),
        [spines],
    );
    if (spines.length === 0) return null;
    return (
        <div className="hidden md:block mb-10">
            <div className="flex items-end justify-center gap-1 pt-20">
                {spines.map((spine, i) => (
                    <SpineLink
                        key={spine.key}
                        spine={spine}
                        width={widths[i] ?? SPINE_MIN_WIDTH}
                        raised={spine.key === hoveredKey}
                    />
                ))}
            </div>
            <div
                className="h-3 rounded-sm shadow-[0_6px_10px_-6px_rgba(58,32,12,0.55)]"
                style={{ backgroundImage: OAK_BOARD }}
            />
        </div>
    );
}

/**
 * Quarter-sawn oak, built from three stacked gradients: fine grain running
 * the length of the timber, a broader figure varying along it so the run
 * never repeats visibly, and last the piece's own modelling — a lit upper
 * edge falling to a shaded underside for the shelf, and a highlight band a
 * third of the way down the rod to round it into a dowel.
 */
const OAK_GRAIN =
    "repeating-linear-gradient(0deg, rgba(58,32,12,0.20) 0 1px, rgba(0,0,0,0) 1px 4px)";
const OAK_FIGURE =
    "repeating-linear-gradient(90deg, rgba(255,229,193,0.10) 0 3px, rgba(0,0,0,0) 3px 29px)";

const OAK_BOARD = [
    OAK_GRAIN,
    OAK_FIGURE,
    "linear-gradient(180deg, #b07c42 0%, #8f5d2c 42%, #6d431e 100%)",
].join(",");

const OAK_ROD = [
    OAK_GRAIN,
    OAK_FIGURE,
    "linear-gradient(180deg, #8a5a2b 0%, #c39152 32%, #966232 68%, #5f3a1a 100%)",
].join(",");

const SPINE_SHEEN =
    "linear-gradient(to right, rgba(0,0,0,0.18), rgba(255,255,255,0.14) 18%, rgba(0,0,0,0.05) 55%, rgba(0,0,0,0.26))";

function SpineLink({
    spine,
    width,
    raised,
}: {
    spine: Spine;
    width: number;
    raised: boolean;
}) {
    const style = {
        width,
        height: spine.height,
        backgroundColor: spine.accent,
        backgroundImage: SPINE_SHEEN,
        boxShadow: "inset 0 -6px 8px -6px rgba(0,0,0,0.35)",
    };
    // The card is a sibling of the spine, not a child: it carries its own
    // links to each edition, and an anchor cannot be nested inside one.
    //
    // `raised` mirrors the pointer's own hover lift, driven from the catalogue
    // below. The lift is a transform, which makes this element a stacking
    // context and traps the card's z-index inside it — a taller neighbour
    // later in the row would then paint over the card — so the whole spine
    // rises above its siblings while it is up.
    const wrapper = `group relative transition-transform duration-150 motion-reduce:transition-none hover:z-20 hover:-translate-y-2.5 focus-within:z-20 focus-within:-translate-y-2.5 ${
        raised ? "z-20 -translate-y-2.5" : ""
    }`;
    return (
        <div className={wrapper}>
            <TargetLink
                target={spine.target}
                className="relative flex justify-center rounded-t-[4px] pt-11"
                style={style}
            >
                <SpineFace spine={spine} />
            </TargetLink>
            <SpineCard spine={spine} />
        </div>
    );
}

/**
 * Binder's tooling rather than photographic detail: a gilt double rule at head
 * and tail, and darkened caps where the covers turn over the boards. All of it
 * sits clear of the title panel, so the lettering keeps a flat field to sit on
 * and stays readable at this size.
 */
function SpineFace({ spine }: { spine: Spine }) {
    const rule = "pointer-events-none absolute inset-x-1 h-px";
    return (
        <>
            <span aria-hidden className={`${rule} top-6 bg-amber-100/55`} />
            <span
                aria-hidden
                className={`${rule} top-[1.85rem] bg-amber-100/30`}
            />
            <span
                aria-hidden
                className={`${rule} bottom-[1.85rem] bg-amber-100/30`}
            />
            <span aria-hidden className={`${rule} bottom-6 bg-amber-100/55`} />
            <span
                aria-hidden
                className="pointer-events-none absolute inset-x-0 top-0 h-2.5 rounded-t-[4px] bg-black/20"
            />
            <span
                aria-hidden
                className="pointer-events-none absolute inset-x-0 bottom-0 h-2.5 bg-black/25"
            />
            <span className="[writing-mode:vertical-rl] max-h-[calc(100%-2rem)] overflow-hidden text-ellipsis whitespace-nowrap text-[0.78rem] tracking-[0.06em] text-white/90">
                {spine.title}
            </span>
        </>
    );
}

/**
 * The hover card. Its outer box sits flush against the top of the spine and
 * holds the visual gap as padding, so the pointer crosses into the card
 * without ever leaving the group and dismissing it on the way. It stays
 * click-through until shown, or ten invisible cards would swallow hover along
 * the whole shelf.
 */
function SpineCard({ spine }: { spine: Spine }) {
    return (
        <div className="pointer-events-none absolute bottom-full left-1/2 z-10 -translate-x-1/2 pb-3.5 opacity-0 transition-opacity duration-150 group-hover:pointer-events-auto group-hover:opacity-100 group-focus-within:pointer-events-auto group-focus-within:opacity-100">
            <div
                className="whitespace-nowrap border border-stone-200 bg-white px-3.5 py-2.5 text-center shadow-lg"
                style={{ borderTop: `2px solid ${spine.accent}` }}
            >
                {spine.author && (
                    <span className="block text-[0.58rem] font-semibold uppercase tracking-[0.12em] text-stone-600">
                        {spine.author}
                    </span>
                )}
                <span className="block text-[0.82rem] text-stone-900">
                    {spine.title}
                </span>
                {spine.year && (
                    <span className="block text-[0.65rem] text-stone-400">
                        {spine.year}
                    </span>
                )}
                {spine.editions.length > 0 && (
                    <span className="mt-1.5 flex justify-center gap-1">
                        {spine.editions.map((e) => (
                            <TargetLink
                                key={e.label}
                                target={e.target}
                                className="rounded border border-stone-300 px-1.5 py-0.5 text-[0.6rem] text-stone-600 transition-colors hover:border-stone-500 hover:text-stone-900"
                            >
                                {e.label}
                            </TargetLink>
                        ))}
                    </span>
                )}
            </div>
        </div>
    );
}

/** Reader link for a resolved target, falling back to plain text. */
function TargetLink({
    target,
    className,
    style,
    children,
}: {
    target: WorkTarget | null;
    className?: string;
    style?: CSSProperties;
    children: ReactNode;
}) {
    if (!target) {
        return (
            <span className={className} style={style}>
                {children}
            </span>
        );
    }
    if (target.nodeSlug) {
        // Shape-3 nested anchor: deep-link into the host book at the
        // toc node slug.
        return (
            <a
                href={`/books/${target.bookSlug}/${target.nodeSlug}`}
                className={className}
                style={style}
            >
                {children}
            </a>
        );
    }
    return (
        <Link
            to="/books/$bookSlug"
            params={{ bookSlug: target.bookSlug }}
            className={className}
            style={style}
        >
            {children}
        </Link>
    );
}

/**
 * Renders one group as a catalogue entry: the author (or compilation) name
 * as a small-caps sidehead in the left margin, works as single rows beside
 * it. Singleton authorless works link straight from the sidehead.
 */
function GroupSection({
    group,
    bible,
    onHover,
}: {
    group: LibraryGroup;
    bible: BibleTranslations;
    onHover: (key: string | null) => void;
}) {
    const accent = accentColorFor(group.primary_label, group.id);
    const isSelf = group.primary_kind === "self";
    const isSingleton = isSelf && group.books.length === 0;
    // Bible-shape: one compilation work in many translations. Book links
    // become the primary navigation, translation is a persisted chooser.
    if (group.book_pills.length > 0) {
        return (
            <CompilationShapeGroup
                group={group}
                accent={accent}
                bible={bible}
                onHover={onHover}
            />
        );
    }

    return (
        <section className="grid gap-x-8 gap-y-2 md:grid-cols-[11.5rem_minmax(0,1fr)]">
            <Sidehead accent={accent}>
                {isSingleton && group.primary_slug ? (
                    <Link
                        to="/books/$bookSlug"
                        params={{
                            bookSlug: stripBooksPrefix(group.primary_slug),
                        }}
                        className="hover:underline"
                    >
                        {group.primary_label}
                    </Link>
                ) : (
                    group.primary_label
                )}
            </Sidehead>
            {!isSingleton && (
                <ul>
                    {group.books.map((work) => (
                        <WorkRow
                            key={work.work_id}
                            work={work}
                            hideTitle={
                                isSelf && work.title === group.primary_label
                            }
                            onHover={onHover}
                        />
                    ))}
                </ul>
            )}
        </section>
    );
}

function Sidehead({
    accent,
    children,
}: {
    accent: string;
    children: ReactNode;
}) {
    return (
        <h2 className="pt-1 text-[0.66rem] font-semibold uppercase tracking-[0.14em] text-stone-600 md:text-right">
            {children}
            <span
                className="mt-1.5 block h-0.5 w-9 rounded-full md:ml-auto"
                style={{ backgroundColor: accent }}
            />
        </h2>
    );
}

function WorkRow({
    work,
    hideTitle = false,
    onHover,
}: {
    work: LibraryWork;
    /**
     * Suppress title and metadata. Used for SelfNamed groups where the
     * sidehead already shows the work's title. The version pills are then
     * the only useful row.
     */
    hideTitle?: boolean;
    onHover: (key: string | null) => void;
}) {
    const versionLabels = useMemo(
        () => labelVersions(work.versions),
        [work.versions],
    );
    // Credits stay beside the title; only the year travels to the end of
    // the row, so the years form an aligned column down the right edge
    // however wide a work's edition pills happen to be.
    const credits = [
        work.co_authors.length > 0 ? `with ${work.co_authors.join(", ")}` : "",
        work.editor_names && work.editor_names.length > 0
            ? `edited by ${work.editor_names.join(", ")}`
            : "",
    ]
        .filter(Boolean)
        .join(" · ");

    return (
        <li className="flex flex-wrap items-baseline gap-x-3 gap-y-1 py-1">
            {!hideTitle && (
                <>
                    {/* Plain text, not a link: the title is the work's English
                        name, while the editions it could open are the pills
                        beside it — one of which may be the German or
                        Norwegian original. Let the pills say where they go. */}
                    <span className="font-serif text-base font-medium text-stone-900">
                        {work.title}
                    </span>
                    {credits && (
                        <span className="text-xs text-stone-400">
                            {credits}
                        </span>
                    )}
                    <span className="hidden min-w-8 flex-1 -translate-y-1 border-b border-dotted border-stone-300 md:block" />
                </>
            )}
            <span
                className="flex flex-wrap gap-1.5"
                {...raisesSpine(onHover, work.work_id)}
            >
                {work.versions.map((v, i) => (
                    <VersionPill
                        key={`${v.book_slug}::${v.node_slug ?? ""}`}
                        version={v}
                        label={versionLabels[i] ?? v.language.toUpperCase()}
                    />
                ))}
            </span>
            {!hideTitle && (
                // Fixed width, not tabular figures: Libre Baskerville ships no
                // `tnum` feature, so its digits keep their drawn widths (a "0"
                // is half again as wide as a "1") and a bare year box would
                // change width per year, nudging the pills beside it. Reserving
                // the width of the widest four-digit year holds the pills still.
                <span className="min-w-9 text-right text-xs text-stone-400">
                    {work.publication_year ?? "Undated"}
                </span>
            )}
        </li>
    );
}

function CompilationShapeGroup({
    group,
    accent,
    bible,
    onHover,
}: {
    group: LibraryGroup;
    accent: string;
    bible: BibleTranslations;
    onHover: (key: string | null) => void;
}) {
    const versions = compilationVersions(group);
    const activeSlug = bible.activeSlug(group);
    const activeVersion = versions.find((v) => v.book_slug === activeSlug);
    const activeLabel =
        activeVersion?.publisher ?? activeVersion?.language.toUpperCase() ?? "";
    // Single-edition compilations have no translation versions, so
    // activeSlug is empty; fall back to the group's own book.
    const pillBookSlug =
        activeSlug || stripBooksPrefix(group.primary_slug ?? "");

    return (
        <section className="grid gap-x-8 gap-y-2 md:grid-cols-[11.5rem_minmax(0,1fr)]">
            <Sidehead accent={accent}>{group.primary_label}</Sidehead>
            <div>
                {/* No title row: the sidehead already names the compilation,
                    and the work's own title is one representative
                    translation's ("American Standard Version"). */}
                <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1 py-1">
                    {/* The compilation's spine is keyed by group id, not by a
                        work id — see buildSpines. */}
                    <span
                        className="flex flex-wrap gap-1.5"
                        {...raisesSpine(onHover, group.id)}
                    >
                        {versions.map((v) => {
                            const isActive = v.book_slug === activeSlug;
                            const label =
                                v.publisher ?? v.language.toUpperCase();
                            const yearSuffix = v.publication_year
                                ? ` (${v.publication_year})`
                                : "";
                            const tooltip = isActive
                                ? `Currently reading: ${label}${yearSuffix}`
                                : `Click to set the Bible version to ${label}${yearSuffix}`;
                            return (
                                <button
                                    type="button"
                                    key={v.book_slug}
                                    onClick={() =>
                                        bible.setActive(group, v.book_slug)
                                    }
                                    title={tooltip}
                                    className={`cursor-pointer rounded border px-2 py-0.5 text-xs transition-colors ${
                                        isActive
                                            ? "border-stone-800 text-stone-900"
                                            : "border-stone-300 text-stone-600 hover:border-stone-500 hover:text-stone-900"
                                    }`}
                                >
                                    {label}
                                </button>
                            );
                        })}
                    </span>
                </div>
                <details className="group/books mt-1">
                    <summary className="cursor-pointer select-none list-none text-xs text-stone-400 hover:text-stone-700 [&::-webkit-details-marker]:hidden">
                        <span className="group-open/books:hidden">▸</span>
                        <span className="hidden group-open/books:inline">
                            ▾
                        </span>{" "}
                        Browse the {group.book_pills.length} books
                    </summary>
                    <div
                        className="mt-2 flex flex-wrap gap-1.5"
                        {...raisesSpine(onHover, group.id)}
                    >
                        {group.book_pills.map((p) => (
                            <Link
                                key={p.node_slug}
                                to="/books/$bookSlug"
                                params={{ bookSlug: pillBookSlug }}
                                hash={p.node_slug}
                                title={
                                    activeLabel
                                        ? `Open ${p.label} (${activeLabel})`
                                        : `Open ${p.label}`
                                }
                                className="cursor-pointer rounded border border-stone-300 px-2 py-0.5 text-xs text-stone-700 transition-colors hover:border-stone-500 hover:text-stone-900"
                            >
                                {p.label}
                            </Link>
                        ))}
                    </div>
                </details>
            </div>
        </section>
    );
}

const ACCENT_PALETTE = [
    "#b45309", // amber-700
    "#047857", // emerald-700
    "#1d4ed8", // blue-700
    "#b91c1c", // red-700
    "#6d28d9", // violet-700
    "#0369a1", // sky-700
    "#a16207", // yellow-700
    "#be185d", // pink-700
];

/** Manual per-group accent overrides, keyed by exact display label. */
const ACCENT_OVERRIDES: Record<string, string> = {
    "Immanuel Kant": "#4169e1", // royal blue
    "The Bible": "#4b0082", // royal purple (indigo)
};

function accentColorFor(label: string, id: string): string {
    const override = ACCENT_OVERRIDES[label];
    if (override) return override;
    let hash = 0;
    for (let i = 0; i < id.length; i++) {
        hash = (hash * 31 + id.charCodeAt(i)) | 0;
    }
    return ACCENT_PALETTE[Math.abs(hash) % ACCENT_PALETTE.length];
}

function VersionPill({
    version,
    label,
}: {
    version: LibraryVersion;
    label: string;
}) {
    const className = `text-xs px-2 py-0.5 rounded border transition-colors ${
        version.is_original
            ? "border-stone-800 text-stone-900 hover:bg-stone-900 hover:text-white"
            : "border-stone-300 text-stone-600 hover:border-stone-500 hover:text-stone-900"
    }`;
    if (version.node_slug) {
        // Shape-3 nested anchor: deep-link into the host book at the
        // toc node slug.
        return (
            <a
                href={`/books/${version.book_slug}/${version.node_slug}`}
                className={className}
            >
                {label}
            </a>
        );
    }
    return (
        <Link
            to="/books/$bookSlug"
            params={{ bookSlug: version.book_slug }}
            className={className}
        >
            {label}
        </Link>
    );
}

function AboutPanel({
    stats,
    onTakeTour,
}: {
    stats: LibraryStats | undefined;
    onTakeTour?: () => void;
}) {
    const pClasses = "text-sm text-stone-600 leading-relaxed";
    return (
        <>
            <h2 className="text-base font-semibold text-stone-600 mb-2 uppercase">
                A living library for scholars
            </h2>
            <p className={pClasses}>
                Scholia is a hermeneutical workspace designed for the deep study
                of literary, philosophical, and sacred texts. Every work is
                structured down to the sentence, linked across translations, and
                open for quotation, notes, and citation in your own writing.
            </p>
            <br />
            <p className={pClasses}>
                Inspired by marginal notes that ancient and medieval scholars
                wrote alongside classical texts, Scholia aims to be a digital
                sanctuary for careful study, developing original insights, and
                building collaborative commentary.
            </p>
            <IntroVideoButton />
            {stats && (
                <p className="text-xs text-stone-400 mt-4 pt-4 border-t border-stone-200">
                    {formatStats(stats)}
                </p>
            )}
            <InfoLinks
                className="text-sm mt-6 flex flex-wrap gap-x-4 gap-y-1 text-stone-500"
                trailing={
                    onTakeTour ? (
                        <button
                            type="button"
                            onClick={onTakeTour}
                            className="cursor-pointer hover:underline"
                        >
                            Take a tour
                        </button>
                    ) : undefined
                }
            />
        </>
    );
}

/**
 * The about text as a banner hung from a rod: a rod with capped ends, then
 * cloth falling from it, swallow-tailed at the foot. The vertical wash plus
 * the darker margins read as a curved surface catching light down the
 * middle, and the whole thing is pale enough to leave the shelf as the one
 * loud element on the page.
 */
const BANNER_CLOTH =
    "polygon(0 0, 100% 0, 100% 100%, 50% calc(100% - 1.75rem), 0 100%)";

function AboutBanner({
    stats,
    onTakeTour,
}: {
    stats: LibraryStats | undefined;
    onTakeTour?: () => void;
}) {
    return (
        <div className="sticky top-8">
            <div
                className="relative -mx-2 h-2.5 rounded-full shadow-[0_2px_4px_-1px_rgba(58,32,12,0.45)]"
                style={{ backgroundImage: OAK_ROD }}
            >
                <span
                    className="absolute -left-1.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 rounded-full shadow-[0_1px_2px_rgba(58,32,12,0.4)]"
                    style={{ backgroundImage: OAK_ROD }}
                />
                <span
                    className="absolute -right-1.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 rounded-full shadow-[0_1px_2px_rgba(58,32,12,0.4)]"
                    style={{ backgroundImage: OAK_ROD }}
                />
            </div>
            <div
                className="relative px-6 pt-6 pb-16 shadow-[0_10px_24px_-16px_rgba(28,25,23,0.45)]"
                style={{
                    clipPath: BANNER_CLOTH,
                    backgroundImage: [
                        "linear-gradient(90deg, rgba(28,25,23,0.07), rgba(28,25,23,0) 14%, rgba(28,25,23,0) 86%, rgba(28,25,23,0.07))",
                        "linear-gradient(180deg, #faf9f7, #f1eee9)",
                    ].join(","),
                }}
            >
                <AboutPanel stats={stats} onTakeTour={onTakeTour} />
            </div>
        </div>
    );
}

/**
 * "Watch the introduction" button + YouTube lightbox. The iframe only
 * exists while the dialog is open (MUI unmounts closed dialogs), so
 * nothing loads from YouTube before the click and playback stops on
 * close. nocookie host keeps the pre-consent surface clean.
 */
function IntroVideoButton() {
    const [open, setOpen] = useState(false);
    return (
        <>
            <button
                type="button"
                onClick={() => setOpen(true)}
                className="mt-4 mx-auto flex items-center gap-1.5 text-sm px-3 py-1.5 border border-stone-300 rounded bg-white text-stone-700 hover:bg-stone-50 hover:border-stone-400 transition-colors cursor-pointer"
            >
                <PlayCircleOutlined
                    sx={{ fontSize: 18 }}
                    className="text-amber-700"
                />
                Watch the introduction
            </button>
            <Dialog
                open={open}
                onClose={() => setOpen(false)}
                maxWidth="md"
                fullWidth
                slotProps={{ paper: { sx: { bgcolor: "black" } } }}
            >
                <div className="relative aspect-video">
                    <iframe
                        src="https://www.youtube-nocookie.com/embed/81kjGzeWAs8?autoplay=1"
                        title="Introduction to Scholia"
                        allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
                        allowFullScreen
                        referrerPolicy="strict-origin-when-cross-origin"
                        className="absolute inset-0 h-full w-full border-0"
                    />
                </div>
            </Dialog>
        </>
    );
}

function plural(n: number, singular: string, pluralForm: string) {
    return `${n} ${n === 1 ? singular : pluralForm}`;
}

function formatStats(stats: LibraryStats) {
    return [
        plural(stats.works, "work", "works"),
        plural(stats.languages, "language", "languages"),
    ].join(" · ");
}

/**
 * Generate the pill label for each version in a work.
 * Same-language versions get disambiguated by translator → publisher → year.
 */
function labelVersions(versions: LibraryVersion[]): string[] {
    const counts = new Map<string, number>();
    for (const v of versions) {
        counts.set(v.language, (counts.get(v.language) ?? 0) + 1);
    }
    // When every version shares the language, the language code carries no
    // information; the publisher / translator alone is the useful pill.
    // (e.g. KJV vs WEB — both English, "EN · KJV" would be noise.)
    const allSameLanguage = counts.size === 1;
    return versions.map((v) => {
        const code = v.language.toUpperCase();
        const ambiguous = (counts.get(v.language) ?? 0) > 1;
        if (!ambiguous) return code;
        const prefix = allSameLanguage ? "" : `${code} · `;
        if (v.translator_names.length > 0) {
            return `${prefix}${v.translator_names.map(lastName).join(" & ")}`;
        }
        if (v.publisher) return `${prefix}${v.publisher}`;
        if (v.publication_year) return `${prefix}${v.publication_year}`;
        return code;
    });
}

function lastName(fullName: string): string {
    const parts = fullName.trim().split(/\s+/);
    return parts[parts.length - 1] ?? fullName;
}
