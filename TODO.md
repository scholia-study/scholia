## BUGS
- unable to save footnote sentence as quotation
- saving a range quotation wrongly shows "edit" icon

## FEATURES
- referencing user-generated content does not add a bibliographical detail (need to set up structure for this)
- add feedback system
- add payment solution (stripe, async stripe crate)
- add "submit for review" for user articles to editors to receive editorial feedback/quality approval

## PATCH
- limit quotations for free tier, add a hard limit for paid tier
- limit notes for free tier, add a hard limit for paid tier
- need filter on tags for quotation in editor
- Extract API base URL to env vars. Three hardcoded http://localhost:4000 references (src/api/fetcher.ts:1, src/routes/login.tsx:123, src/routes/register.tsx:130). This actively breaks any non-local deployment. Move to import.meta.env.VITE_API_BASE_URL and fail loudly if missing.

## INFRA
- need proper resend setup

## MAYBE
- add "commentary, paraphrase, allusion submission to editors for review and approval" 

## NICE-TO-HAVES / FUTURE
- closing the reader entirely should go back to where the user was previous instead of the root page

## DOCS
- update READMEs

## TO TEST
- paid to free tier, check over-quota items are no longer editable
- test input caps
- test editing of sources and persons