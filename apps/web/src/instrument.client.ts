import * as Sentry from "@sentry/tanstackstart-react";
import config from "#/config";

Sentry.init({
    dsn: config.SENTRY_DSN,
    environment: config.PROFILE,
    enableLogs: true,
    tracesSampleRate: 0,
});
