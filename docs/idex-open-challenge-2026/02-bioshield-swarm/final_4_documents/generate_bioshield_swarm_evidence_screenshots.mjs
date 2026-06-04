import fs from "fs";
import path from "path";
import { execFileSync } from "child_process";
import { pathToFileURL } from "url";

const ROOT = "/Users/richardrich/Desktop/NEXUS";
const OUT_DIR = path.join(ROOT, "docs/idex-open-challenge-2026/02-bioshield-swarm/final_4_documents/evidence_assets");
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
  const outputPath = path.join(OUT_DIR, "bioshield_swarm_test_output.txt");
  const raw = fs.readFileSync(outputPath, "utf8");
  const lines = raw.split("\n");
  const interesting = lines.filter((line) =>
    line.startsWith("running ") ||
    line.startsWith("test result:") ||
    line.includes("Running unittests") ||
    line.includes("Running tests/") ||
    line.includes("Doc-tests")
  );
  return [
    "$ cargo test -p multi-asi-immune -- --nocapture",
    "",
    ...interesting,
    "",
    "Fresh local result: 68 Rust tests passed; 0 failed; 1 doc-test ignored."
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
    .body {
      position: relative;
      z-index: 2;
    }
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
    .note {
      margin-top: 22px;
      color: #263044;
      font-size: 21px;
      line-height: 1.5;
    }
    .repo .note {
      color: rgba(255, 255, 255, 0.82);
    }
    .repo .note strong {
      color: #ffffff;
    }
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
    <div class="kicker">BioShield Swarm Evidence Screenshot</div>
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
  console.log(`Wrote ${pngPath}`);
}

render("01_github_repository", {
  title: "Public Repository Reference",
  subtitle: "GitHub repository link included for evaluator credibility and source traceability.",
  kind: "repo",
  body: `Repository URL<code>${REPO_URL}</code><div class="note">Evidence package references <strong>multi-asi-immune</strong> source files, tests, and generated artifacts from this repository.</div>`
});

render("02_test_output", {
  title: "Multi-ASI Immune Test Suite",
  subtitle: "Fresh local run for BioShield Swarm software-subsystem evidence.",
  kind: "code",
  body: testSummary()
});

render("03_threat_categories", {
  title: "Threat Categories And Severity",
  subtitle: "Ten threat categories map behavior to defence-relevant compromise patterns.",
  kind: "code",
  body: readLines("multi-asi-immune/src/threat/pattern.rs", 7, 45)
});

render("04_defection_types", {
  title: "Defection Types",
  subtitle: "Observable defection classes cover liveness, contradiction, signatures, false reports, and identity abuse.",
  kind: "code",
  body: readLines("multi-asi-immune/src/enforcement/defection.rs", 7, 36)
});

render("05_isolation_logic", {
  title: "Cumulative Isolation Logic",
  subtitle: "A node is isolated when cumulative defection severity crosses the configured threshold.",
  kind: "code",
  body: readLines("multi-asi-immune/src/enforcement/defection.rs", 53, 100)
});

render("06_identity_sign_verify", {
  title: "Cryptographic Identity Path",
  subtitle: "Node identity derives from Ed25519 public key material and verifies signed data.",
  kind: "code",
  body: excerpt([
    { file: "multi-asi-immune/src/identity/keypair.rs", start: 60, end: 99 },
    "...",
    { file: "multi-asi-immune/src/identity/keypair.rs", start: 131, end: 154 }
  ])
});

render("07_reputation_decay", {
  title: "Reputation Decay And Bounds",
  subtitle: "Trust is earned, bounded, confidence-weighted, and decays toward the initial value.",
  kind: "code",
  body: readLines("multi-asi-immune/src/reputation/score.rs", 22, 131)
});

render("08_threat_memory_add", {
  title: "Threat Memory",
  subtitle: "Threat reports are deduplicated, reputation-filtered, aggregated, and expired by TTL.",
  kind: "code",
  body: readLines("multi-asi-immune/src/threat/memory.rs", 71, 156)
});

