import * as Sentry from "@sentry/tanstackstart-react";
import posthog from "posthog-js";
import config from "#/config";

Sentry.init({
    dsn: config.SENTRY_DSN,
    environment: config.PROFILE,
    enableLogs: true,
    tracesSampleRate: 0,
});

if (config.POSTHOG_TOKEN) {
    posthog.init(config.POSTHOG_TOKEN, {
        api_host: config.POSTHOG_HOST,
        defaults: "2026-05-30",
        persistence: "localStorage",
    });
}
