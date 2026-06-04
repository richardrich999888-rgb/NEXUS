#!/usr/bin/env python3
"""Syntriass CBOM Scanner.

Static cryptographic bill-of-materials scanner for post-quantum migration
planning. The scanner stays local: it does not upload files, connect to
databases, or attempt to decrypt protected material.
"""

from __future__ import annotations

import argparse
import dataclasses
import datetime as dt
import fnmatch
import json
import os
import re
import sys
from collections import Counter
from pathlib import Path
from typing import Iterable, Sequence


SEVERITY_ORDER = {
    "info": 0,
    "low": 1,
    "medium": 2,
    "high": 3,
    "critical": 4,
}

DEFAULT_EXCLUDED_DIRS = {
    ".git",
    ".hg",
    ".svn",
    ".venv",
    "__pycache__",
    "node_modules",
    "target",
    "dist",
    "build",
    ".pytest_cache",
}

TEXT_EXTENSIONS = {
    ".cfg",
    ".conf",
    ".crt",
    ".cs",
    ".css",
    ".env",
    ".go",
    ".h",
    ".html",
    ".ini",
    ".java",
    ".js",
    ".json",
    ".key",
    ".kt",
    ".lock",
    ".md",
    ".pem",
    ".properties",
    ".pub",
    ".py",
    ".rb",
    ".rs",
    ".sh",
    ".sql",
    ".swift",
    ".toml",
    ".ts",
    ".txt",
    ".xml",
    ".yaml",
    ".yml",
}

DATASTORE_EXTENSIONS = {
    ".bak",
    ".db",
    ".dump",
    ".mdb",
    ".sqlite",
    ".sqlite3",
    ".sqlitedb",
}


@dataclasses.dataclass(frozen=True)
class PatternRule:
    rule_id: str
    kind: str
    algorithm: str
    severity: str
    regex: re.Pattern[str]
    recommendation: str


@dataclasses.dataclass(frozen=True)
class Finding:
    path: str
    line: int
    rule_id: str
    kind: str
    algorithm: str
    severity: str
    evidence: str
    recommendation: str


@dataclasses.dataclass(frozen=True)
class ScanReport:
    root: str
    generated_at: str
    files_scanned: int
    files_skipped: int
    findings: list[Finding]

    def severity_counts(self) -> dict[str, int]:
        counts = Counter(f.severity for f in self.findings)
        return {sev: counts.get(sev, 0) for sev in ["critical", "high", "medium", "low", "info"]}


