---
name: verify-mercury
description: "Launch and drive Project Mercury's static catalog in the OMP browser and its Rust CLI. Use before merging changes to resources, state pages, filters, validation, or generated site output."
---

# Verify Project Mercury

Use this skill from the repository root. Read [the feature map](features/README.md), then choose every journey affected by the change. The public website is the primary interface. The `project-mercury` CLI is the other interface. This procedure does not deploy.

## Accounts

Not applicable. The generated site and CLI have no sign-in, database, or private family data. Use the committed catalog and disposable copies only. Do not add information about a specific family to fixtures, logs, or evidence.

## Launch

1. Read `skill://boat-workers`. Check the installed CLI's help, login, live allowance, and saved snapshots. Require an included start and a ready `omp-worker` snapshot. One slice uses one worker. Reuse it for retries, stop it while idle, and delete it after evidence collection. The worker needs Rust, Cargo, and Python 3. No application secrets or environment file are needed.
2. Create a run ID and evidence directory on the coordinator. Keep both evidence directories outside their checkouts. Record the revision and any uncommitted source changes before upload.

```sh
export RUN_ID="verify-mercury-$(date -u +%Y%m%dT%H%M%SZ)"
export EVIDENCE="$HOME/.local/share/project-mercury/verification/$RUN_ID"
export TMPDIR="$HOME/.cache/tmp/$RUN_ID"
export WORKER_ROOT=/home/user/project-mercury
export WORKER_EVIDENCE="/home/user/.local/share/project-mercury/verification/$RUN_ID"
export REVISION="$(git rev-parse HEAD)"
mkdir -p "$EVIDENCE/browser" "$TMPDIR"
git rev-parse HEAD > "$EVIDENCE/source-revision.txt"
git status --short > "$EVIDENCE/source-state.txt"
git ls-files --cached --others --exclude-standard > "$EVIDENCE/upload-files.txt"
boat --no-update new --from omp-worker --no-env --no-auto-stop --json
```

Save the creation response as `$EVIDENCE/worker-create.json`. Set `WORKER_ID` to the actual `bx_...` ID in that response. Never use `current`. Inspect `upload-files.txt` for secrets and unrelated files before pushing.

```sh
boat --no-update push "$WORKER_ID" . --to "$WORKER_ROOT"
boat --no-update exec "$WORKER_ID" --cwd "$WORKER_ROOT" --timeout 0 -- \
  env MERCURY_RUN_ID="$RUN_ID" MERCURY_REVISION="$REVISION" MERCURY_WORKER_ID="$WORKER_ID" \
  TMPDIR="/home/user/.cache/tmp/$RUN_ID" \
  sh -c 'mkdir -p "$TMPDIR" && .agents/skills/verify-mercury/scripts/verify.py --evidence "$1" launch' sh "$WORKER_EVIDENCE"
```

The executable helper builds both Rust binaries with `--locked`, then runs `mercury-site --data data --out dist`. The generator validates the committed catalog before writing pages. A fresh worker checkout must not already contain `dist/`. Success produces `build.json`, `generate.json`, `version.json`, and `launch.json` in the worker evidence directory. `dist/__verification.json` is a verification-only run identity, not a product asset to deploy.

3. Start the README's Python static server on the worker. Retain the returned process ID as `SERVER_PID` and save the response as `$EVIDENCE/server-start.json`.

```sh
boat --no-update exec "$WORKER_ID" --cwd "$WORKER_ROOT" --detach --json -- \
  python3 -u -m http.server --bind 0.0.0.0 --directory dist 8000
boat --no-update exec "$WORKER_ID" --status "$SERVER_PID" --json
```

Readiness requires the `Serving HTTP on` log and a passing Doctor. A listening port alone is not enough.

4. Forward the worker's port through an owned, named background service in the coordinator harness. Keep its service handle for cleanup.

```sh
boat --no-update forward "$WORKER_ID" --remote 8000 --local 8765 --bind 127.0.0.1
```

Use `http://127.0.0.1:8765` as `APP_URL`. If the local port is occupied, choose another free local port and record the resulting URL. Do not stop its existing owner. Each worker has its own generated files, server, and rejection fixture. Concurrent runs need distinct run IDs, local ports, and browser tab names.

