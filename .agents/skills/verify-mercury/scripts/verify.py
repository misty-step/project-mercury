#!/usr/bin/env python3
"""Capture Mercury's real build, CLI, HTTP identity, and rejection journeys."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import urllib.request

ROOT = Path(__file__).resolve().parents[4]


def save(evidence, name, value):
    path = evidence / name
    with path.open("x") as output:
        json.dump(value, output, indent=2)
        output.write("\n")
    print(path)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def capture(evidence, name, command, expected=0):
    if (evidence / name).exists():
        raise RuntimeError(f"refusing to overwrite {evidence / name}")
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
    save(evidence, name, {
        "argv": [str(arg) for arg in command], "cwd": str(ROOT),
        "exit_code": result.returncode, "expected_exit_code": expected,
        "stdout": result.stdout, "stderr": result.stderr,
    })
    print(result.stdout, end="")
    print(result.stderr, end="", file=sys.stderr)
    if result.returncode != expected:
        raise RuntimeError(f"expected exit {expected}, got {result.returncode}")
    return result


def launch(evidence):
    if (ROOT / "dist").exists():
        raise RuntimeError("dist already exists; use the run's clean worker checkout")
    identity = {key: os.environ[f"MERCURY_{key.upper()}"]
                for key in ("run_id", "revision", "worker_id")}
    identity["snapshot"] = "omp-worker"
    capture(evidence, "build.json", ["cargo", "build", "--locked", "-p", "mercury-site",
                                     "-p", "project-mercury-cli"])
    capture(evidence, "generate.json", ["target/debug/mercury-site", "--data", "data", "--out", "dist"])
    capture(evidence, "version.json", ["target/debug/project-mercury", "--version"])
    identity["dump_sha256"] = digest(ROOT / "dist/dump.json")
    save(evidence, "launch.json", identity)
    (ROOT / "dist/__verification.json").write_text(json.dumps(identity) + "\n")


def doctor(evidence, url):
    identity = json.loads((evidence / "launch.json").read_text())
    observations = []
    bodies = {}
    for route in ("/__verification.json", "/", "/dump.json", "/entries/math-academy/", "/states/", "/states/ca/"):
        with urllib.request.urlopen(url.rstrip("/") + route, timeout=5) as response:
            body = response.read()
            bodies[route] = body
            observations.append({"url": response.url, "status": response.status,
                                 "sha256": hashlib.sha256(body).hexdigest()})
    if json.loads(bodies["/__verification.json"]) != identity:
        raise RuntimeError("HTTP instance does not match this run's launch identity")
    if hashlib.sha256(bodies["/dump.json"]).hexdigest() != identity["dump_sha256"]:
        raise RuntimeError("HTTP catalog differs from this run's generated catalog")
    if digest(ROOT / "dist/dump.json") != identity["dump_sha256"]:
        raise RuntimeError("generated catalog changed after launch")
    if b"Project Mercury" not in bodies["/"] or b'id="age"' not in bodies["/"]:
        raise RuntimeError("home page is not the usable Mercury catalog")
    save(evidence, "doctor.json", {"identity": identity, "requests": observations, "result": "pass"})


def reject(evidence):
    original = ROOT / "data/entries/math-academy.yaml"
    original_hash = digest(original)
    before = original.read_text()
    needle = "    source_id: mathacademy-home\n"
    if needle not in before:
        raise RuntimeError("Math Academy fixture changed; update the mapped rejection journey")
    after = before.replace(needle, "    source_id: verification-unsourced\n", 1)
    with tempfile.TemporaryDirectory(prefix="mercury-reject-") as directory:
        scratch = Path(directory)
        data = scratch / "data"
        shutil.copytree(ROOT / "data", data)
        fixture = data / "entries/math-academy.yaml"
        fixture.write_text(after)
        save(evidence, "rejection-fixture.json", {"path": str(fixture), "before": before, "after": after})
        result = capture(evidence, "reject-validate.json", ["target/debug/project-mercury", "--data", str(data), "validate"], 1)
        diagnostic = "math-academy: claim on `subjects` cites unknown source `verification-unsourced`"
        if diagnostic not in result.stderr:
            raise RuntimeError("validator failed without the unsourced-claim diagnostic")
        output = scratch / "rejected-dist"
        result = capture(evidence, "reject-build.json", ["target/debug/mercury-site", "--data", str(data), "--out", str(output)], 1)
        if diagnostic not in result.stderr or output.exists():
            raise RuntimeError("invalid catalog was not rejected before writing the site")
        fixture.write_text(before)
        capture(evidence, "reject-recovery.json", ["target/debug/project-mercury", "--data", str(data), "validate"])
        if digest(original) != original_hash:
            raise RuntimeError("repository fixture changed during rejection")
        save(evidence, "rejection.json", {"result": "pass", "original_unchanged": True,
             "rejected_output_exists": False, "recovered": True, "scratch": str(scratch)})
    if scratch.exists():
        raise RuntimeError("rejection scratch directory survived cleanup")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--evidence", type=Path, required=True)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("launch")
    check = commands.add_parser("doctor")
    check.add_argument("--url", default="http://127.0.0.1:8000")
    cli = commands.add_parser("cli")
    cli.add_argument("--name", required=True, help="unique artifact basename without .json")
    cli.add_argument("args", nargs=argparse.REMAINDER)
    commands.add_parser("reject")
    args = parser.parse_args()
    evidence = args.evidence.expanduser().resolve()
    if evidence == ROOT or ROOT in evidence.parents:
        parser.error("evidence must be outside the repository")
    evidence.mkdir(parents=True, exist_ok=True)
    if args.command == "launch":
        launch(evidence)
    elif args.command == "doctor":
        doctor(evidence, args.url)
    elif args.command == "reject":
        reject(evidence)
    else:
        if not args.name or Path(args.name).name != args.name:
            parser.error("--name must be a basename")
        arguments = args.args[1:] if args.args[:1] == ["--"] else args.args
        if not arguments:
            parser.error("cli requires arguments after --")
        capture(evidence, args.name + ".json", ["target/debug/project-mercury", *arguments])


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, ValueError) as error:
        sys.exit(f"verification failed: {error}")
