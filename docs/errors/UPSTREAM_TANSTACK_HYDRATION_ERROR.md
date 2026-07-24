# TanStack Start: late streamed Suspense boundary races client hydration → React #418; stream can also omit the boundary reveal entirely

Draft issue report, ready to file against `TanStack/router` (Start lives in that
monorepo). Everything below was observed on a production deployment and
reproduced/instrumented with headless-browser probes; the raw evidence captures
are described inline.

---

## Summary

A route loader that mixes **awaited** queries with a **fire-and-forget**
`prefetchInfiniteQuery` (the documented streaming pattern) produces SSR
responses where the suspended section streams in late. Two distinct defects
were observed on the streamed path:

1. **Hydration race (client):** when the JS bundle is already in the browser
   cache, client hydration starts — and finishes taking over the still-pending
   Suspense boundary — *before* the tail of the SSR stream arrives. React's
   late `$RC` boundary-reveal script then mutates DOM that React has already
   claimed, throwing **minified React error #418**
   (`args[]=HTML` — "Hydration failed because the server rendered HTML didn't
   match the client") on **every warm-cache load**. Cold-cache loads are fine,
   which makes the bug look session-/machine-specific and very hard to
   attribute.

2. **Malformed stream (server):** some streamed responses contain a completed
   HTML document (`</html>` present) in which a segment
   (`<div hidden id="S:1">`, ~786 KB in our capture) was written **without its
   matching `<template id="B:1">` anchor and without any `$RC("B:1","S:1")`
   reveal script — and without an `$RX` abort either**. The segment is
   unreachable dead weight; the boundary stays in its pending state
   (`<!--$?-->`) despite the full content having been serialized into the
   response.

A single-word workaround (`await` the prefetch, so the route never streams)
eliminates both symptoms deterministically.

## Versions

| Package | Version |
| --- | --- |
| `@tanstack/react-start` | 1.168.32 |
| `@tanstack/react-router` | 1.170.18 |
| `@tanstack/react-router-ssr-query` | 1.167.1 (`router-ssr-query-core` 1.169.1) |
| `@tanstack/react-query` | 5.101.4 |
| `react` / `react-dom` | 19.2.8 |
| `srvx` (server runtime) | 0.12.4 |
| Node.js | 24.x |

All were the latest published versions at the time of writing; upgrading from
`react-start` 1.168.26 / `react-router` 1.170.16 / `react-query` 5.101.0 /
React 19.2.0 changed nothing.

## Setup that triggers it

Route loader (abridged) — three light queries awaited for `head()`, the heavy
chapter content deliberately fire-and-forget so it streams:

```tsx
loader: async ({ context, params }) => {
    // Heavy chapter content: fire-and-forget, streamed into the HTML
    // via react-query dehydration — awaiting it would block first-byte.
    context.queryClient.prefetchInfiniteQuery(getNodePageSuspenseQueryOptions(...));

    const [bookRes, tocRes, metaRes] = await Promise.all([
        context.queryClient.ensureQueryData(...),
        context.queryClient.ensureQueryData(...),
        context.queryClient.ensureQueryData(...),
    ]);
    return { /* scalars for head() */ };
},
```

The component under this route reads the infinite query with
`useSuspenseInfiniteQuery`. The SSR-query integration is the standard
`setupRouterSsrQueryIntegration({ router, queryClient })`.

Whether a given request streams is a per-request race: if the prefetch
resolves before React's render reaches the suspense point, the section is
rendered synchronously (no bug); if not, the shell flushes with a pending
boundary and the section streams (bug conditions armed). In production we saw
some clients reliably get one path and some the other, on identical URLs and
data — pure timing.

## Reproduction conditions (client race, defect 1)

All four must hold:

1. SSR response takes the **streamed** path (suspense query pending at shell
   flush).
2. Response is **actually streamed** to the browser (no full-response
   buffering in between — see "Environmental masking" below).
3. The JS bundle is in the **browser cache** (second visit), so hydration
   begins before the stream tail arrives.
4. Normal page load (hard refresh redownloads the bundle and un-arms
   condition 3).

Observed repro loop: open page in a new tab → no error (cold cache) → close
tab → open again → **React #418 every time** (warm cache). Incognito or hard
refresh → no error. This "first load fine, second load broken" signature is
characteristic.

Error as seen in production console:

```
Uncaught Error: Minified React error #418; visit
https://react.dev/errors/418?args[]=HTML&args[]= …
    at Pi (index-*.js:588:31196)
    …
    at MessagePort.O (index-*.js:581:126665)
```

The expanded stack bottoms out in the inline stream scripts injected by the
SSR runtime — `$RV`/`$RC` at the end of the HTML document (`$RC @ <page>:77`),
scheduled through `requestAnimationFrame`/`setTimeout` — i.e. the error is
triggered by the **boundary-completion script executing after hydration**, not
by app render logic.

## Evidence for the malformed stream (defect 2)

Marker census of a captured streamed response (authed request, no intermediary
caching, `X-Cache-Status: BYPASS`, `</html>` present):

```
<template id="B:...">   → ["B:0"]
<div hidden id="S:...">  → ["S:0", "S:1"]      // S:1 ≈ 786 KB (the entire suspended section)
$RC("B:n","S:n") calls   → [B:0 ← S:0]          // nothing reveals S:1
$RX calls                → none                  // not aborted either
```

`S:1` contains the complete rendered content of the suspended boundary, so
React *did* finish rendering it server-side — but the reveal instruction and
template anchor were never emitted. The client is left with a pending
boundary and takes over via client render (correct per se), yet the server
serialized ~786 KB that can never be used. Suspicion (unverified): the
integration's stream teardown (`serverSsr.onRenderFinished` →
`queryStream.close()` in `router-ssr-query-core`, and/or Start's stream
lifecycle) ends the response after React wrote the segment but before the
completion script, then still appends the document close.