RULES: tuple[PatternRule, ...] = (
    PatternRule(
        "pem-rsa-private-key",
        "private-key",
        "RSA",
        "critical",
        re.compile(r"-----BEGIN RSA PRIVATE KEY-----"),
        "Rotate out of RSA; protect key material; plan ML-DSA or hybrid signing migration.",
    ),
    PatternRule(
        "pem-ec-private-key",
        "private-key",
        "ECC",
        "critical",
        re.compile(r"-----BEGIN EC PRIVATE KEY-----"),
        "Inventory ECC private keys; plan ML-DSA or SLH-DSA migration for signatures.",
    ),
    PatternRule(
        "pem-openssh-private-key",
        "private-key",
        "OpenSSH classical key",
        "high",
        re.compile(r"-----BEGIN OPENSSH PRIVATE KEY-----"),
        "Identify key type with ssh-keygen; replace RSA/ECDSA/Ed25519 keys in PQC migration plan.",
    ),
    PatternRule(
        "ssh-rsa-public-key",
        "ssh-public-key",
        "RSA",
        "high",
        re.compile(r"\bssh-rsa\b"),
        "Replace RSA SSH trust anchors with approved migration profile when supported.",
    ),
    PatternRule(
        "ssh-ecdsa-public-key",
        "ssh-public-key",
        "ECDSA",
        "high",
        re.compile(r"\becdsa-sha2-nistp(?:256|384|521)\b"),
        "Inventory ECDSA SSH trust anchors and plan post-quantum-capable access paths.",
    ),
    PatternRule(
        "ssh-ed25519-public-key",
        "ssh-public-key",
        "Ed25519",
        "high",
        re.compile(r"\bssh-ed25519\b"),
        "Track Ed25519 SSH usage; it is efficient today but not quantum-resistant.",
    ),
    PatternRule(
        "jwt-rsa-algorithm",
        "jwt-signature",
        "RSASSA",
        "high",
        re.compile(r"\bRS(?:256|384|512)\b"),
        "Inventory JWT issuers/verifiers using RS* algorithms; plan ML-DSA-capable token profiles.",
    ),
    PatternRule(
        "jwt-ecdsa-algorithm",
        "jwt-signature",
        "ECDSA",
        "high",
        re.compile(r"\bES(?:256|384|512)\b"),
        "Inventory JWT issuers/verifiers using ES* algorithms; plan ML-DSA-capable token profiles.",
    ),
    PatternRule(
        "jwt-eddsa-algorithm",
        "jwt-signature",
        "EdDSA",
        "high",
        re.compile(r"\bEdDSA\b"),
        "Track EdDSA token usage; define hybrid or PQC replacement before long-lived deployment.",
    ),
    PatternRule(
        "tls-ecdhe-reference",
        "tls-key-exchange",
        "ECDHE",
        "medium",
        re.compile(r"\bECDHE\b|\bX25519\b|\bsecp(?:256r1|384r1|521r1)\b|\bprime256v1\b", re.IGNORECASE),
        "Inventory TLS key-exchange dependencies; plan ML-KEM/hybrid TLS migration where available.",
    ),
    PatternRule(
        "tls-rsa-reference",
        "tls-signature-or-key-exchange",
        "RSA",
        "medium",
        re.compile(r"\bRSA\b"),
        "Classify RSA usage as encryption, signatures, certificates, or legacy key exchange.",
    ),
    PatternRule(
        "crypto-library-classical",
        "dependency",
        "Classical crypto library",
        "medium",
        re.compile(
            r"\b(openssl|rsa|p256|p384|ecdsa|ed25519-dalek|jsonwebtoken|pyjwt|cryptography|paramiko|ssh2)\b",
            re.IGNORECASE,
        ),
        "Review dependency usage and verify whether a PQC migration path exists.",
    ),
    PatternRule(
        "ring-crypto-library",
        "dependency",
        "ring crypto library",
        "medium",
        re.compile(r"(?:\bname\s*=\s*[\"']ring[\"']|[\"']ring(?:\s+\d[^\"']*)?[\"']|\bring\s*=)", re.IGNORECASE),
        "Review ring usage and verify whether a PQC migration path exists.",
    ),
    PatternRule(
        "pqc-library-reference",
        "dependency",
        "PQC library",
        "info",
        re.compile(r"\b(fips203|fips204|fips205|ml-kem|ml_dsa|ml-dsa|slh-dsa|kyber|dilithium|sphincs)\b", re.IGNORECASE),
        "Confirm the PQC implementation is tested, feature-enabled, and wired into production flows.",
    ),
)


def should_skip_dir(path: Path, excluded_dirs: set[str], include_hidden: bool) -> bool:
    name = path.name
    if name in excluded_dirs:
        return True
    if not include_hidden and name.startswith("."):
        return True
    return False


def is_probably_text(path: Path) -> bool:
    if path.suffix.lower() in TEXT_EXTENSIONS:
        return True
    if path.name in {
        "Dockerfile",
        "Makefile",
        "Cargo.lock",
        "authorized_keys",
        "known_hosts",
        "package-lock.json",
        "yarn.lock",
    }:
        return True
    return False


def matches_excluded_glob(path: Path, root: Path, excluded_globs: Sequence[str]) -> bool:
    relative = safe_relative(path, root).replace(os.sep, "/")
    return any(fnmatch.fnmatch(relative, pattern) for pattern in excluded_globs)


