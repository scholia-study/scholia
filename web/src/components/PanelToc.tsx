import { useState, useEffect, useMemo, useRef } from 'react'
import type { TocNodeResponse } from '../api/model'

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

interface PanelTocProps {
  toc: TocNodeResponse[]
  activeNodeSlug: string | undefined
  onNavigate: (nodeSlug: string) => void
}

export function PanelToc({ toc, activeNodeSlug, onNavigate }: PanelTocProps) {
  const prevAncestorsRef = useRef(new Set<string>())
  const expandedAncestors = useMemo(() => {
    const next = activeNodeSlug ? findAncestorPath(toc, activeNodeSlug) : new Set<string>()
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
  }, [toc, activeNodeSlug])

  return (
    <aside className="w-64 border-r border-stone-200 overflow-y-auto bg-white shrink-0">
      <nav className="p-2">
        <ul>
          {toc.map((node) => (
            <TocItem
              key={node.id}
              node={node}
              activeSlug={activeNodeSlug}
              onNavigate={onNavigate}
              expandedAncestors={expandedAncestors}
            />
          ))}
        </ul>
      </nav>
    </aside>
  )
}

function TocItem({
  node,
  activeSlug,
  onNavigate,
  expandedAncestors,
}: {
  node: TocNodeResponse
  activeSlug: string | undefined
  onNavigate: (nodeSlug: string) => void
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
          <button
            onClick={() => onNavigate(node.slug)}
            className="flex-1 truncate text-left"
          >
            {node.label}
          </button>
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
              activeSlug={activeSlug}
              onNavigate={onNavigate}
              expandedAncestors={expandedAncestors}
            />
          ))}
        </ul>
      )}
    </li>
  )
}