## Doctor

Run this read-only application probe before driving and whenever the instance looks wrong. It writes observations only to the evidence directory.

```sh
boat --no-update exec "$WORKER_ID" --cwd "$WORKER_ROOT" -- \
  .agents/skills/verify-mercury/scripts/verify.py --evidence "$WORKER_EVIDENCE" doctor
```

Require `doctor.json` with `result: pass`. The probe checks the served run identity against `launch.json`, compares the served catalog digest with the generated catalog, and fetches the home page, Math Academy, states index, and California page. It rejects a wrong or stale instance. Also read `APP_URL/__verification.json` through the forwarded connection. Require the same run ID, revision, and worker ID before opening the browser. Keep that response as `$EVIDENCE/forwarded-identity.json`.

Doctor does not claim that JavaScript, links, filters, or CLI journeys work. Those need Drive. If Doctor fails, inspect the recorded build output and owned server status. Run Cleanup after a failed attempt. Preserve the failed evidence and use a new run ID rather than overwriting it.

## Drive

Read `xd://eval/browser` before the first browser call. Use OMP's browser through Eval, with a dedicated headless tab. Do not borrow an operator tab or use their signed-in session. This app needs neither.

Copy the shell's run ID and expanded evidence path into Eval's environment with `env("MERCURY_RUN_ID", runId)` and `env("EVIDENCE", evidencePath)`. These values identify this run, not a new run.

```js
const appUrl = "http://127.0.0.1:8765";
const runId = env("MERCURY_RUN_ID");
const tab = await browser.open({
  name: runId,
  url: appUrl,
  app: { relay: false, tern: false },
  headed: false,
  viewport: { width: 1440, height: 1100 },
  allowed_domains: ["127.0.0.1"],
});
print(await tab.observe());
```

Use `tab.select('#age', '9')` for the age control. Use `.chip[data-axis="subject"][data-value="math"]` for the math chip. Read `#count`, `#grid > li:not([hidden])`, `#empty`, and `#clear` after each action. Interact with browser controls, not injected DOM setters. Read-only DOM inspection is suitable for assertions. The catalog recipe supplies the exact expected fixture checks and reset path.

Follow the selected [feature file](features/README.md) for entry points and outcomes. Record each action and its resulting state. A page reached directly does not prove navigation from the map, table, or approach listing.

For the CLI, run the built binary through the helper. Give every command a unique evidence name. The helper preserves the exact invocation, working directory, stdout, stderr, and exit status.

```sh
boat --no-update exec "$WORKER_ID" --cwd "$WORKER_ROOT" -- \
  .agents/skills/verify-mercury/scripts/verify.py --evidence "$WORKER_EVIDENCE" \
  cli --name cli-search -- search --age 9 --subject math --json
boat --no-update exec "$WORKER_ID" --cwd "$WORKER_ROOT" -- \
  .agents/skills/verify-mercury/scripts/verify.py --evidence "$WORKER_EVIDENCE" \
  cli --name cli-show -- show math-academy
boat --no-update exec "$WORKER_ID" --cwd "$WORKER_ROOT" -- \
  .agents/skills/verify-mercury/scripts/verify.py --evidence "$WORKER_EVIDENCE" \
  cli --name cli-validate -- validate
```

Run the mapped unsourced-claim rejection when validation or generation changes, and when proving this skill's own end-to-end recipe.

```sh
boat --no-update exec "$WORKER_ID" --cwd "$WORKER_ROOT" -- \
  env TMPDIR="/home/user/.cache/tmp/$RUN_ID" \
  .agents/skills/verify-mercury/scripts/verify.py --evidence "$WORKER_EVIDENCE" reject
```

Require the specific unknown-source diagnostic, exit `1` from both validation and generation, no rejected site directory, a successful restored validation, and `rejection.json` with `result: pass`. A syntax error or any unrelated nonzero exit is not a passing rejection.

## Evidence

Keep the proof at `$HOME/.local/share/project-mercury/verification/$RUN_ID` on the coordinator. This directory survives worker deletion and worktree removal. Do not put evidence in Git, `dist/`, the worktree, or the run's `TMPDIR`.

