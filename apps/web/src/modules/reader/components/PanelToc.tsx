import { Link } from "@tanstack/react-router";
import parse from "html-react-parser";
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import type { TocNodeResponse } from "../../../api/model";

function findAncestorPath(
    nodes: TocNodeResponse[],
    targetSlug: string,
): Set<string> {
    const result = new Set<string>();

    function walk(node: TocNodeResponse, path: string[]): boolean {
        if (node.slug === targetSlug) {
            for (const id of path) result.add(id);
            return true;
        }
        for (const child of node.children) {
            if (walk(child, [...path, node.slug])) return true;
        }
        return false;
    }

    for (const node of nodes) {
        if (walk(node, [])) break;
    }
    return result;
}

/** Slugs of every node with children — the set "expand all" opens. */
function collectCollapsibleSlugs(nodes: TocNodeResponse[]): string[] {
    const slugs: string[] = [];
    function walk(node: TocNodeResponse) {
        if (node.children.length > 0) slugs.push(node.slug);
        for (const child of node.children) walk(child);
    }
    for (const node of nodes) walk(node);
    return slugs;
}

/** Initial open set: nodes with children in the top two levels. */
function seedExpanded(nodes: TocNodeResponse[]): Set<string> {
    const set = new Set<string>();
    function walk(node: TocNodeResponse) {
        if (node.children.length > 0 && node.depth < 2) set.add(node.slug);
        for (const child of node.children) walk(child);
    }
    for (const node of nodes) walk(node);
    return set;
}

interface PanelTocProps {
    toc: TocNodeResponse[];
    bookSlug: string;
    activeNodeSlug: string | undefined;
    onNavigate?: (nodeSlug: string) => void;
    /**
     * Whether the nav owns its own scroll (sidebar panel). When false
     * (full-page TOC route) the nav grows with the page and the sticky
     * toggle follows the page scroll instead of a trapped inner scroll.
     */
    scrollable?: boolean;
}

export function PanelToc({
    toc,
    bookSlug,
    activeNodeSlug,
    onNavigate,
    scrollable = true,
}: PanelTocProps) {
    // Bible-shape: top-level nodes are bibliographic anchors (source_id
    // set on each — e.g. Genesis, John inside a Bible). Switch to the
    // 2-level pill UI from PLAN_BIG_BOOKS.md Q4. Heuristic stays scoped
    // to the obvious indicator so future compilations behave the same
    // automatically.
    const isBibleShape = toc.length > 0 && toc.every((n) => n.source_id);
    if (isBibleShape) {
        return (
            <BibleShapeToc
                toc={toc}
                bookSlug={bookSlug}
                activeNodeSlug={activeNodeSlug}
                onNavigate={onNavigate}
            />
        );
    }

    return (
        <DefaultToc
            toc={toc}
            bookSlug={bookSlug}
            activeNodeSlug={activeNodeSlug}
            onNavigate={onNavigate}
            scrollable={scrollable}
        />
    );
}

function DefaultToc({
    toc,
    bookSlug,
    activeNodeSlug,
    onNavigate,
    scrollable = true,
}: PanelTocProps) {
    const collapsibleSlugs = useMemo(() => collectCollapsibleSlugs(toc), [toc]);
    const [expanded, setExpanded] = useState(() => seedExpanded(toc));

    // Follow the reader: opening the active node's ancestors (without
    // collapsing anything the user opened manually).
    useEffect(() => {
        if (!activeNodeSlug) return;
        const ancestors = findAncestorPath(toc, activeNodeSlug);
        if (ancestors.size === 0) return;
        setExpanded((prev) => {
            let changed = false;
            const next = new Set(prev);
            for (const slug of ancestors) {
                if (!next.has(slug)) {
                    next.add(slug);
                    changed = true;
                }
            }
            return changed ? next : prev;
        });
    }, [toc, activeNodeSlug]);

    const toggle = (slug: string) =>
        setExpanded((prev) => {
            const next = new Set(prev);
            if (next.has(slug)) next.delete(slug);
            else next.add(slug);
            return next;
        });

    const allExpanded =
        collapsibleSlugs.length > 0 &&
        collapsibleSlugs.every((slug) => expanded.has(slug));

    const toggleAll = () =>
        setExpanded(allExpanded ? new Set() : new Set(collapsibleSlugs));

    return (
        <nav className={`p-2 flex-1 ${scrollable ? "overflow-y-auto" : ""}`}>
            {collapsibleSlugs.length > 1 && (
                <div className="sticky -top-2 z-10 flex justify-end -mx-2 -mt-2 mb-1 px-2 py-2 backdrop-blur-sm">
                    <button
                        type="button"
                        onClick={toggleAll}
                        className="text-xs text-stone-500 hover:text-stone-900 transition-colors"
                    >
                        {allExpanded ? "Collapse all" : "Expand all"}
                    </button>
                </div>
            )}
            <ul>
                {toc.map((node) => (
                    <TocItem
                        key={node.id}
                        node={node}
                        bookSlug={bookSlug}
                        activeSlug={activeNodeSlug}
                        onNavigate={onNavigate}
                        expanded={expanded}
                        onToggle={toggle}
                    />
                ))}
            </ul>
        </nav>
    );
}

