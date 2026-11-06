# Unsourced claim rejection

A contributor receives an actionable error when a resource claim cites no existing source. Neither validation nor site generation may silently accept that catalog. Restoring the source makes validation succeed again.

## Sub-features

- `unsourced-cli-reject` rejects a valid YAML claim whose source reference is unknown.
- `unsourced-site-reject` rejects the same catalog before writing a site.
- `unsourced-recover` accepts the restored copy and preserves the original data.
- `unsourced-cleanup` removes disposable mutation data while retaining evidence.

## How to get to it (user POV)

- A contributor edits an entry's `source_id`, then runs `project-mercury --data DIR validate`.
- A contributor builds that data with `mercury-site --data DIR --out DIR`.
- The verification helper supplies a disposable, reproducible version of this contributor mistake.

## Driving it with shell

Preconditions:

- Launch built both binaries and accepted the committed data.
- Math Academy's first `subjects` claim cites `mathacademy-home` in the committed fixture.
- The run's worker `TMPDIR` exists outside the checkout. Do not mutate repository data.

- **Run.** Through `boat exec`, invoke `env TMPDIR="/home/user/.cache/tmp/$RUN_ID" .agents/skills/verify-mercury/scripts/verify.py --evidence "$WORKER_EVIDENCE" reject`.
- **Inspect the fault.** Open `rejection-fixture.json`. The helper copies the full data directory and changes only the first Math Academy claim's source ID to `verification-unsourced`. The YAML still parses. The original fixture remains unchanged.
- **CLI rejection.** Open `reject-validate.json`. Require the real CLI exit code `1` and stderr containing `math-academy: claim on \`subjects\` cites unknown source \`verification-unsourced\``. An unrelated parse or missing-file error does not pass.
- **Build rejection.** Open `reject-build.json`. Require the same diagnostic and generator exit `1`. Require `rejection.json` to record `rejected_output_exists: false`. The generator must not create its requested output directory for this invalid catalog.
- **Recovery.** Open `reject-recovery.json`. The helper restores the source reference in the disposable copy, then runs the same CLI with the same data directory. Require exit `0` and the valid-catalog summary.
- **Side effects and cleanup.** Require `rejection.json` with `original_unchanged: true`, `recovered: true`, and `result: pass`. The helper removes its scratch directory even after a failed check. Copy all transcripts to coordinator evidence before deleting the worker. Confirm the copied files still exist after Cleanup.

## Gotchas

- A missing `source_id` key may fail YAML decoding before the provenance validator runs. This recipe deliberately uses an unknown source ID to reach the real validator.
- The helper itself exits `0` only when both expected rejections, the no-output check, original-data check, and recovery pass. Its JSON transcripts preserve the binaries' actual rejected exit codes of `1`.
- An already existing output directory would make a no-write check ambiguous. The helper uses a new path inside its fresh scratch directory.
- If the first Math Academy claim or its source changes, update the fixture and expected diagnostic together. Do not accept a different error to keep the journey green.
- Validation proves internal sourcing consistency, not the truth of the source or legal correctness of a state-law claim.
