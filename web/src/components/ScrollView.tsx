import { useRef, useState, useEffect, forwardRef, useImperativeHandle } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'
import type { NodeDetail } from '../api/model'
import { Block } from './BlockRenderer'
import { useSentenceSelection } from './SentenceSelectionContext'

export interface ScrollViewHandle {
  scrollToNode: (nodeSlug: string, playOrder?: number) => void
}

interface ScrollViewProps {
  nodes: NodeDetail[]
  hasNextPage: boolean
  isFetchingNextPage: boolean
  fetchNextPage: () => void
  onVisibleNodeChange?: (nodeSlug: string) => void
}

export const ScrollView = forwardRef<ScrollViewHandle, ScrollViewProps>(
  function ScrollView({ nodes, hasNextPage, isFetchingNextPage, fetchNextPage, onVisibleNodeChange }, ref) {
    const parentRef = useRef<HTMLDivElement>(null)
    const { selectedSentenceId, onSelectSentence } = useSentenceSelection()
    const [pendingScrollTarget, setPendingScrollTarget] = useState<{
      nodeSlug: string
      playOrder: number
    } | null>(null)

    const virtualizer = useVirtualizer({
      count: nodes.length,
      getScrollElement: () => parentRef.current,
      estimateSize: () => 400,
      overscan: 3,
    })

    const items = virtualizer.getVirtualItems()

    // Infinite scroll trigger
    useEffect(() => {
      if (!items.length) return
      const lastItem = items[items.length - 1]
      if (lastItem.index >= nodes.length - 5 && hasNextPage && !isFetchingNextPage) {
        fetchNextPage()
      }
    }, [items, nodes.length, hasNextPage, isFetchingNextPage, fetchNextPage])

    // TOC scroll tracking via IntersectionObserver
    useEffect(() => {
      if (!onVisibleNodeChange || !parentRef.current) return

      const observer = new IntersectionObserver(
        (entries) => {
          for (const entry of entries) {
            if (entry.isIntersecting) {
              const nodeSlug = (entry.target as HTMLElement).dataset.nodeSlug
              if (nodeSlug) onVisibleNodeChange(nodeSlug)
            }
          }
        },
        {
          root: parentRef.current,
          rootMargin: '-10% 0px -80% 0px',
        },
      )

      const container = parentRef.current
      const nodeElements = container.querySelectorAll('[data-node-slug]')
      nodeElements.forEach((el) => observer.observe(el))

      return () => observer.disconnect()
    }, [items, onVisibleNodeChange])

    // Scroll-to-node via imperative handle
    useImperativeHandle(ref, () => ({
      scrollToNode(nodeSlug: string, playOrder?: number) {
        const index = nodes.findIndex((n) => n.slug === nodeSlug)
        if (index >= 0) {
          virtualizer.scrollToIndex(index, { align: 'start' })
        } else if (playOrder != null) {
          setPendingScrollTarget({ nodeSlug, playOrder })
        }
      },
    }), [nodes, virtualizer])

    // When nodes update, check if pending target is now loaded
    useEffect(() => {
      if (!pendingScrollTarget) return
      const index = nodes.findIndex((n) => n.slug === pendingScrollTarget.nodeSlug)
      if (index >= 0) {
        setPendingScrollTarget(null)
        virtualizer.scrollToIndex(index, { align: 'start' })
      }
    }, [nodes, pendingScrollTarget, virtualizer])

    // Progressively fetch until pending target is loaded
    useEffect(() => {
      if (!pendingScrollTarget) return
      if (!isFetchingNextPage && hasNextPage) {
        fetchNextPage()
      }
    }, [pendingScrollTarget, isFetchingNextPage, hasNextPage, fetchNextPage])

    return (
      <div ref={parentRef} className="h-full overflow-y-auto">
        <div
          className="relative w-full"
          style={{ height: virtualizer.getTotalSize() }}
        >
          <div
            className="absolute top-0 left-0 w-full"
            style={{ transform: `translateY(${items[0]?.start ?? 0}px)` }}
          >
            {items.map((virtualRow) => {
              const node = nodes[virtualRow.index]
              return (
                <div
                  key={node.id}
                  data-index={virtualRow.index}
                  data-node-slug={node.slug}
                  ref={virtualizer.measureElement}
                  className="max-w-2xl mx-auto px-8"
                >
                  <div className="py-8 border-b border-stone-100">
                    {node.blocks.map((block) => (
                      <Block
                        key={block.id}
                        block={block}
                        selectedSentenceId={selectedSentenceId}
                        onSelectSentence={onSelectSentence}
                      />
                    ))}
                  </div>
                </div>
              )
            })}
          </div>
        </div>
        {isFetchingNextPage && (
          <div className="flex justify-center py-8 text-stone-400">
            <p>Loading more...</p>
          </div>
        )}
      </div>
    )
  },
)