/**
 * Two-level pill TOC for Bible-shape books, sidebar variant. Book pills
 * change the *visible* book locally — they do NOT navigate. Only chapter
 * pills navigate. The selected book starts at whichever book contains the
 * active node so the sidebar matches the reader on first paint.
 */
function BibleShapeToc({
    toc,
    bookSlug,
    activeNodeSlug,
    onNavigate,
}: PanelTocProps) {
    const containingBook = useMemo(() => {
        if (!activeNodeSlug) return toc[0];
        for (const book of toc) {
            if (book.slug === activeNodeSlug) return book;
            if (book.children.some((c) => c.slug === activeNodeSlug))
                return book;
        }
        return toc[0];
    }, [toc, activeNodeSlug]);

    const [selectedBookSlug, setSelectedBookSlug] = useState(
        containingBook?.slug,
    );

    // If the user navigates the reader to a different book externally
    // (e.g. clicks a chapter from elsewhere), follow the read position.
    // Keyed on `containingBookSlug` actually changing — *not* on
    // disagreement with `selectedBookSlug` — so when the user clicks a
    // different book pill the effect doesn't immediately yank them back.
    const containingBookSlug = containingBook?.slug;
    const lastSyncedContainingBookRef = useRef(containingBookSlug);
    useEffect(() => {
        if (
            containingBookSlug &&
            containingBookSlug !== lastSyncedContainingBookRef.current
        ) {
            lastSyncedContainingBookRef.current = containingBookSlug;
            setSelectedBookSlug(containingBookSlug);
        }
    }, [containingBookSlug]);

    const visibleBook = useMemo(
        () => toc.find((b) => b.slug === selectedBookSlug) ?? toc[0],
        [toc, selectedBookSlug],
    );

    return (
        <nav className="p-3 overflow-y-auto flex-1 space-y-4">
            <div className="flex flex-wrap gap-1.5">
                {toc.map((book) => (
                    <button
                        type="button"
                        key={book.slug}
                        onClick={() => setSelectedBookSlug(book.slug)}
                        className={`text-xs px-2 py-0.5 rounded border transition-colors ${
                            visibleBook?.slug === book.slug
                                ? "border-stone-800 text-stone-900 bg-stone-100"
                                : "border-stone-300 text-stone-600 hover:border-stone-500 hover:text-stone-900"
                        }`}
                    >
                        {book.label}
                    </button>
                ))}
            </div>
            {visibleBook && visibleBook.children.length > 0 && (
                <div className="flex flex-wrap gap-1">
                    {visibleBook.children.map((child) => (
                        <Link
                            key={child.slug}
                            to="/books/$bookSlug/$nodeSlug"
                            params={{ bookSlug, nodeSlug: child.slug }}
                            onClick={
                                onNavigate
                                    ? (e: React.MouseEvent) => {
                                          e.preventDefault();
                                          onNavigate(child.slug);
                                      }
                                    : undefined
                            }
                            className={`text-xs px-2 py-0.5 rounded border transition-colors ${
                                child.slug === activeNodeSlug
                                    ? "border-stone-800 text-stone-900 bg-stone-100"
                                    : "border-stone-300 text-stone-600 hover:border-stone-500 hover:text-stone-900"
                            }`}
                        >
                            {chapterPillLabel(child.label)}
                        </Link>
                    ))}
                </div>
            )}
        </nav>
    );
}

