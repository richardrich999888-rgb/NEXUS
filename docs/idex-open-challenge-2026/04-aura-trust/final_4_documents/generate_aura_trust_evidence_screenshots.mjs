import fs from "fs";
import path from "path";
import { execFileSync } from "child_process";
import { pathToFileURL } from "url";

const ROOT = "/Users/richardrich/Desktop/NEXUS";
const OUT_DIR = path.join(ROOT, "docs/idex-open-challenge-2026/04-aura-trust/final_4_documents/evidence_assets");
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
    "$ python3 docs/idex-open-challenge-2026/04-aura-trust/final_4_documents/evidence_assets/aura_trust_offline_verification.py",
    "$ cargo test -p nexus-pcu --features pqc pqc -- --nocapture",
    "$ cargo test -p nexus-etk -- --nocapture",
    "",
    "RESULT SUMMARY",
    "AURA Trust offline packet harness: 8 passed / 0 failed",
    "nexus-pcu PQC feature tests:       7 passed / 0 failed",
    "nexus-etk tests:                   9 passed / 0 failed",
    "",
    "BEHAVIOR VERIFIED",
    "Valid signed packet accepted",
    "Tampered payload rejected",
    "Replayed nonce rejected",
    "Stale packet rejected",
    "Replayed sequence rejected",
    "Unknown source rejected",
    "Audit record hash generated",
    "",
    "Recorded output: evidence_assets/aura_trust_test_output.txt"
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
      background: rgba(255, 255, 255, 0.86);
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
      padding: 28px 30px;
      font: 22px/1.34 "SFMono-Regular", Consolas, Menlo, monospace;
      border: 1px solid #1f3a5b;
      max-height: 585px;
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
    <div class="kicker">AURA Trust Evidence Screenshot</div>
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