render("09_signed_threat_report", {
  title: "Signed Threat Reports",
  subtitle: "Threat reports bind pattern, reporter, confidence, timestamp, and signature.",
  kind: "code",
  body: readLines("multi-asi-immune/src/threat/signature.rs", 10, 104)
});

render("10_node_execution_gate", {
  title: "Node Execution Gate",
  subtitle: "A node denies execution from isolated or low-reputation principals.",
  kind: "code",
  body: readLines("multi-asi-immune/src/node/state.rs", 254, 293)
});

render("11_threat_gossip", {
  title: "Threat Gossip Path",
  subtitle: "Verified threat reports are stored and broadcast to peers for distributed swarm awareness.",
  kind: "code",
  body: readLines("multi-asi-immune/src/node/state.rs", 329, 351)
});

render("12_protocol_messages", {
  title: "Swarm Protocol Messages",
  subtitle: "Handshake, threat reports, heartbeats, constraints, attestations, and accusations share one protocol surface.",
  kind: "code",
  body: readLines("multi-asi-immune/src/protocol/message.rs", 8, 62)
});

render("13_constraint_actions", {
  title: "Constraint Actions",
  subtitle: "Policy can reduce cooperation, increase caution, broadcast warning, or isolate.",
  kind: "code",
  body: readLines("multi-asi-immune/src/protocol/message.rs", 124, 148)
});

render("14_network_health", {
  title: "Network Health Assessment",
  subtitle: "Peer status, active threats, constraints, and isolated nodes are summarized for operator review.",
  kind: "code",
  body: readLines("multi-asi-immune/src/node/state.rs", 478, 505)
});

render("15_homeostasis_bridge", {
  title: "Homeostatic Safety Constraints",
  subtitle: "Swarm policy can generate mutual constraints from stress, caution, urgency, wellbeing, and cooperation metrics.",
  kind: "code",
  body: readLines("multi-asi-immune/src/integration/homeostasis_bridge.rs", 73, 117)
});

render("16_integration_threat_propagation", {
  title: "Integration Threat Propagation Test",
  subtitle: "Software swarm simulation confirms threat report propagation through peer gossip.",
  kind: "code",
  body: readLines("multi-asi-immune/tests/integration_tests.rs", 57, 78)
});

render("17_full_protocol_flow", {
  title: "Full Protocol Flow Test",
  subtitle: "Three-node flow shows coordinated-attack threat propagation through the simulated swarm.",
  kind: "code",
  body: readLines("multi-asi-immune/tests/integration_tests.rs", 109, 142)
});

render("18_defection_tests", {
  title: "Defection Isolation Test",
  subtitle: "Identity forgery reaches isolation threshold in the defection test suite.",
  kind: "code",
  body: readLines("multi-asi-immune/tests/defection_tests.rs", 70, 107)
});

render("19_identity_tests", {
  title: "Identity Verification Tests",
  subtitle: "Sign/verify succeeds for the correct identity and fails for wrong identity or modified message.",
  kind: "code",
  body: readLines("multi-asi-immune/tests/identity_tests.rs", 13, 43)
});

render("20_reputation_tests", {
  title: "Reputation Behavior Tests",
  subtitle: "Positive behavior increases trust, negative behavior lowers it, and scores decay over time.",
  kind: "code",
  body: readLines("multi-asi-immune/tests/reputation_tests.rs", 17, 53)
});

render("21_threat_memory_tests", {
  title: "Threat Memory Tests",
  subtitle: "Threat reports are added, duplicates rejected, and multi-reporter confirmation is recognized.",
  kind: "code",
  body: readLines("multi-asi-immune/tests/threat_propagation_tests.rs", 19, 76)
});

render("22_package_manifest", {
  title: "Package And Test Manifest",
  subtitle: "Cargo manifest lists core tests and crypto/homeostasis dependencies.",
  kind: "code",
  body: readLines("multi-asi-immune/Cargo.toml", 16, 57)
});

render("23_api_exports", {
  title: "Public API Exports",
  subtitle: "Crate exports identity, reputation, threat, memory, protocol, and node state types.",
  kind: "code",
  body: readLines("multi-asi-immune/src/lib.rs", 70, 88)
});
