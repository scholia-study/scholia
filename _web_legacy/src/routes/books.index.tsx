import { createFileRoute, Link } from '@tanstack/react-router'
import { useListBooks, getListBooksQueryOptions } from '../api/books/books'

export const Route = createFileRoute('/books/')({
  loader: async ({ context }) => {
    await context.queryClient.ensureQueryData(getListBooksQueryOptions())
  },
  component: BooksPage,
})

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
