```ts
import { defineConfig } from "orval";

export default defineConfig({
  petstore: {
    output: {
      client: "react-query",
      override: {
        operations: {
          listPets: {
            query: {
              useInfinite: true,
              useInfiniteQueryParam: "cursor",
            },
          },
        },
      },
    },
    input: {
      target: "./petstore.yaml",
    },
  },
});
```
