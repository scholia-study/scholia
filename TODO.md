## 🐞 BUGS
- saving a range quotation wrongly shows "edit" icon
- Treated as 1 sentence:
  - Thus the criterion of the possibility of a concept (not of its object) is the definition, in which the unity of the concept, the truth of all that may initially be derived from it, and finally the completeness of what has been drawn from it, constitute all that is necessary for the production of the entire concept; or similarly, the criterion of a hypothesis is the intelligibility of the assumed explanatory ground or its unity (without auxiliary hypotheses), the truth (agreement with itself and with experience) of the consequences to be derived from it, and finally the completeness of the explanatory ground in relation to them, which point back to nothing more and nothing less than what was assumed in the hypothesis, and which deliver back a posteriori analytically what had been thought a priori synthetically, and agree with it. — Thus, by the concepts of unity, truth, and perfection, the transcendental table of the categories is not supplemented at all, as if it were perhaps deficient, but rather, by entirely setting aside the relation of these concepts to objects, the procedure with them is brought under general logical rules of the agreement of cognition with itself.
- refine headings

## 🌿 FEATURES
- add "submit for review" for user articles to editors to receive editorial feedback/quality approval
- caching layer on backend API
- rate-limiting on backend API

## 🪡 PATCH
- need filter on tags for quotation in editor
- Verify the production build path end-to-end. pnpm --filter @apps/web build && pnpm --filter @apps/web start should
  produce a Node SSR server. We've never actually run it through the proxy. Worth a smoke test before assuming k3s will
  Just Work — particularly because the start script references ./dist/server/server.js and ../client, and I haven't
  verified those paths match what the current TanStack Start build actually emits.
- API_BASE_URL on the SSR side in production. Right now fetcher.ts does process.env.API_BASE_URL ??
  "http://localhost:4000". For k3s, the Node SSR pod needs API_BASE_URL=http://api:4000 (or whatever the Service is named)
  in its Deployment env. Doesn't affect local at all, but easy to forget when writing manifests.

## 🏗️ INFRA / EXTERNAL SETUP
- need proper resend setup
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
- 🚨 test kant1 pipeline after reorganizing
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