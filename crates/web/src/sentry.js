(() => {
  const meta = (name) => document.querySelector(`meta[name="mercury-${name}"]`).content;
  const release = meta("release");
  const environment = meta("environment");
  const cleanUrl = (value) => {
    try {
      const url = new URL(value, location.origin);
      return /^https?:$/.test(url.protocol) ? url.origin + url.pathname : undefined;
    } catch {
      return undefined;
    }
  };
  let sent = 0;
  window.sentryOnLoad = () => {
    try {
      Sentry.init({
        release,
        environment,
        sendDefaultPii: false,
        autoSessionTracking: false,
        sendClientReports: false,
        sampleRate: environment === "production" ? 0.25 : 1,
        tracesSampleRate: 0,
        replaysSessionSampleRate: 0,
        replaysOnErrorSampleRate: 0,
        maxBreadcrumbs: 0,
        integrations: (defaults) => defaults.filter((item) =>
          ["GlobalHandlers", "InboundFilters", "Dedupe", "BrowserApiErrors"].includes(item.name)),
        beforeSend(event) {
          if (sent >= 5 || !event.exception?.values?.length) return null;
          sent += 1;
          return {
            event_id: event.event_id,
            timestamp: event.timestamp,
            platform: "javascript",
            level: "error",
            release,
            environment,
            tags: { project: "project-mercury" },
            request: { url: cleanUrl(location.href) },
            exception: {
              values: event.exception.values.slice(0, 1).map((error) => ({
                type: /^(Error|TypeError|ReferenceError|SyntaxError|RangeError|URIError|EvalError|AggregateError)$/.test(error.type)
                  ? error.type : "Error",
                value: "Browser error. Message omitted for privacy.",
                stacktrace: {
                  frames: (error.stacktrace?.frames || []).slice(-20).map((frame) => ({
                    filename: cleanUrl(frame.filename),
                    lineno: frame.lineno,
                    colno: frame.colno,
                    in_app: frame.in_app,
                  })),
                },
              })),
            },
          };
        },
      });
    } catch {
      console.warn("Mercury error reporting could not start.");
    }
  };
  const loader = document.createElement("script");
  loader.src = "https://js.sentry-cdn.com/6615a621d9cbea0470f7db6d7792eb31.min.js";
  loader.async = true;
  loader.crossOrigin = "anonymous";
  loader.onerror = () => console.warn("Mercury error reporting is unavailable.");
  document.head.append(loader);
})();