For a browser interaction, capture a short video covering the action and result, a timestamped action log with assertions, before and after screenshots at the same viewport, and an ARIA snapshot. Start recording before the first action.

```js
const evidence = env("EVIDENCE");
await tab.recordStart(`${evidence}/browser/catalog.webm`);
await tab.run(async ({ page }, directory) => {
  await page.screenshot({ path: `${directory}/browser/catalog-before.png`, fullPage: true });
}, { args: [evidence] });
await tab.select("#age", "9");
const actions = [{ action: "select age 9", count: await tab.text("#count"), at: new Date().toISOString() }];
await tab.click('.chip[data-axis="subject"][data-value="math"]');
actions.push({ action: "select math", count: await tab.text("#count"), at: new Date().toISOString() });
await write(`${evidence}/browser/catalog-actions.json`, JSON.stringify(actions, null, 2));
await write(`${evidence}/browser/catalog.aria.txt`, await tab.ariaSnapshot());
await tab.run(async ({ page }, directory) => {
  await page.screenshot({ path: `${directory}/browser/catalog-after.png`, fullPage: true });
}, { args: [evidence] });
await tab.recordStop();
```

For a CLI journey, retain the helper's JSON transcript and inspect stdout, stderr, and the real exit code. For rejection, also inspect `rejection-fixture.json`, `reject-build.json`, `reject-recovery.json`, and `rejection.json`. The helper changes a disposable copy and verifies that the repository fixture stays unchanged. No mock endpoint or internal validation call substitutes for these binaries.

Before deleting the worker, copy its evidence back once.

```sh
boat --no-update scp -r "$WORKER_ID:$WORKER_EVIDENCE" "$EVIDENCE/worker"
boat --no-update usage "$WORKER_ID" --json
```

Keep the usage response with the proof. Open every screenshot, video, ARIA snapshot, and command transcript you cite. Check legibility, revision, instance identity, and absence of secrets or family data. Save a short `result.json` connecting feature IDs, exact actions, observed outcomes, source revision and dirty state, runtime URL, worker ID, artifacts, and unverified entry points. Missing fixture preconditions are failures, not a reason to narrow the recipe.

Publish inspected proof through native GitHub PR attachments. A local path alone is not a delivered PR link. If attachments are unavailable, use an authorized non-latest evidence-only prerelease asset in this repository, not an evidence commit or an expiring URL. Link the retained bundle and state exact proof gaps in the PR's Evidence section. Do not describe unexecuted features as verified.

## Cleanup

1. Stop recording if it is active. Close only the tab this run opened with `browser.close({name: runId})`.
2. Copy and inspect the evidence before deletion. Stop the exact server process with its recorded Boat process ID. Save the server's final status and logs if they explain a failure.

```sh
boat --no-update exec "$WORKER_ID" --kill "$SERVER_PID" --json
```

3. Stop the owned forward using its coordinator service handle. Never kill by process name or port. Delete only the worker this run created.

```sh
boat --no-update delete "$WORKER_ID" --yes --json
boat --no-update list --all --json
```

Save the deletion receipt as `$EVIDENCE/worker-delete.json`. Require completed deletion and the owned ID absent from the complete inventory. If deletion returns an operation ID, follow `boat deletion status` using that exact ID. Worker deletion removes the generated site, verification marker, binaries, server, and worker scratch. The rejection helper removes its disposable data even on failure. Remove only the coordinator's run-scoped `TMPDIR` after collection.

4. Reopen `$EVIDENCE/result.json`, its screenshots, and the copied CLI transcripts after cleanup. Confirm the delivered PR evidence link is readable with repository access. Cleanup must not remove the evidence directory or its published copy. Report deletion or publication failures explicitly.

## Helpers

[`scripts/verify.py`](scripts/verify.py) is an executable Python 3 standard-library helper. Run it from the worker checkout as shown above. Its `launch`, `doctor`, `cli`, and `reject` commands capture the real binaries and served output. It refuses evidence inside the repository and refuses to overwrite a named transcript. It does not create workers, forward ports, drive a browser, deploy, or delete workers.

Maintain this skill and its map with `/maintain-verification-skill` when routes, controls, data preconditions, CLI flags, or launch commands change. The repository's `.agents/skills/verify-mercury/` directory is canonical.