def iter_files(
    root: Path,
    excluded_dirs: set[str],
    include_hidden: bool,
    excluded_globs: Sequence[str] = (),
) -> Iterable[Path]:
    if root.is_file():
        if not matches_excluded_glob(root, root.parent, excluded_globs):
            yield root
        return

    for dirpath, dirnames, filenames in os.walk(root):
        current = Path(dirpath)
        dirnames[:] = [
            dirname
            for dirname in dirnames
            if not should_skip_dir(current / dirname, excluded_dirs, include_hidden)
        ]
        for filename in filenames:
            path = current / filename
            if not include_hidden and any(part.startswith(".") for part in path.relative_to(root).parts):
                continue
            if matches_excluded_glob(path, root, excluded_globs):
                continue
            yield path


def safe_relative(path: Path, root: Path) -> str:
    try:
        return str(path.resolve().relative_to(root.resolve()))
    except ValueError:
        return str(path)


def normalize_evidence(text: str, max_len: int = 120) -> str:
    text = " ".join(text.strip().split())
    if len(text) <= max_len:
        return text
    return text[: max_len - 3] + "..."


def datastore_finding(path: Path, root: Path) -> Finding | None:
    if path.suffix.lower() not in DATASTORE_EXTENSIONS:
        return None
    return Finding(
        path=safe_relative(path, root),
        line=0,
        rule_id="datastore-artifact",
        kind="data-at-rest",
        algorithm="unknown",
        severity="medium",
        evidence=f"File extension {path.suffix.lower()} indicates database, dump, or backup artifact.",
        recommendation="Verify encryption-at-rest, backup controls, retention period, and ML-KEM/hybrid envelope plan for long-life data.",
    )


def scan_text_file(path: Path, root: Path, max_file_size: int) -> tuple[list[Finding], bool]:
    try:
        stat = path.stat()
    except OSError:
        return [], False

    if stat.st_size > max_file_size or not is_probably_text(path):
        return [], False

    try:
        text = path.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return [], False

    findings: list[Finding] = []
    for line_no, line in enumerate(text.splitlines(), start=1):
        for rule in RULES:
            if rule.regex.search(line):
                findings.append(
                    Finding(
                        path=safe_relative(path, root),
                        line=line_no,
                        rule_id=rule.rule_id,
                        kind=rule.kind,
                        algorithm=rule.algorithm,
                        severity=rule.severity,
                        evidence=normalize_evidence(line),
                        recommendation=rule.recommendation,
                    )
                )
    return findings, True


def scan_path(
    root: Path,
    *,
    max_file_size: int = 1_000_000,
    include_hidden: bool = False,
    excluded_dirs: set[str] | None = None,
    excluded_globs: Sequence[str] = (),
) -> ScanReport:
    root = root.resolve()
    excluded = set(DEFAULT_EXCLUDED_DIRS if excluded_dirs is None else excluded_dirs)
    findings: list[Finding] = []
    files_scanned = 0
    files_skipped = 0

    for path in iter_files(root, excluded, include_hidden, excluded_globs):
        data_finding = datastore_finding(path, root)
        if data_finding:
            findings.append(data_finding)

        file_findings, scanned = scan_text_file(path, root, max_file_size)
        if scanned:
            files_scanned += 1
            findings.extend(file_findings)
        else:
            files_skipped += 1

    findings.sort(
        key=lambda f: (-SEVERITY_ORDER[f.severity], f.path, f.line, f.rule_id)
    )
    return ScanReport(
        root=str(root),
        generated_at=dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat(),
        files_scanned=files_scanned,
        files_skipped=files_skipped,
        findings=findings,
    )


def report_to_dict(report: ScanReport) -> dict[str, object]:
    return {
        "tool": "syntriass-cbom-scanner",
        "version": "0.1.0",
        "root": report.root,
        "generated_at": report.generated_at,
        "files_scanned": report.files_scanned,
        "files_skipped": report.files_skipped,
        "severity_counts": report.severity_counts(),
        "findings": [dataclasses.asdict(finding) for finding in report.findings],
        "limitations": [
            "Static indicator scan only; findings require human review.",
            "No network traffic capture, HSM query, certificate-chain validation, or live database connection.",
            "PQC readiness requires protocol and target-environment integration tests.",
        ],
    }


