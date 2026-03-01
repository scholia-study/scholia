import { useState, useEffect, useMemo, useRef } from 'react'
import { Link, useParams } from '@tanstack/react-router'
import { useGetToc } from '../api/toc/toc'
import type { TocNodeResponse } from '../api/model'

export type ViewMode = 'section' | 'scroll'

function findAncestorPath(
  nodes: TocNodeResponse[],
  targetSlug: string,
): Set<string> {
  const result = new Set<string>()

  function walk(node: TocNodeResponse, path: string[]): boolean {
    if (node.slug === targetSlug) {
      for (const id of path) result.add(id)
      return true
    }
    for (const child of node.children) {
      if (walk(child, [...path, node.slug])) return true
    }
    return false
  }

  for (const node of nodes) {
    if (walk(node, [])) break
  }
  return result
}

function TocItem({
  node,
  slug,
  activeSlug,
  viewMode,
  onScrollToNode,
  expandedAncestors,
}: {
  node: TocNodeResponse
  slug: string
  activeSlug?: string
  viewMode: ViewMode
  onScrollToNode?: (nodeSlug: string, playOrder: number) => void
  expandedAncestors: Set<string>
}) {
  const [expanded, setExpanded] = useState(node.depth < 2)
  const hasChildren = node.children.length > 0
  const isActive = node.slug === activeSlug

  useEffect(() => {
    if (expandedAncestors.has(node.slug)) {
      setExpanded(true)
    }
  }, [expandedAncestors, node.slug])

  const handleClick = () => {
    if (viewMode === 'scroll' && onScrollToNode) {
      onScrollToNode(node.slug, node.play_order)
    }
  }

  return (
    <li>
      <div
        className={`flex items-center gap-1 py-1 pr-2 rounded text-sm ${
          isActive
            ? 'bg-stone-200 font-medium'
            : 'hover:bg-stone-100'
        }`}
        style={{ paddingLeft: `${node.depth * 12 + 4}px` }}
      >
        {hasChildren ? (
          <button
            onClick={() => setExpanded(!expanded)}
            className="w-4 h-4 flex items-center justify-center text-stone-400 shrink-0"
          >
            {expanded ? '\u25BE' : '\u25B8'}
          </button>
        ) : (
          <span className="w-4 shrink-0" />
        )}
        {node.has_content ? (
          viewMode === 'scroll' ? (
            <button
              onClick={handleClick}
              className="flex-1 truncate text-left"
            >
              {node.label}
            </button>
          ) : (
            <Link
              to="/books/$slug/$nodeSlug"
              params={{ slug, nodeSlug: node.slug }}
              className="flex-1 truncate"
            >
              {node.label}
            </Link>
          )
        ) : (
          <span
            className="flex-1 truncate cursor-pointer"
            onClick={() => hasChildren && setExpanded(!expanded)}
          >
            {node.label}
          </span>
        )}
      </div>
      {hasChildren && expanded && (
        <ul>
          {node.children.map((child) => (
            <TocItem
              key={child.id}
              node={child}
              slug={slug}
              activeSlug={activeSlug}
              viewMode={viewMode}
              onScrollToNode={onScrollToNode}
              expandedAncestors={expandedAncestors}
            />
          ))}
        </ul>
      )}
    </li>
  )
}

interface TocSidebarProps {
  slug: string
  viewMode: ViewMode
  onToggleView: () => void
  activeSlugOverride?: string
  onScrollToNode?: (nodeSlug: string, playOrder: number) => void
}

export function TocSidebar({ slug, viewMode, onToggleView, activeSlugOverride, onScrollToNode }: TocSidebarProps) {
  const params = useParams({ strict: false }) as { nodeSlug?: string }
  const { data, isLoading, error } = useGetToc(slug)
  const toc = data?.data

  const activeSlug = viewMode === 'scroll' ? activeSlugOverride : params.nodeSlug

  const prevAncestorsRef = useRef(new Set<string>())
  const expandedAncestors = useMemo(() => {
    const next = toc && activeSlug ? findAncestorPath(toc, activeSlug) : new Set<string>()
    const prev = prevAncestorsRef.current
    if (next.size === prev.size) {
      let same = true
      for (const id of next) {
        if (!prev.has(id)) { same = false; break }
      }
      if (same) return prev
    }
    prevAncestorsRef.current = next
    return next
  }, [toc, activeSlug])

  return (
    <aside className="w-80 border-r border-stone-200 overflow-y-auto bg-white shrink-0">
      <div className="p-4 border-b border-stone-200 flex items-center justify-between">
        <h1 className="text-lg font-semibold text-stone-800">Prospero</h1>
        <button
          onClick={onToggleView}
          className="text-xs px-2 py-1 rounded border border-stone-300 text-stone-600 hover:bg-stone-100 transition-colors"
        >
          {viewMode === 'section' ? 'Scroll' : 'Section'}
        </button>
      </div>
      <nav className="p-2">
        {isLoading ? <p className="text-sm text-stone-400 p-2">Loading...</p> : null}
        {error ? <p className="text-sm text-red-500 p-2">Failed to load TOC</p> : null}
        {toc ? (
          <ul>
            {toc.map((node) => (
              <TocItem
                key={node.id}
                node={node}
                slug={slug}
                activeSlug={activeSlug}
                viewMode={viewMode}
                onScrollToNode={onScrollToNode}
                expandedAncestors={expandedAncestors}
              />
            ))}
          </ul>
        ) : null}
      </nav>
    </aside>
  )
}
