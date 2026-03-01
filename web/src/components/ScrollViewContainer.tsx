import { useMemo, forwardRef } from 'react'
import { useGetNodePageInfinite } from '../api/nodes/nodes'
import { ScrollView } from './ScrollView'
import type { ScrollViewHandle } from './ScrollView'

interface ScrollViewContainerProps {
  slug: string
  onVisibleNodeChange?: (nodeSlug: string) => void
}

export const ScrollViewContainer = forwardRef<ScrollViewHandle, ScrollViewContainerProps>(
  function ScrollViewContainer({ slug, onVisibleNodeChange }, ref) {
    const {
      data,
      hasNextPage,
      isFetchingNextPage,
      fetchNextPage,
      isLoading,
      error,
    } = useGetNodePageInfinite(slug, { limit: 20 }, {
      query: {
        initialPageParam: undefined,
        getNextPageParam: (lastPage) => {
          const page = lastPage.data
          if (!page.has_more || page.nodes.length === 0) return undefined
          return page.nodes[page.nodes.length - 1].play_order
        },
      },
    })

    const nodes = useMemo(
      () => data?.pages.flatMap((page) => page.data.nodes) ?? [],
      [data],
    )

    if (isLoading) {
      return (
        <div className="flex items-center justify-center h-full text-stone-400">
          <p>Loading...</p>
        </div>
      )
    }

    if (error) {
      return (
        <div className="flex items-center justify-center h-full text-red-500">
          <p>Failed to load content.</p>
        </div>
      )
    }

    return (
      <ScrollView
        ref={ref}
        nodes={nodes}
        hasNextPage={hasNextPage ?? false}
        isFetchingNextPage={isFetchingNextPage}
        fetchNextPage={fetchNextPage}
        onVisibleNodeChange={onVisibleNodeChange}
      />
    )
  },
)

export type { ScrollViewHandle }
