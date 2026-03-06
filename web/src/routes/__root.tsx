import {
  createRootRouteWithContext,
  createRoute,
  Link,
  Outlet,
  useNavigate,
  useParams,
  useSearch,
} from '@tanstack/react-router'
import type { QueryClient } from '@tanstack/react-query'
import { useListBooks } from '../api/books/books'
import { useGetToc } from '../api/toc/toc'
import { PanelToc } from '../components/PanelToc'
import { ReaderLayout } from '../components/ReaderLayout'
import type { PanelState } from '../components/ReaderLayout'

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

// --- /books route: book listing ---

function BooksPage() {
  const { data, isLoading, error } = useListBooks()
  const books = data?.data

  return (
    <div className="max-w-2xl mx-auto px-8 py-16">
      <h1 className="text-3xl font-bold text-stone-900 mb-2">Prospero</h1>
      <p className="text-stone-500 mb-8">Select a book to begin reading.</p>
      {isLoading && <p className="text-stone-400">Loading...</p>}
      {error ? <p className="text-red-500">Failed to load books.</p> : null}
      {books && (
        <ul className="space-y-3">
          {books.map((book) => (
            <li key={book.id}>
              <Link
                to="/books/$bookSlug"
                params={{ bookSlug: book.slug }}
                className="block w-full text-left p-4 rounded-lg border border-stone-200 bg-white hover:bg-stone-50 transition-colors"
              >
                <div className="font-semibold text-stone-900">{book.title}</div>
                <div className="text-sm text-stone-500">{book.author}</div>
              </Link>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}

const booksRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/books',
  component: BooksPage,
})

// --- /books/:bookSlug route: book TOC ---

function BookPage() {
  const { bookSlug } = useParams({ from: '/books/$bookSlug' })
  const navigate = useNavigate()
  const { data: tocData, isLoading, error } = useGetToc(bookSlug)
  const toc = tocData?.data

  const handleNavigate = (nodeSlug: string) => {
    navigate({
      to: '/books/$bookSlug/$nodeSlug',
      params: { bookSlug, nodeSlug },
    })
  }

  return (
    <div className="flex h-screen">
      <div className="max-w-3xl mx-auto px-8 py-16 w-full">
        <Link to="/books" className="text-sm text-stone-500 hover:text-stone-700 mb-4 inline-block">
          &larr; All books
        </Link>
        <h1 className="text-3xl font-bold text-stone-900 mb-8">{bookSlug}</h1>
        {isLoading && <p className="text-stone-400">Loading...</p>}
        {error ? <p className="text-red-500">Failed to load table of contents.</p> : null}
        {toc && (
          <PanelToc
            toc={toc}
            activeNodeSlug={undefined}
            onNavigate={handleNavigate}
          />
        )}
      </div>
    </div>
  )
}

const bookRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/books/$bookSlug',
  component: BookPage,
})

// --- /books/:bookSlug/:nodeSlug route: reader view ---

export type ReaderSearch = {
  p2?: string
  p3?: string
  p4?: string
  s?: string
  s2?: string
  s3?: string
  s4?: string
}

function parsePanel(param: string): PanelState {
  const slashIdx = param.indexOf('/')
  if (slashIdx === -1) return { bookSlug: param, nodeSlug: undefined }
  return {
    bookSlug: param.slice(0, slashIdx),
    nodeSlug: param.slice(slashIdx + 1) || undefined,
  }
}

function ReaderPage() {
  const { bookSlug, nodeSlug } = useParams({ from: '/books/$bookSlug/$nodeSlug' })
  const search = useSearch({ from: '/books/$bookSlug/$nodeSlug' }) as ReaderSearch

  // Build panels array: primary from path, rest from search params
  const panels: PanelState[] = [
    { bookSlug, nodeSlug },
    ...(search.p2 ? [parsePanel(search.p2)] : []),
    ...(search.p3 ? [parsePanel(search.p3)] : []),
    ...(search.p4 ? [parsePanel(search.p4)] : []),
  ]

  // Build selections map
  const selections = new Map<number, string>()
  if (search.s) selections.set(0, search.s)
  if (search.s2) selections.set(1, search.s2)
  if (search.s3) selections.set(2, search.s3)
  if (search.s4) selections.set(3, search.s4)

  return (
    <ReaderLayout
      panels={panels}
      selections={selections}
    />
  )
}

const readerRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/books/$bookSlug/$nodeSlug',
  validateSearch: (search: Record<string, unknown>): ReaderSearch => ({
    p2: search.p2 as string | undefined,
    p3: search.p3 as string | undefined,
    p4: search.p4 as string | undefined,
    s: search.s as string | undefined,
    s2: search.s2 as string | undefined,
    s3: search.s3 as string | undefined,
    s4: search.s4 as string | undefined,
  }),
  component: ReaderPage,
})

// --- / route: redirect to /books ---

function IndexPage() {
  return (
    <div className="flex items-center justify-center h-screen">
      <div className="text-center">
        <h1 className="text-3xl font-bold text-stone-900 mb-2">Prospero</h1>
        <p className="text-stone-500 mb-6">No texts open.</p>
        <Link to="/books" className="px-4 py-2 rounded bg-stone-800 text-white hover:bg-stone-700 transition-colors">
          Browse Books
        </Link>
      </div>
    </div>
  )
}

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: IndexPage,
})

const routeTree = rootRoute.addChildren([
  indexRoute,
  booksRoute,
  bookRoute,
  readerRoute,
])

export { routeTree }
