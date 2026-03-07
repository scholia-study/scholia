import { createFileRoute, Link } from '@tanstack/react-router'

export const Route = createFileRoute('/')({
  component: IndexPage,
})

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
