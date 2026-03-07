// app.config.ts
import { defineConfig } from "@tanstack/react-start/config";
import tailwindcss from "@tailwindcss/vite";
var app_config_default = defineConfig({
  tsr: {
    appDirectory: "src",
    autoCodeSplitting: false
  },
  vite: {
    plugins: [tailwindcss()]
  },
  server: {
    preset: "static",
    prerender: {
      routes: async () => {
        const apiUrl = process.env.API_URL || "http://localhost:4000";
        const routes = ["/", "/books"];
        try {
          const booksRes = await fetch(`${apiUrl}/api/books`);
          const books = await booksRes.json();
          for (const book of books) {
            let collectSlugs2 = function(nodes) {
              for (const node of nodes) {
                if (node.has_content) {
                  routes.push(`/books/${book.slug}/${node.slug}`);
                }
                if (node.children) collectSlugs2(node.children);
              }
            };
            var collectSlugs = collectSlugs2;
            routes.push(`/books/${book.slug}`);
            const tocRes = await fetch(`${apiUrl}/api/books/${book.slug}/toc`);
            const toc = await tocRes.json();
            collectSlugs2(toc);
          }
        } catch (e) {
          console.warn("Could not fetch routes for prerendering:", e);
        }
        return routes;
      },
      crawlLinks: true
    }
  }
});
export {
  app_config_default as default
};
