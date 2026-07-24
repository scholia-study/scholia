## 🐞 BUGS
- saving a range quotation wrongly shows "edit" icon
- Bible layout comparison broken

## 🌿 FEATURES
- add "submit for review" for user articles to editors to receive editorial feedback/quality approval
- insignias/badges
- caching layer on backend API
- rate-limiting on backend API

## 🪡 PATCH
- need filter on tags for quotation in editor
- Verify the production build path end-to-end. pnpm --filter @apps/web build && pnpm --filter @apps/web start should
  produce a Node SSR server. We've never actually run it through the proxy. Worth a smoke test before assuming k3s will
  Just Work — particularly because the start script references ./dist/server/server.js and ../client, and I haven't
  verified those paths match what the current TanStack Start build actually emits.

## 🏗️ INFRA / EXTERNAL SETUP
- Auth callback       <domain>/api/auth/github/callback
- Stripe webhook      <domain>/api/webhooks/stripe

## 🤔 MAYBE
- add "commentary, paraphrase, allusion submission to editors for review and approval" 
- how-to-cite-this article, ready with .bib formatting and options

## ✨ NICE-TO-HAVES / FUTURE
- closing the reader entirely should go back to where the user was previous instead of the root page

## 📚 DOCS
- update READMEs

## 🧪 TO TEST
- paid to free tier, check over-quota items are no longer editable
- test input caps
- test editing of sources and persons
- limit quotations for free tier, add a hard limit for paid tier
- limit notes for free tier, add a hard limit for paid tier
- referencing user-generated content does not add a bibliographical detail
- test feedback system
- test public user profile page?
- test special tags (e.g. paying users) to display on a public profile
- stripe payment integration (partially locally tested)