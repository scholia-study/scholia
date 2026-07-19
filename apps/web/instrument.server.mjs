import * as Sentry from "@sentry/tanstackstart-react";

Sentry.init({
    dsn: process.env.SENTRY_DSN || undefined,
    environment: process.env.APP_PROFILE ?? "local",
    release: process.env.SENTRY_RELEASE,
    enableLogs: true,
    tracesSampleRate: 0,
});
