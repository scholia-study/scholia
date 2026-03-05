import {
  createRootRouteWithContext,
  createRoute,
  Link,
  Outlet,
  useNavigate,
  useSearch,
} from '@tanstack/react-router'
import type { QueryClient } from '@tanstack/react-query'
import { useListBooks } from '../api/books/books'
import { ReaderLayout } from '../components/ReaderLayout'

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

// --- /books route: book selection ---

function BooksPage() {
  const { data, isLoading, error } = useListBooks()
  const books = data?.data
  const navigate = useNavigate()

  const handleOpenBook = (slug: string) => {
    navigate({ to: '/', search: { texts: slug } })
  }

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
              <button
                onClick={() => handleOpenBook(book.slug)}
                className="block w-full text-left p-4 rounded-lg border border-stone-200 bg-white hover:bg-stone-50 transition-colors"
              >
                <div className="font-semibold text-stone-900">{book.title}</div>
                <div className="text-sm text-stone-500">{book.author}</div>
              </button>
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

// --- / route: reader (search-param driven) ---

export type ReaderSearch = {
  texts?: string  // comma-separated "bookSlug/nodeSlug" or "bookSlug"
  s?: string      // comma-separated "panelIndex:sentenceId" entries
}

// Parse "0:abc,1:def" into a Map<number, string>
function parseSelections(s: string | undefined): Map<number, string> {
  const map = new Map<number, string>()
  if (!s) return map
  for (const entry of s.split(',')) {
    const colonIdx = entry.indexOf(':')
    if (colonIdx === -1) continue
    const idx = parseInt(entry.slice(0, colonIdx), 10)
    const id = entry.slice(colonIdx + 1)
    if (!isNaN(idx) && id) map.set(idx, id)
  }
  return map
}

function ReaderPage() {
  const search = useSearch({ from: '/' })
  const navigate = useNavigate({ from: '/' })

  const textsParam = (search as ReaderSearch).texts ?? ''
  const sParam = (search as ReaderSearch).s

  // Parse panels from texts param
  const panels = textsParam
    ? textsParam.split(',').map((entry) => {
        const slashIdx = entry.indexOf('/')
        if (slashIdx === -1) return { bookSlug: entry, nodeSlug: undefined }
        return { bookSlug: entry.slice(0, slashIdx), nodeSlug: entry.slice(slashIdx + 1) || undefined }
      })
    : []

  // Parse per-panel sentence selections
  const selections = parseSelections(sParam)

  // If no panels, show welcome
  if (panels.length === 0) {
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

  const updateSearch = (newSearch: ReaderSearch) => {
    navigate({ search: newSearch })
  }

  return (
    <ReaderLayout
      panels={panels}
      selections={selections}
      onUpdateSearch={updateSearch}
    />
  )
}

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  validateSearch: (search: Record<string, unknown>): ReaderSearch => ({
    texts: search.texts as string | undefined,
    s: search.s as string | undefined,
  }),
  component: ReaderPage,
})

const routeTree = rootRoute.addChildren([
  indexRoute,
  booksRoute,
])

export { routeTree }
