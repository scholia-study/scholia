import { useState, useCallback, useRef } from 'react'
import {
  createRootRouteWithContext,
  createRoute,
  Outlet,
  redirect,
  useNavigate,
} from '@tanstack/react-router'
import type { QueryClient } from '@tanstack/react-query'
import { TocSidebar } from '../components/TocSidebar'
import type { ViewMode } from '../components/TocSidebar'
import { NodeContent } from '../components/NodeContent'
import { ScrollViewContainer } from '../components/ScrollViewContainer'
import type { ScrollViewHandle } from '../components/ScrollViewContainer'

interface RouterContext {
  queryClient: QueryClient
}

const rootRoute = createRootRouteWithContext<RouterContext>()({
  component: () => (
    <div className="min-h-screen bg-stone-50 text-stone-900">
      <Outlet />
    </div>
  ),
})

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  beforeLoad: () => {
    throw redirect({ to: '/books/$slug', params: { slug: 'wissenschaft-der-logik' } })
  },
})

function BookLayout() {
  const { slug } = bookRoute.useParams()
  const navigate = useNavigate()
  const [viewMode, setViewMode] = useState<ViewMode>('section')
  const [visibleNcxId, setVisibleNcxId] = useState<string | undefined>()
  const scrollViewRef = useRef<ScrollViewHandle>(null)

  const handleToggleView = useCallback(() => {
    if (viewMode === 'scroll' && visibleNcxId) {
      navigate({
        to: '/books/$slug/nodes/$ncxId',
        params: { slug, ncxId: visibleNcxId },
      })
    }
    setViewMode((prev) => prev === 'section' ? 'scroll' : 'section')
  }, [navigate, slug, viewMode, visibleNcxId])

  const handleScrollToNode = useCallback((ncxId: string, playOrder: number) => {
    scrollViewRef.current?.scrollToNode(ncxId, playOrder)
  }, [])

  return (
    <div className="flex h-screen">
      <TocSidebar
        slug={slug}
        viewMode={viewMode}
        onToggleView={handleToggleView}
        activeNcxIdOverride={visibleNcxId}
        onScrollToNode={handleScrollToNode}
      />
      <main className="flex-1 overflow-hidden">
        {viewMode === 'scroll' ? (
          <ScrollViewContainer
            ref={scrollViewRef}
            slug={slug}
            onVisibleNodeChange={setVisibleNcxId}
          />
        ) : (
          <div className="h-full overflow-y-auto">
            <Outlet />
          </div>
        )}
      </main>
    </div>
  )
}

const bookRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/books/$slug',
  component: BookLayout,
})

const bookIndexRoute = createRoute({
  getParentRoute: () => bookRoute,
  path: '/',
  component: () => (
    <div className="flex items-center justify-center h-full text-stone-400">
      <p>Select a section from the table of contents.</p>
    </div>
  ),
})

const nodeRoute = createRoute({
  getParentRoute: () => bookRoute,
  path: '/nodes/$ncxId',
  component: () => {
    const { slug, ncxId } = nodeRoute.useParams()
    return <NodeContent slug={slug} ncxId={ncxId} />
  },
})

const routeTree = rootRoute.addChildren([
  indexRoute,
  bookRoute.addChildren([bookIndexRoute, nodeRoute]),
])

export { routeTree }
