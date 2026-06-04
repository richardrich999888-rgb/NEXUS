import fs from "fs";
import path from "path";
import { execFileSync } from "child_process";
import { pathToFileURL } from "url";

const ROOT = "/Users/richardrich/Desktop/NEXUS";
const OUT_DIR = path.join(ROOT, "docs/idex-open-challenge-2026/07-causalux-contested-sync/final_4_documents/evidence_assets");
const CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const LOGO_IMAGE = "/Users/richardrich/WhatsApp Image 2025-12-23 at 16.05.08.jpeg";
const REPO_URL = "https://github.com/richardrich999888-rgb/NEXUS";

fs.mkdirSync(OUT_DIR, { recursive: true });

function esc(value) {
  return String(value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function readLines(file, start, end) {
  const fullPath = path.join(ROOT, file);
  const lines = fs.readFileSync(fullPath, "utf8").split("\n");
  return lines
    .slice(start - 1, end)
    .map((line, idx) => `${String(start + idx).padStart(4, " ")}  ${line}`)
    .join("\n");
}

function excerpt(parts) {
  return parts
    .map((part) => typeof part === "string" ? part : readLines(part.file, part.start, part.end))
    .join("\n\n");
}

function testSummary() {
  return [
    "$ cargo test -p causalux-v2 --lib --tests -- --nocapture",
    "RESULT: 59 library tests passed + 1 integration test passed",
    "",
    "$ cargo test -p nexus-sync --lib -- --nocapture",
    "RESULT: 10 library tests passed",
    "",
    "$ cargo test -p nexus-pcu uso -- --nocapture",
    "RESULT: 10 library USO tests + 2 chaos tests + 3 fuzz tests passed",
    "",
    "$ cargo test -p nexus-compress -- --nocapture",
    "RESULT: 5 compression tests passed",
    "",
    "TOTAL SELECTED EXECUTABLE EVIDENCE: 90 passed checks / 0 failed in selected commands",
    "Recorded output: evidence_assets/causalux_contested_sync_clean_test_output.txt"
  ].join("\n");
}

function caveatSummary() {
  return [
    "BROADER PRE-SUBMISSION RUN CAVEATS",
    "",
    "$ cargo test -p causalux-v2 -- --nocapture",
    "Library and integration tests passed, but one doctest failed.",
    "Reason: stale documentation example imports ed25519_dalek::Keypair, while the current crate API uses newer signing-key types.",
    "",
    "$ cargo test -p nexus-sync -- --nocapture",
    "Library tests pass, but stale integration_e2e.rs does not compile.",
    "Observed issues: missing nexus_compress dev dependency in that test target, stale PCU::new argument shape, missing bincode test dependency, and Vec<u8> where ContentHash is expected.",
    "",
    "Submission position: software-subsystem TRL 3-4. The iDEX work package includes updating stale E2E tests and adding network-in-loop contested-link validation."
  ].join("\n");
}

function pageHtml({ title, subtitle, kind, body }) {
  const logoUrl = pathToFileURL(LOGO_IMAGE).href;
  return `<!doctype html>
<html>
<head>
  <meta charset="utf-8" />
  <style>
    * { box-sizing: border-box; }
    body {
      margin: 0;
      width: 1440px;
      height: 960px;
      overflow: hidden;
      background:
        radial-gradient(circle at 10% 8%, rgba(37, 223, 255, 0.20), transparent 30%),
        radial-gradient(circle at 92% 18%, rgba(216, 43, 241, 0.13), transparent 26%),
        linear-gradient(180deg, #ffffff 0%, #f7f8fc 100%);
      font-family: Aptos, Calibri, Arial, sans-serif;
      color: #172033;
      padding: 54px 64px;
    }
    .card {
      position: relative;
      width: 100%;
      height: 100%;
      background: rgba(255, 255, 255, 0.88);
      border: 1px solid #d7dce8;
      border-radius: 28px;
      padding: 42px 46px;
      box-shadow: 0 20px 70px rgba(22, 32, 55, 0.08);
      overflow: hidden;
    }
    .card::after {
      content: "";
      position: absolute;
      right: -120px;
      bottom: -160px;
      width: 440px;
      height: 440px;
      background: url("${logoUrl}") center/cover no-repeat;
      opacity: 0.05;
      border-radius: 50%;
    }
    .header {
      display: grid;
      grid-template-columns: 210px 1fr 380px;
      gap: 28px;
      align-items: center;
      padding-bottom: 28px;
      border-bottom: 1px solid #d7dce8;
      position: relative;
      z-index: 2;
    }
    .logo {
      width: 190px;
      height: 108px;
      object-fit: cover;
      border-radius: 16px;
      border: 1px solid rgba(18, 18, 63, 0.12);
    }
    .brand {
      color: #12123f;
      font-size: 28px;
      font-weight: 800;
      letter-spacing: 4px;
    }
    .tagline {
      margin-top: 10px;
      color: #677089;
      font-size: 14px;
      letter-spacing: 0.7px;
    }
    .meta {
      text-align: right;
      color: #3d4558;
      font-size: 14px;
      line-height: 1.5;
    }
    .kicker {
      margin-top: 30px;
      color: #6a4dff;
      font-weight: 800;
      letter-spacing: 2.5px;
      text-transform: uppercase;
      font-size: 15px;
      position: relative;
      z-index: 2;
    }
    h1 {
      margin: 10px 0 8px;
      font-size: 42px;
      line-height: 1.05;
      color: #11152a;
      letter-spacing: -0.8px;
      position: relative;
      z-index: 2;
    }
    .subtitle {
      color: #465169;
      font-size: 20px;
      line-height: 1.35;
      margin-bottom: 28px;
      position: relative;
      z-index: 2;
    }
    .body { position: relative; z-index: 2; }
    pre {
      margin: 0;
      white-space: pre-wrap;
      background: #071122;
      color: #d7e3ff;
      border-radius: 18px;
      padding: 26px 28px;
      font: 21px/1.32 "SFMono-Regular", Consolas, Menlo, monospace;
      border: 1px solid #1f3a5b;
      max-height: 590px;
      overflow: hidden;
    }
    .repo {
      background: linear-gradient(135deg, #101440, #1b4fbc);
      color: #ffffff;
      border-radius: 22px;
      padding: 34px 38px;
      font-size: 28px;
      line-height: 1.35;
      box-shadow: inset 0 0 0 1px rgba(255,255,255,0.12);
    }
    .repo code {
      display: block;
      margin-top: 22px;
      font: 26px/1.25 "SFMono-Regular", Consolas, Menlo, monospace;
      color: #2fdfff;
      overflow-wrap: anywhere;
    }
    .note { margin-top: 22px; color: #263044; font-size: 21px; line-height: 1.5; }
    .repo .note { color: rgba(255, 255, 255, 0.82); }
    .repo .note strong { color: #ffffff; }
  </style>
</head>
<body>
  <div class="card">
    <div class="header">
      <img class="logo" src="${logoUrl}" alt="SYNTRIASS logo" />
      <div>
        <div class="brand">SYNTRIASS LABS</div>
        <div class="tagline">Replacing Digital Laws. Rebuilding Civilization.</div>
      </div>
      <div class="meta">
        <strong>Syntriass Labs Private Limited</strong><br />
        CIN: U62011AP2025PTC120239<br />
        kattanaga5555@gmail.com | +91 88864 68060
      </div>
    </div>
    <div class="kicker">CAUSALUX Contested Sync Evidence Screenshot</div>
    <h1>${esc(title)}</h1>
    <div class="subtitle">${esc(subtitle)}</div>
    <div class="body">${kind === "repo" ? `<div class="repo">${body}</div>` : `<pre>${esc(body)}</pre>`}</div>
  </div>
</body>
</html>`;
}

function render(name, spec) {
  const htmlPath = path.join(OUT_DIR, `${name}.html`);
  const pngPath = path.join(OUT_DIR, `${name}.png`);
  const jpgPath = path.join(OUT_DIR, `${name}.jpg`);
  fs.writeFileSync(htmlPath, pageHtml(spec));
  execFileSync(CHROME, [
    "--headless=new",
    "--disable-gpu",
    "--no-first-run",
    "--no-default-browser-check",
    "--window-size=1440,960",
    "--screenshot=" + pngPath,
    pathToFileURL(htmlPath).href
  ], { stdio: "inherit" });
  execFileSync("sips", ["-Z", "920", "-s", "format", "jpeg", "-s", "formatOptions", "42", pngPath, "--out", jpgPath], { stdio: "inherit" });
  console.log(`Wrote ${jpgPath}`);
}

const shots = [
  ["01_github_repository", {
    title: "Public Repository Reference",
    subtitle: "GitHub repository link included for evaluator credibility and source traceability.",
    kind: "repo",
    body: `Repository URL<code>${REPO_URL}</code><div class="note">Evidence package references <strong>CAUSALUX causal sync</strong>, <strong>CRDT merge</strong>, <strong>USO state objects</strong>, <strong>VECTRA compression</strong>, and generated artifacts from this repository.</div>`
  }],
  ["02_clean_test_output", {
    title: "Selected Test Output",
    subtitle: "Fresh local selected evidence run across CAUSALUX, nexus-sync, nexus-pcu USO, and nexus-compress.",
    kind: "code",
    body: testSummary()
  }],
  ["03_full_run_caveats", {
    title: "Full-Run Caveats",
    subtitle: "Broader test run findings are documented so reviewers can separate passing subsystem tests from stale integration gaps.",
    kind: "code",
    body: caveatSummary()
  }],
  ["04_version_vector", {
    title: "Version Vector Causality",
    subtitle: "Version vectors track happens-before ordering, conflict detection, and deterministic merge context.",
    kind: "code",
    body: excerpt([{ file: "causalux/src/version_vector.rs", start: 28, end: 94 }, { file: "causalux/src/version_vector.rs", start: 100, end: 166 }])
  }],
  ["05_sync_protocol", {
    title: "Hierarchical Sync Protocol",
    subtitle: "Sync requests carry node ID, version vector, latest snapshot ID, and Merkle-root context.",
    kind: "code",
    body: excerpt([{ file: "causalux/src/sync.rs", start: 10, end: 56 }, { file: "causalux/src/sync.rs", start: 80, end: 160 }])
  }],
  ["06_adaptive_sync_savings", {
    title: "Adaptive Sync and Bandwidth Savings",
    subtitle: "Strategy selection changes for long partitions, and tests check savings calculation.",
    kind: "code",
    body: excerpt([{ file: "causalux/src/sync.rs", start: 220, end: 289 }, { file: "causalux/src/sync.rs", start: 321, end: 353 }])
  }],
  ["07_rga_text_crdt", {
    title: "RGA Text CRDT",
    subtitle: "Replicated growable array text state supports remote inserts and deterministic ordering.",
    kind: "code",
    body: excerpt([{ file: "causalux/src/crdt.rs", start: 13, end: 126 }])
  }],
  ["08_crdt_merge_types", {
    title: "Counter, Set, and Map CRDTs",
    subtitle: "Grow-only counter, PN counter, observed-remove set, and last-writer map are implemented.",
    kind: "code",
    body: excerpt([{ file: "causalux/src/crdt.rs", start: 172, end: 240 }, { file: "causalux/src/crdt.rs", start: 287, end: 431 }])
  }],
  ["09_crdt_document_tests", {
    title: "CRDT Convergence Tests",
    subtitle: "Tests exercise concurrent inserts, counter merge, set behavior, map merge, and document convergence.",
    kind: "code",
    body: excerpt([{ file: "causalux/src/crdt.rs", start: 525, end: 620 }])
  }],
  ["10_causal_dag_insert", {
    title: "Causal DAG Insert Path",
    subtitle: "The DAG checks idempotence, dependencies, conflicts, version-vector merge, state application, and snapshot triggers.",
    kind: "code",
    body: excerpt([{ file: "causalux/src/dag.rs", start: 38, end: 167 }])
  }],
  ["11_causal_dag_order_tests", {
    title: "DAG Ordering and Dependency Tests",
    subtitle: "Tests cover insert, causal ordering, and missing dependency rejection.",
    kind: "code",
    body: excerpt([{ file: "causalux/src/dag.rs", start: 336, end: 452 }])
  }],
  ["12_runtime_sync_tests", {
    title: "Runtime Disconnected Merge Tests",
    subtitle: "Software tests simulate document creation, collaborative editing, distributed counter, and set synchronization.",
    kind: "code",
    body: excerpt([{ file: "causalux/src/runtime.rs", start: 559, end: 679 }])
  }],
  ["13_snapshot_compression", {
    title: "Snapshots and Compression",
    subtitle: "Snapshots carry Merkle root, version vector, operation count, compressed size, and common-snapshot negotiation.",
    kind: "code",
    body: excerpt([{ file: "causalux/src/snapshot.rs", start: 13, end: 100 }, { file: "causalux/src/snapshot.rs", start: 206, end: 324 }])
  }],
  ["14_sovereign_envelope", {
    title: "Encrypted Operation Envelope",
    subtitle: "Operation envelope carries routing metadata while protecting operation contents through authenticated encryption.",
    kind: "code",
    body: excerpt([{ file: "causalux/src/envelope.rs", start: 164, end: 278 }])
  }],
  ["15_envelope_tests", {
    title: "Envelope Access Tests",
    subtitle: "Tests cover key derivation, seal/unseal, wrong-key rejection, and key revocation.",
    kind: "code",
    body: excerpt([{ file: "causalux/src/envelope.rs", start: 345, end: 401 }])
  }],
  ["16_crdt_uso_model", {
    title: "CRDT-Backed USO Model",
    subtitle: "USO types map raw, JSON, counters, sets, maps, and text to merge-specific CRDT state.",
    kind: "code",
    body: excerpt([{ file: "nexus-sync/src/crdt_uso.rs", start: 9, end: 65 }, { file: "nexus-sync/src/crdt_uso.rs", start: 247, end: 282 }])
  }],
  ["17_crdt_uso_tests", {
    title: "CRDT USO Tests",
    subtitle: "Tests cover counter, PN counter, set, text, and counter merge behavior.",
    kind: "code",
    body: excerpt([{ file: "nexus-sync/src/crdt_uso.rs", start: 303, end: 370 }])
  }],
  ["18_nexus_sync_engine", {
    title: "NEXUS Sync Engine",
    subtitle: "The sync engine wraps CAUSALUX DAG with USO registry, signed operations, sync deltas, and remote merge.",
    kind: "code",
    body: excerpt([{ file: "nexus-sync/src/sync_engine.rs", start: 11, end: 160 }])
  }],
  ["19_sync_engine_tests", {
    title: "Sync Engine Tests",
    subtitle: "Library tests cover engine creation, USO registration, and USO update with signed causal operation creation.",
    kind: "code",
    body: excerpt([{ file: "nexus-sync/src/sync_engine.rs", start: 191, end: 237 }])
  }],
  ["20_uso_policy_history", {
    title: "USO Sync Policy and Causal History",
    subtitle: "Universal State Objects carry sync policy, access policy, vector-clock history, and operation log.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/uso.rs", start: 79, end: 183 }, { file: "nexus-pcu/src/uso.rs", start: 225, end: 312 }])
  }],
  ["21_uso_merge_tests", {
    title: "USO Merge and Serialization Tests",
    subtitle: "Tests cover update, merge, serialization, sync policy, and happens-before logic.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/uso.rs", start: 391, end: 445 }, { file: "nexus-pcu/src/uso.rs", start: 483, end: 545 }])
  }],
  ["22_pcu_compression", {
    title: "PCU Compression Path",
    subtitle: "VECTRA compression wraps PCU input data, keeps hashes and statistics, and supports decompression.",
    kind: "code",
    body: excerpt([{ file: "nexus-compress/src/pcu_compress.rs", start: 9, end: 81 }, { file: "nexus-compress/src/pcu_compress.rs", start: 83, end: 177 }])
  }],
  ["23_uso_compression", {
    title: "USO Compression Path",
    subtitle: "Compressed USO packets retain access policy, sync policy, lamport timestamp, ratio, and batch statistics.",
    kind: "code",
    body: excerpt([{ file: "nexus-compress/src/uso_compress.rs", start: 10, end: 120 }, { file: "nexus-compress/src/uso_compress.rs", start: 122, end: 151 }])
  }]
];

for (const [name, spec] of shots) {
  render(name, spec);
}