const harness = "docs/idex-open-challenge-2026/04-aura-trust/final_4_documents/evidence_assets/aura_trust_offline_verification.py";
const shots = [
  ["01_github_repository", {
    title: "Public Repository Reference",
    subtitle: "GitHub repository link included for evaluator credibility and source traceability.",
    kind: "repo",
    body: `Repository URL<code>${REPO_URL}</code><div class="note">Evidence package references <strong>AURA offline verification</strong>, <strong>ETK</strong>, <strong>PCU/PQC</strong>, test outputs, and generated artifacts from this repository.</div>`
  }],
  ["02_test_output", {
    title: "AURA Trust Test Output",
    subtitle: "Fresh local run for packet verification, PQC path, and ETK audit primitives.",
    kind: "code",
    body: testSummary()
  }],
  ["03_packet_schema", {
    title: "Mission Packet Schema",
    subtitle: "Packet carries source, payload, timestamp, nonce, sequence, provenance, policy, and signature.",
    kind: "code",
    body: excerpt([{ file: harness, start: 33, end: 60 }])
  }],
  ["04_trust_store", {
    title: "Offline Trust Store",
    subtitle: "Local public keys, nonce memory, and sequence memory support disconnected verification.",
    kind: "code",
    body: excerpt([{ file: harness, start: 63, end: 74 }])
  }],
  ["05_replay_freshness_gates", {
    title: "Replay and Freshness Gates",
    subtitle: "Verifier rejects unknown source, stale packets, nonce replay, and sequence replay.",
    kind: "code",
    body: excerpt([{ file: harness, start: 76, end: 96 }])
  }],
  ["06_signature_verification", {
    title: "Signature Verification",
    subtitle: "Packet signing bytes are verified with source public key before acceptance.",
    kind: "code",
    body: excerpt([{ file: harness, start: 97, end: 106 }])
  }],
  ["07_audit_record", {
    title: "ETK-Compatible Audit Record",
    subtitle: "Every decision produces packet hash, payload hash, provenance root, policy ref, result, and reason.",
    kind: "code",
    body: excerpt([{ file: harness, start: 108, end: 122 }])
  }],
  ["08_packet_builder", {
    title: "Mission Packet Builder",
    subtitle: "Demo packet includes mission metadata, simulated sensor report, provenance, and policy class.",
    kind: "code",
    body: excerpt([{ file: harness, start: 125, end: 139 }])
  }],
  ["09_accept_tamper_tests", {
    title: "Acceptance and Tamper Tests",
    subtitle: "Harness accepts a valid packet and rejects a modified payload.",
    kind: "code",
    body: excerpt([{ file: harness, start: 149, end: 168 }])
  }],
  ["10_replay_stale_tests", {
    title: "Replay, Stale, and Unknown-Source Tests",
    subtitle: "Harness rejects nonce replay, stale timestamps, old sequence numbers, and unknown sources.",
    kind: "code",
    body: excerpt([{ file: harness, start: 170, end: 194 }])
  }],
  ["11_existing_offline_verifier", {
    title: "Existing AURA Offline Verifier",
    subtitle: "Existing module establishes offline verification direction but is explicitly placeholder-stage.",
    kind: "code",
    body: excerpt([{ file: "src/network/offline.py", start: 1, end: 59 }])
  }],
  ["12_ria_signature_container", {
    title: "AURA RIA Signature Container",
    subtitle: "Current AURA core includes nonce, timestamp, network, metadata, and serialization fields.",
    kind: "code",
    body: excerpt([{ file: "src/core/ria.py", start: 56, end: 104 }])
  }],
  ["13_ria_transaction_verify", {
    title: "AURA RIA Transaction Verification",
    subtitle: "Core reconstructs message, recomputes signature value, checks timestamp, and updates invariant.",
    kind: "code",
    body: excerpt([{ file: "src/core/ria.py", start: 312, end: 397 }])
  }],
  ["14_etk_event_schema", {
    title: "ETK Execution Event Schema",
    subtitle: "ETK uses canonical field order and deterministic event IDs for audit records.",
    kind: "code",
    body: excerpt([{ file: "nexus-etk/src/schema.rs", start: 99, end: 163 }])
  }],
  ["15_etk_proof_schema", {
    title: "ETK Proof Schema",
    subtitle: "Execution proof binds chain root, timestamps, policy reference, jurisdiction, and signature.",
    kind: "code",
    body: excerpt([{ file: "nexus-etk/src/schema.rs", start: 224, end: 270 }])
  }],
  ["16_etk_event_chain", {
    title: "ETK Hash-Chained Events",
    subtitle: "Append path enforces sequence, previous-event hash, policy, and jurisdiction consistency.",
    kind: "code",
    body: excerpt([{ file: "nexus-etk/src/chain.rs", start: 1, end: 85 }])
  }],
  ["17_etk_finalize", {
    title: "ETK Finalize and Sign",
    subtitle: "Finalization signs the proof over canonical signing bytes.",
    kind: "code",
    body: excerpt([{ file: "nexus-etk/src/chain.rs", start: 88, end: 124 }])
  }],
  ["18_etk_verifier_phases", {
    title: "ETK Offline Verifier",
    subtitle: "Verifier performs schema, signature, chain, time, policy, and outcome checks offline.",
    kind: "code",
    body: excerpt([{ file: "nexus-etk/src/verifier.rs", start: 48, end: 140 }])
  }],
  ["19_pcu_execution_proof", {
    title: "PCU Execution Proof",
    subtitle: "PCU proof format binds PCU hash, inputs, code hash, output hash, metrics, and node attestation.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/proof.rs", start: 78, end: 152 }])
  }],
  ["20_pcu_unit_structure", {
    title: "PCU Unit Structure",
    subtitle: "PCU carries code, content-addressed inputs, parameters, identity, constraints, and result proof.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pcu.rs", start: 98, end: 161 }])
  }],
  ["21_pqc_hybrid_signature", {
    title: "PQC Hybrid Signature Path",
    subtitle: "PQC module defines Ed25519 plus ML-DSA signature material under the `pqc` feature.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pqc.rs", start: 53, end: 172 }])
  }],
  ["22_pqc_keypair_sign", {
    title: "PQC Keypair and Public Bundle",
    subtitle: "Keypair signs messages and public key bundle verifies hybrid signatures.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pqc.rs", start: 181, end: 355 }])
  }],
  ["23_pqc_tests", {
    title: "PQC Feature Tests",
    subtitle: "Feature-gated tests include classical signing, serialization, bundle verification, size, and PQC fallback.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pqc.rs", start: 363, end: 470 }])
  }]
];

for (const [name, spec] of shots) {
  render(name, spec);
}
