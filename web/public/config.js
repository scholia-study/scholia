// Local dev — no overrides; the app falls back to the "local" profile.
// In containerized deployments this file is overwritten at pod startup
// by an envsubst-rendered version of `config.js.template` that sets
// `window.__ENV__ = { APP_PROFILE: "<dev|prod>" }`.
