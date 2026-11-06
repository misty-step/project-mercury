(() => {
  const { key, host } = document.currentScript.dataset;
  const meta = (name) => document.querySelector(`meta[name="mercury-${name}"]`).content;
  const common = { project: "project-mercury", release: meta("release"), environment: meta("environment") };
  const fields = {
    $pageview: [],
    filter_changed: ["axis", "value"],
    state_viewed: ["code", "regulation"],
    source_clicked: ["page_type", "source_type", "host"],
    state_map_tile_clicked: ["code"],
  };
  let client;
  let sent = 0;
  const capture = (event, properties) => {
    try { client?.capture(event, properties); } catch {
      console.warn("Mercury analytics event could not be sent.");
    }
  };
  document.addEventListener("mercury:filter_changed", (event) => capture("filter_changed", event.detail));
  document.addEventListener("click", (event) => {
    const link = event.target.closest?.("a");
    if (!link) return;
    if (link.dataset.stateTile) capture("state_map_tile_clicked", { code: link.dataset.stateTile });
    if (link.dataset.sourceType) {
      const path = location.pathname;
      const page_type = path.startsWith("/states/") ? "state" : path.startsWith("/entries/") ? "entry"
        : path.startsWith("/approaches/") ? "approach" : "resources";
      capture("source_clicked", { page_type, source_type: link.dataset.sourceType, host: new URL(link.href).host });
    }
  });
  const viewed = () => {
    const state = document.querySelector("[data-state-code]");
    if (state) capture("state_viewed", { code: state.dataset.stateCode, regulation: state.dataset.regulation });
  };
  const sdk = document.createElement("script");
  sdk.src = "https://cdn.jsdelivr.net/npm/posthog-js@1.434.0/dist/array.js";
  sdk.async = true;
  sdk.crossOrigin = "anonymous";
  sdk.onerror = () => console.warn("Mercury analytics is unavailable.");
  sdk.onload = () => {
    try {
      posthog.init(key, {
        api_host: host,
        persistence: "memory",
        disable_persistence: true,
        persistence_name: "mercury_analytics",
        cross_subdomain_cookie: false,
        autocapture: false,
        capture_pageview: true,
        capture_pageleave: false,
        capture_performance: false,
        capture_dead_clicks: false,
        rageclick: false,
        disable_session_recording: true,
        disable_surveys: true,
        disable_external_dependency_loading: true,
        advanced_disable_flags: true,
        advanced_disable_decide: true,
        person_profiles: "never",
        ip: false,
        save_referrer: false,
        save_campaign_params: false,
        request_batching: true,
        rate_limiting: { events_per_second: 2, events_burst_limit: 10 },
        before_send(event) {
          const allowed = fields[event.event];
          if (!allowed || sent >= 50) return null;
          sent += 1;
          const original = event.properties || {};
          const properties = { ...common, $current_url: location.origin + location.pathname,
            $pathname: location.pathname, $host: location.host, $geoip_disable: true, $process_person_profile: false };
          for (const name of ["token", "distinct_id", "$lib", "$lib_version", ...allowed]) {
            if (original[name] !== undefined) properties[name] = original[name];
          }
          return { event: event.event, properties, timestamp: event.timestamp, uuid: event.uuid };
        },
        loaded(instance) {
          client = instance;
          client.register(common);
          if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", viewed, { once: true });
          else viewed();
        },
      });
    } catch {
      console.warn("Mercury analytics could not start.");
    }
  };
  document.head.append(sdk);
})();
