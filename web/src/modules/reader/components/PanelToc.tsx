import { Link } from "@tanstack/react-router";
import parse from "html-react-parser";
import { useEffect, useMemo, useRef, useState } from "react";
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

interface PanelTocProps {
    toc: TocNodeResponse[];
    bookSlug: string;
    activeNodeSlug: string | undefined;
    onNavigate?: (nodeSlug: string) => void;
}

export function PanelToc({
    toc,
    bookSlug,
    activeNodeSlug,
    onNavigate,
}: PanelTocProps) {
    const prevAncestorsRef = useRef(new Set<string>());
    const expandedAncestors = useMemo(() => {
        const next = activeNodeSlug
            ? findAncestorPath(toc, activeNodeSlug)
            : new Set<string>();
        const prev = prevAncestorsRef.current;
        if (next.size === prev.size) {
            let same = true;
            for (const id of next) {
                if (!prev.has(id)) {
                    same = false;
                    break;
                }
            }
            if (same) return prev;
        }
        prevAncestorsRef.current = next;
        return next;
    }, [toc, activeNodeSlug]);

    return (
        <nav className="p-2 overflow-y-auto flex-1">
            <ul>
                {toc.map((node) => (
                    <TocItem
                        key={node.id}
                        node={node}
                        bookSlug={bookSlug}
                        activeSlug={activeNodeSlug}
                        onNavigate={onNavigate}
                        expandedAncestors={expandedAncestors}
                    />
                ))}
            </ul>
        </nav>
    );
}

function TocItem({
    node,
    bookSlug,
    activeSlug,
    onNavigate,
    expandedAncestors,
}: {
    node: TocNodeResponse;
    bookSlug: string;
    activeSlug: string | undefined;
    onNavigate?: (nodeSlug: string) => void;
    expandedAncestors: Set<string>;
}) {
    const [expanded, setExpanded] = useState(node.depth < 2);
    const hasChildren = node.children.length > 0;
    const isActive = node.slug === activeSlug;

    useEffect(() => {
        if (expandedAncestors.has(node.slug)) {
            setExpanded(true);
        }
    }, [expandedAncestors, node.slug]);

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
                        onClick={() => setExpanded(!expanded)}
                        className="w-4 h-4 mt-0.5 flex items-center justify-center text-stone-400 shrink-0"
                    >
                        {expanded ? "\u25BE" : "\u25B8"}
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
                        onClick={() => hasChildren && setExpanded(!expanded)}
                    >
                        {node.label_html ? parse(node.label_html) : node.label}
                    </span>
                )}
            </div>
            {hasChildren && expanded && (
                <ul>
                    {node.children.map((child) => (
                        <TocItem
                            key={child.id}
                            node={child}
                            bookSlug={bookSlug}
                            activeSlug={activeSlug}
                            onNavigate={onNavigate}
                            expandedAncestors={expandedAncestors}
                        />
                    ))}
                </ul>
            )}
        </li>
    );
}
