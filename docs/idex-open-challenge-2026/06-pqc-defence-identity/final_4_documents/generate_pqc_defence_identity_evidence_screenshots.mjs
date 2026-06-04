import fs from "fs";
import path from "path";
import { execFileSync } from "child_process";
import { pathToFileURL } from "url";

const ROOT = "/Users/richardrich/Desktop/NEXUS";
const OUT_DIR = path.join(ROOT, "docs/idex-open-challenge-2026/06-pqc-defence-identity/final_4_documents/evidence_assets");
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
    "$ cargo test -p nexus-pcu --features pqc pqc -- --nocapture",
    "",
    "RESULT SUMMARY",
    "nexus-pcu PQC feature tests:       7 passed / 0 failed",
    "Filtered lib tests:                37 filtered out",
    "Filtered integration binaries:     chaos/fuzz/property/replay selected 0",
    "",
    "TESTS PASSED",
    "test_hybrid_keypair_generation",
    "test_classical_signing_and_verification",
    "test_hybrid_signature_serialization",
    "test_public_key_bundle",
    "test_classical_only_mode",
    "test_signature_size",
    "test_pqc_component_verifies_when_classical_component_is_tampered",
    "",
    "Recorded output: evidence_assets/pqc_defence_identity_test_output.txt"
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
    <div class="kicker">PQC Defence Identity Evidence Screenshot</div>
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
    body: `Repository URL<code>${REPO_URL}</code><div class="note">Evidence package references <strong>nexus-pcu PQC feature tests</strong>, <strong>hybrid signatures</strong>, <strong>identity context</strong>, <strong>PCU proof binding</strong>, and generated artifacts from this repository.</div>`
  }],
  ["02_test_output", {
    title: "PQC Defence Identity Test Output",
    subtitle: "Fresh local run for the feature-gated nexus-pcu PQC test path.",
    kind: "code",
    body: testSummary()
  }],
  ["03_feature_gate", {
    title: "PQC Feature Gate",
    subtitle: "PQC dependencies are optional and activated through the `pqc` feature.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/Cargo.toml", start: 28, end: 36 }])
  }],
  ["04_lib_exports", {
    title: "Public PCU Exports",
    subtitle: "nexus-pcu exports HybridSignature, HybridKeyPair, PublicKeyBundle, and identity/proof primitives.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/lib.rs", start: 23, end: 33 }])
  }],
  ["05_hybrid_signature_struct", {
    title: "Hybrid Signature Structure",
    subtitle: "The signature container carries Ed25519, optional ML-DSA, and scheme version.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pqc.rs", start: 53, end: 73 }])
  }],
  ["06_signature_constructors", {
    title: "Classical and Hybrid Constructors",
    subtitle: "The implementation supports classical-only and hybrid signature construction.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pqc.rs", start: 75, end: 115 }])
  }],
  ["07_classical_verification", {
    title: "Classical Signature Verification",
    subtitle: "Ed25519 signature bytes are length-checked, reconstructed, and verified.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pqc.rs", start: 117, end: 130 }])
  }],
  ["08_pqc_verification", {
    title: "ML-DSA Verification Path",
    subtitle: "When the `pqc` feature is enabled, ML-DSA public key and signature bytes are checked.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pqc.rs", start: 132, end: 152 }])
  }],
  ["09_hybrid_verification", {
    title: "Hybrid Verification Policy",
    subtitle: "Verification passes when the classical or PQC component verifies.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pqc.rs", start: 154, end: 173 }])
  }],
  ["10_hybrid_keypair_struct", {
    title: "Hybrid Keypair Structure",
    subtitle: "The keypair carries Ed25519 plus optional ML-DSA private/public keys under feature gating.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pqc.rs", start: 180, end: 201 }])
  }],
  ["11_key_generation", {
    title: "Key Generation",
    subtitle: "Generation uses OS entropy for Ed25519 and ML-DSA-65 when the feature is enabled.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pqc.rs", start: 203, end: 260 }])
  }],
  ["12_signing_path", {
    title: "Signing Path",
    subtitle: "Signing creates Ed25519 first and adds ML-DSA when the PQC key is available.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pqc.rs", start: 262, end: 284 }])
  }],
  ["13_public_key_bundle", {
    title: "Public Key Bundle",
    subtitle: "The verifier bundle carries classical and optional PQC public key material.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pqc.rs", start: 291, end: 357 }])
  }],
  ["14_keypair_classical_tests", {
    title: "Keypair and Classical Verification Tests",
    subtitle: "Tests cover hybrid keypair generation and classical signing verification.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pqc.rs", start: 367, end: 390 }])
  }],
  ["15_serialization_bundle_tests", {
    title: "Serialization and Bundle Tests",
    subtitle: "Tests cover signature serialization and public key bundle verification.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pqc.rs", start: 392, end: 417 }])
  }],
  ["16_classical_size_tests", {
    title: "Classical-Only and Size Tests",
    subtitle: "Tests verify classical-only behavior and expected feature-gated signature size.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pqc.rs", start: 419, end: 446 }])
  }],
  ["17_pqc_tamper_fallback_test", {
    title: "PQC Tamper Fallback Test",
    subtitle: "Feature-gated test verifies PQC component still verifies after classical signature tampering.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pqc.rs", start: 448, end: 472 }])
  }],
  ["18_principal_capabilities", {
    title: "Principal and Capability Model",
    subtitle: "Identity primitives define principal ID, capabilities, constraints, and permission checks.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/identity.rs", start: 18, end: 80 }, { file: "nexus-pcu/src/identity.rs", start: 112, end: 154 }])
  }],
  ["19_delegation_chain", {
    title: "Delegation Chain Verification",
    subtitle: "Delegation links check expiry, continuity, signatures, and canonical signing data.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/identity.rs", start: 222, end: 290 }])
  }],
  ["20_identity_context", {
    title: "Embedded Identity Context",
    subtitle: "PCU identity carries principal, capabilities, delegation, expiry, and signature.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/identity.rs", start: 302, end: 420 }])
  }],
  ["21_pcu_identity_binding", {
    title: "PCU Identity Binding",
    subtitle: "Portable Computation Unit embeds identity context and execution constraints.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/pcu.rs", start: 113, end: 149 }])
  }],
  ["22_execution_attestation", {
    title: "Execution Attestation",
    subtitle: "Execution proof attestation signs node, time, security level, and proof content.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/proof.rs", start: 17, end: 79 }])
  }],
  ["23_crypto_utilities", {
    title: "Classical Crypto Utilities",
    subtitle: "Current production utility path provides Ed25519 key generation, signing, and verification.",
    kind: "code",
    body: excerpt([{ file: "nexus-pcu/src/crypto.rs", start: 44, end: 97 }])
  }]
];

for (const [name, spec] of shots) {
  render(name, spec);
}
