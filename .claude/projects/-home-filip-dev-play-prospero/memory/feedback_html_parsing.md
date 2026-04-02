---
name: html-react-parser
description: Use html-react-parser's parse() instead of dangerouslySetInnerHTML for rendering HTML
type: feedback
---

Never use `dangerouslySetInnerHTML`. Always use `parse` from `html-react-parser` to render HTML strings in React components.

**Why:** Project convention for safer HTML rendering.

**How to apply:** `import parse from "html-react-parser"` and use `{parse(htmlString)}` instead of `dangerouslySetInnerHTML={{ __html: htmlString }}`.
