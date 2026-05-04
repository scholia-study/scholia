 All clean: types pass, biome passes.

  Frontend done

  What's built

  - web/src/modules/billing/:
    - stripe.ts — singleton getStripe() lazy-loading the publishable key
    - EmbeddedCheckoutModal.tsx — MUI <Dialog> wrapping Stripe's <EmbeddedCheckoutProvider> + <EmbeddedCheckout>
    - MembershipPage.tsx — auth-aware page with three states (logged out / free / paid), tier picker → checkout modal, post-checkout polling on /auth/me, "Manage
  subscription" → portal redirect
    - index.ts — re-export
  - web/src/routes/membership.tsx — thin route file (TanStack Router auto-discovers; the routeTree regenerates on pnpm dev)
  - web/src/api/billing/billing.ts — Orval-generated useCreateCheckoutSession() and useCreatePortalSession() mutation hooks
  - Stripe deps added to web/package.json

  Tier prices (hardcoded in MembershipPage.tsx)

  Scholiast    €5/mo
  Benefactor   €15/mo
  Patron       €50/mo

  These are display-only — the actual charge comes from your Stripe Price objects. Verify these match what you set in the Stripe Dashboard (the IDs are in
  STRIPE_PRICE_BASE/_MID/_HIGH). If they don't, edit the TIERS const in MembershipPage.tsx.

  To exercise it end-to-end

  # Terminal 1 — start API
  cargo run --bin api --manifest-path packages/api/Cargo.toml

  # Terminal 2 — webhook tunnel (paste the whsec_ into .env, restart API)
  stripe listen --forward-to localhost:4000/webhooks/stripe

  # Terminal 3 — frontend
  cd web && pnpm dev

  Then visit http://localhost:3000/membership:
  - Logged out: pitch + "Sign up to support" CTAs
  - Logged in (free): three tier cards with "Become a [Tier]" buttons; click → modal opens with Embedded Checkout
  - Pay with 4242 4242 4242 4242 (any future expiry, any CVC)
  - Modal closes, "Activating your membership…" overlay appears, polling fires, role flips, toast says "Welcome, [Tier]!"
  - Page now shows the active subscription card with "Manage subscription" → redirects to Stripe Portal

  → Two known follow-ups I'd flag:
  1. Navbar entry: I didn't add a "Membership" link anywhere in Navbar.tsx / UserSubnav.tsx. The page works at /membership but isn't discoverable. Want me to add a nav link?
  2. vite.config.ts may need to know about the new env var if it does any allowlist filtering — though VITE_* prefixed vars are auto-exposed by Vite by default, so likely
  fine.
