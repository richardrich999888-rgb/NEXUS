import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCANNER_PATH = ROOT / "tools" / "cbom_scanner" / "cbom_scan.py"

spec = importlib.util.spec_from_file_location("cbom_scan", SCANNER_PATH)
cbom_scan = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules["cbom_scan"] = cbom_scan
spec.loader.exec_module(cbom_scan)


class CbomScannerTests(unittest.TestCase):
    def test_detects_classical_crypto_and_datastore_indicators(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "app.py").write_text('JWT_ALG = "RS256"\ncurve = "X25519"\n', encoding="utf-8")
            (root / "authorized_keys").write_text("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC test\n", encoding="utf-8")
            (root / "mission.sqlite").write_bytes(b"SQLite format 3\x00")

            report = cbom_scan.scan_path(root)
            rule_ids = {finding.rule_id for finding in report.findings}

            self.assertIn("jwt-rsa-algorithm", rule_ids)
            self.assertIn("tls-ecdhe-reference", rule_ids)
            self.assertIn("ssh-rsa-public-key", rule_ids)
            self.assertIn("datastore-artifact", rule_ids)
            self.assertGreaterEqual(report.severity_counts()["high"], 2)

    def test_default_excludes_target_directory(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            target = root / "target"
            target.mkdir()
            (target / "leaked.pem").write_text("-----BEGIN RSA PRIVATE KEY-----\n", encoding="utf-8")

            report = cbom_scan.scan_path(root)

            self.assertEqual(report.findings, [])

    def test_cli_writes_json_and_markdown(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "package.json").write_text('{"dependencies":{"jsonwebtoken":"latest"}}\n', encoding="utf-8")
            json_path = root / "report.json"
            md_path = root / "report.md"

            result = subprocess.run(
                [
                    sys.executable,
                    str(SCANNER_PATH),
                    str(root),
                    "--json",
                    str(json_path),
                    "--markdown",
                    str(md_path),
                ],
                check=False,
                text=True,
                capture_output=True,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            data = json.loads(json_path.read_text(encoding="utf-8"))
            self.assertEqual(data["tool"], "syntriass-cbom-scanner")
            self.assertTrue(data["findings"])
            self.assertIn("Syntriass CBOM Scanner Report", md_path.read_text(encoding="utf-8"))

    def test_fail_on_threshold_returns_status_two(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "token.conf").write_text("alg=ES256\n", encoding="utf-8")

            result = subprocess.run(
                [sys.executable, str(SCANNER_PATH), str(root), "--fail-on", "high"],
                check=False,
                text=True,
                capture_output=True,
            )

            self.assertEqual(result.returncode, 2)


if __name__ == "__main__":
    unittest.main()