def render_markdown(report: ScanReport) -> str:
    counts = report.severity_counts()
    lines = [
        "# Syntriass CBOM Scanner Report",
        "",
        f"- Generated: `{report.generated_at}`",
        f"- Root: `{report.root}`",
        f"- Files scanned: `{report.files_scanned}`",
        f"- Files skipped: `{report.files_skipped}`",
        f"- Findings: `{len(report.findings)}`",
        "",
        "## Severity Summary",
        "",
        "| Severity | Count |",
        "|---|---:|",
    ]
    for severity in ["critical", "high", "medium", "low", "info"]:
        lines.append(f"| {severity.upper()} | {counts[severity]} |")

    lines.extend(
        [
            "",
            "## Defence Relevance",
            "",
            "- RSA/ECC-era identities and signatures are migration priorities for post-quantum readiness.",
            "- Database, dump, and backup artifacts may represent harvest-now-decrypt-later exposure if long-life sensitive data is present.",
            "- Findings are discovery evidence, not proof of compromise.",
            "",
            "## Findings",
            "",
            "| Severity | Kind | Algorithm | Location | Evidence | Recommendation |",
            "|---|---|---|---|---|---|",
        ]
    )
    for finding in report.findings:
        location = finding.path if finding.line == 0 else f"{finding.path}:{finding.line}"
        evidence = finding.evidence.replace("|", "\\|")
        recommendation = finding.recommendation.replace("|", "\\|")
        lines.append(
            f"| {finding.severity.upper()} | {finding.kind} | {finding.algorithm} | "
            f"`{location}` | `{evidence}` | {recommendation} |"
        )

    if not report.findings:
        lines.append("| INFO | none | none | `-` | `No indicators detected.` | Continue inventory with live assets and vendor systems. |")

    lines.extend(
        [
            "",
            "## Limitations",
            "",
            "- Static scanner only; it does not upload data, decrypt secrets, query HSMs, or connect to databases.",
            "- Certificate algorithm parsing is shallow in this MVP.",
            "- PQC migration must be validated with protocol-level and target-hardware tests.",
            "",
        ]
    )
    return "\n".join(lines)


def write_report(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def exit_code_for_threshold(report: ScanReport, fail_on: str) -> int:
    if fail_on == "none":
        return 0
    threshold = SEVERITY_ORDER[fail_on]
    return 2 if any(SEVERITY_ORDER[f.severity] >= threshold for f in report.findings) else 0


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Create a local cryptographic bill of materials for PQC migration planning."
    )
    parser.add_argument("path", nargs="?", default=".", help="File or directory to scan.")
    parser.add_argument("--json", dest="json_path", help="Write JSON report to this path.")
    parser.add_argument("--markdown", dest="markdown_path", help="Write Markdown report to this path.")
    parser.add_argument(
        "--fail-on",
        choices=["none", "low", "medium", "high", "critical"],
        default="none",
        help="Exit with status 2 if findings at or above this severity are present.",
    )
    parser.add_argument(
        "--include-hidden",
        action="store_true",
        help="Include hidden files and directories.",
    )
    parser.add_argument(
        "--max-file-size",
        type=int,
        default=1_000_000,
        help="Maximum text file size to scan in bytes.",
    )
    parser.add_argument(
        "--exclude-glob",
        action="append",
        default=[],
        help="Exclude files matching a path glob relative to the scan root. May be provided multiple times.",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    root = Path(args.path)
    if not root.exists():
        print(f"error: path does not exist: {root}", file=sys.stderr)
        return 1

    report = scan_path(
        root,
        max_file_size=args.max_file_size,
        include_hidden=args.include_hidden,
        excluded_globs=args.exclude_glob,
    )
    data = report_to_dict(report)

    if args.json_path:
        write_report(Path(args.json_path), json.dumps(data, indent=2) + "\n")
    if args.markdown_path:
        write_report(Path(args.markdown_path), render_markdown(report))

    counts = report.severity_counts()
    print(
        "CBOM scan complete: "
        f"{len(report.findings)} findings "
        f"(critical={counts['critical']}, high={counts['high']}, medium={counts['medium']}, "
        f"low={counts['low']}, info={counts['info']})"
    )
    return exit_code_for_threshold(report, args.fail_on)


if __name__ == "__main__":
    raise SystemExit(main())
