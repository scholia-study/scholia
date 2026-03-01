import { defineConfig } from 'orval'

export default defineConfig({
  prospero: {
    input: {
      target: '../openapi.json',
    },
    output: {
      mode: 'tags-split',
      target: 'src/api',
      schemas: 'src/api/model',
      client: 'react-query',
      override: {
        mutator: {
          path: './src/lib/fetcher.ts',
          name: 'customFetch',
        },
        operations: {
          get_node_page: {
            query: {
              useInfinite: true,
              useInfiniteQueryParam: 'after',
            },
          },
        },
      },
    },
  },
})
