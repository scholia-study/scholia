import {
  createRootRouteWithContext,
  Outlet,
  ScrollRestoration,
} from '@tanstack/react-router'
import { Meta, Scripts } from '@tanstack/react-start'
import type { QueryClient } from '@tanstack/react-query'
import '../index.css'

interface RouterContext {
  queryClient: QueryClient
}

export const Route = createRootRouteWithContext<RouterContext>()({
  head: () => ({
    meta: [
      { charSet: 'utf-8' },
      { name: 'viewport', content: 'width=device-width, initial-scale=1' },
      { title: 'Prospero' },
    ],
    links: [
      { rel: 'icon', type: 'image/svg+xml', href: '/vite.svg' },
    ],
  }),
  component: RootComponent,
})

function RootComponent() {
  return (
    <html lang="en">
      <head>
        <Meta />
      </head>
      <body>
        <div className="min-h-screen bg-stone-50 text-stone-900">
          <Outlet />
        </div>
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  )
}