Note the interaction: a client that hydrates **late** (cold cache) treats the
orphaned boundary as "server never finished" and client-renders it — no
error. A client that hydrates **early** on a *well-formed* streamed response
gets the `$RC`-after-hydration race instead. Both paths degrade; only the
timing decides which.

## Environmental masking (why this is so hard to attribute)

Our edge proxy (nginx) caches anonymous HTML, which forces full-response
buffering — anonymous users receive the response as one burst, data always
present before hydration, no race. Authenticated requests bypass the cache
and stream for real. The bug therefore presented as "only logged-in users,
only on the deployed environment, only on repeat visits" — three red
herrings deep. Local dev (same machine, near-zero latency) never streams
in practice, so it also never reproduced there.

## Workaround

```diff
-        context.queryClient.prefetchInfiniteQuery(
+        await context.queryClient.prefetchInfiniteQuery(
```

(then folded into the existing `Promise.all` so first byte costs
`max(all queries)` rather than being serialized). With the suspended section
rendered synchronously into the SSR HTML there is no streamed boundary, no
late `$RC`, and both defects are structurally impossible. Verified with 40
concurrent authed fetches (0 streamed segments, 0 orphans) and repeated
warm-cache browser reloads (0 errors) on the previously-failing sessions.

The cost of the workaround is real: streaming for that route is disabled
entirely, which is presumably not what Start intends the fire-and-forget
pattern for.

## What a fix might look like (from the outside)

- The client hydration barrier (`$_TSR`, and the ssr-query integration's
  `queryStream` reader, which applies streamed cache entries in promise
  microtasks) does not currently prevent React from hydrating/taking over a
  pending boundary whose reveal script is still in flight. Either delaying
  boundary takeover until stream end, or making late `$RC`/`$RV` no-ops once
  hydration owns the boundary, would remove defect 1.
- The server-side stream teardown should never emit a segment without its
  template + reveal (or should `$RX` the boundary instead) — defect 2.

Happy to provide the full HTML captures, HAR files, or a minimal repro on
request.