/**
 * Full TOC page variant for Bible-shape books: every book listed
 * vertically with its chapter pills inline. Book sections carry
 * `id={book.slug}` so URL fragments (`/books/kjv-bible#john`) scroll
 * straight to the book. Used on the book TOC route.
 */
export function BibleShapeFullToc({
    toc,
    bookSlug,
    initialAnchor,
}: {
    toc: TocNodeResponse[];
    bookSlug: string;
    /**
     * Optional URL fragment (without the leading `#`) — when supplied,
     * scrolls that book's section into view on first mount. Drives the
     * "library book pill → /books/<slug>#genesis" shortcut.
     */
    initialAnchor?: string;
}) {
    useLayoutEffect(() => {
        if (!initialAnchor) return;
        const el = document.getElementById(initialAnchor);
        if (el) el.scrollIntoView({ block: "start" });
    }, [initialAnchor]);

    return (
        <div className="space-y-8">
            {toc.map((book) => (
                <section key={book.slug} id={book.slug}>
                    <h2 className="text-xl font-semibold text-stone-900 mb-3">
                        {book.label}
                    </h2>
                    {book.children.length > 0 && (
                        <div className="flex flex-wrap gap-1">
                            {book.children.map((child) => (
                                <Link
                                    key={child.slug}
                                    to="/books/$bookSlug/$nodeSlug"
                                    params={{
                                        bookSlug,
                                        nodeSlug: child.slug,
                                    }}
                                    className="text-xs px-2 py-0.5 rounded border border-stone-300 text-stone-700 hover:border-stone-500 hover:text-stone-900 transition-colors"
                                >
                                    {chapterPillLabel(child.label)}
                                </Link>
                            ))}
                        </div>
                    )}
                </section>
            ))}
        </div>
    );
}

function chapterPillLabel(label: string): string {
    const m = label.match(/^Chapter\s+(\d+)$/i);
    return m ? m[1] : label;
}

function TocItem({
    node,
    bookSlug,
    activeSlug,
    onNavigate,
    expanded,
    onToggle,
}: {
    node: TocNodeResponse;
    bookSlug: string;
    activeSlug: string | undefined;
    onNavigate?: (nodeSlug: string) => void;
    expanded: Set<string>;
    onToggle: (slug: string) => void;
}) {
    const hasChildren = node.children.length > 0;
    const isActive = node.slug === activeSlug;
    const isExpanded = expanded.has(node.slug);

    return (
        <li>
            <div
                className={`flex items-start gap-1 py-1 pr-2 rounded text-sm ${
                    isActive ? "bg-stone-300 font-medium" : "hover:bg-stone-200"
                }`}
                style={{ paddingLeft: `${node.depth * 12 + 4}px` }}
            >
                {hasChildren ? (
                    <button
                        onClick={() => onToggle(node.slug)}
                        className="w-4 h-4 mt-0.5 flex items-center justify-center text-stone-400 shrink-0"
                    >
                        {isExpanded ? "\u25BE" : "\u25B8"}
                    </button>
                ) : (
                    <span className="w-4 mt-0.5 shrink-0" />
                )}
                {node.has_content ? (
                    <Link
                        to="/books/$bookSlug/$nodeSlug"
                        params={{ bookSlug, nodeSlug: node.slug }}
                        onClick={
                            onNavigate
                                ? (e: React.MouseEvent) => {
                                      e.preventDefault();
                                      onNavigate(node.slug);
                                  }
                                : undefined
                        }
                        className="flex-1 text-left"
                    >
                        {node.label_html ? parse(node.label_html) : node.label}
                    </Link>
                ) : (
                    <span
                        className="flex-1 cursor-pointer"
                        onClick={() => hasChildren && onToggle(node.slug)}
                    >
                        {node.label_html ? parse(node.label_html) : node.label}
                    </span>
                )}
            </div>
            {hasChildren && isExpanded && (
                <ul>
                    {node.children.map((child) => (
                        <TocItem
                            key={child.id}
                            node={child}
                            bookSlug={bookSlug}
                            activeSlug={activeSlug}
                            onNavigate={onNavigate}
                            expanded={expanded}
                            onToggle={onToggle}
                        />
                    ))}
                </ul>
            )}
        </li>
    );
}
