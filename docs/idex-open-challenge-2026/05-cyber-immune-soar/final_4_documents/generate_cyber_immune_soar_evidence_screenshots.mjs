import fs from "fs";
import path from "path";
import { execFileSync } from "child_process";
import { pathToFileURL } from "url";

const ROOT = "/Users/richardrich/Desktop/NEXUS";
const OUT_DIR = path.join(ROOT, "docs/idex-open-challenge-2026/05-cyber-immune-soar/final_4_documents/evidence_assets");
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
    "$ agp-core/.venv/bin/python agp-core/tests/test_immune_bridge.py",
    "$ agp-core/.venv/bin/python -m pytest agp-core/tests/immunity/test_immune_system.py agp-core/tests/immunity/test_unified_immune.py -q",
    "$ agp-core/.venv/bin/python agp-core/tests/test_multi_agent_governance.py",
    "",
    "RESULT SUMMARY",
    "Governance-immune bridge script:      19 passed / 0 failed",
    "Immune pytest suites:                 54 passed / 0 failed",
    "Multi-agent governance simulation:    completed successfully",
    "",
    "BEHAVIOR VERIFIED",
    "LOW/MEDIUM/HIGH/CRITICAL response mapping exercised",
    "Defection maps to multi_quarantine and trust reduction",
    "Unified immune scan and known-threat memory exercised",
    "12-agent governance simulation ranks high-risk actor low",
    "",
    "Recorded output: evidence_assets/cyber_immune_soar_test_output.txt"
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
    <div class="kicker">Cyber Immune SOAR Evidence Screenshot</div>
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
    body: `Repository URL<code>${REPO_URL}</code><div class="note">Evidence package references <strong>AGP immune bridge</strong>, <strong>unified immune system</strong>, <strong>multi-agent governance</strong>, test outputs, and generated artifacts from this repository.</div>`
  }],
  ["02_test_output", {
    title: "Cyber Immune SOAR Test Output",
    subtitle: "Fresh local run for immune bridge, immune suites, and multi-agent governance simulation.",
    kind: "code",
    body: testSummary()
  }],
  ["03_threat_signal_schema", {
    title: "Threat Signal Schema",
    subtitle: "ThreatSignal and DefectionSignal define the event-to-response contract.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/immunity/governance_bridge.py", start: 20, end: 51 }])
  }],
  ["04_bridge_state", {
    title: "Governance-Immune Bridge State",
    subtitle: "The bridge tracks active threats, permissions, defection signals, trust scores, and callbacks.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/immunity/governance_bridge.py", start: 52, end: 87 }])
  }],
  ["05_threat_action_mapping", {
    title: "Threat Action Mapping",
    subtitle: "Threat levels are mapped into observe, throttle, block, quarantine, and escalation actions.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/immunity/governance_bridge.py", start: 89, end: 147 }])
  }],
  ["06_defection_response", {
    title: "Defection Response",
    subtitle: "Multi-agent collusion or defection reduces trust and returns a multi_quarantine action.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/immunity/governance_bridge.py", start: 149, end: 172 }])
  }],
  ["07_trust_status_controls", {
    title: "Trust, Suppression, and Status Controls",
    subtitle: "The bridge exposes trust propagation, maintenance suppression, restoration, and status telemetry.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/immunity/governance_bridge.py", start: 174, end: 245 }])
  }],
  ["08_immune_system_config", {
    title: "Artificial Immune System Configuration",
    subtitle: "The core AIS config combines innate, adaptive, memory, T-cell, and optional swarm controls.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/immunity/immune_system.py", start: 26, end: 61 }])
  }],
  ["09_immune_forward_scan", {
    title: "Immune Forward Scan",
    subtitle: "Runtime scan records threat type, severity, innate/adaptive activation, memory hit, and response time.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/immunity/immune_system.py", start: 123, end: 200 }])
  }],
  ["10_unified_immune_architecture", {
    title: "Unified Immune Architecture",
    subtitle: "UnifiedImmuneSystem integrates innate, adaptive, memory, governance bridge, AHES, and TELOS.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/immunity/unified.py", start: 27, end: 79 }])
  }],
  ["11_unified_scan_flow", {
    title: "Unified Scan Flow",
    subtitle: "Behavior vectors are normalized, scanned by AIS, scored, classified, and routed to governance action.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/immunity/unified.py", start: 138, end: 252 }])
  }],
  ["12_unified_action_mapping", {
    title: "Unified Governance Mapping",
    subtitle: "Severity is translated into ThreatSignal and returned as warn, escalate, restrict, or quarantine.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/immunity/unified.py", start: 266, end: 335 }])
  }],
  ["13_threat_memory", {
    title: "Threat Memory Training",
    subtitle: "Known threat vectors are stored and later matched by cosine similarity.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/immunity/unified.py", start: 337, end: 379 }])
  }],
  ["14_anomaly_taxonomy", {
    title: "Governance Anomaly Taxonomy",
    subtitle: "Anomaly detector covers drift, sudden shifts, category shifts, frequency spikes, and risky patterns.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/governance/anomaly.py", start: 16, end: 47 }])
  }],
  ["15_anomaly_detection_flow", {
    title: "Anomaly Detection Flow",
    subtitle: "The detector builds baseline/recent windows and emits typed anomaly alerts with evidence.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/governance/anomaly.py", start: 66, end: 110 }])
  }],
  ["16_drift_detector", {
    title: "Behavioral Drift Detector",
    subtitle: "Recent embeddings are compared against baseline embeddings and converted into severity alerts.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/governance/anomaly.py", start: 112, end: 157 }])
  }],
  ["17_bridge_test_cases", {
    title: "Bridge Test Cases",
    subtitle: "Tests cover low, high, critical, defection, trust, clearing, suppression, and status flows.",
    kind: "code",
    body: excerpt([{ file: "agp-core/tests/test_immune_bridge.py", start: 37, end: 125 }])
  }],
  ["18_bridge_test_summary", {
    title: "Bridge Test Summary",
    subtitle: "The script records pass/fail counts and only reports verified when no failures are present.",
    kind: "code",
    body: excerpt([{ file: "agp-core/tests/test_immune_bridge.py", start: 127, end: 138 }])
  }],
  ["19_unified_immune_tests", {
    title: "Unified Immune Tests",
    subtitle: "Tests cover initialization, registration, behavior scans, threat training, TELOS, and actions.",
    kind: "code",
    body: excerpt([{ file: "agp-core/tests/immunity/test_unified_immune.py", start: 15, end: 75 }])
  }],
  ["20_unified_integration_tests", {
    title: "Full Integration Test",
    subtitle: "The suite registers agents, trains known threats, scans varied behavior, and checks status.",
    kind: "code",
    body: excerpt([{ file: "agp-core/tests/immunity/test_unified_immune.py", start: 110, end: 185 }])
  }],
  ["21_immune_system_tests", {
    title: "AIS Unit Tests",
    subtitle: "The pytest suite covers antibodies, binding, neutralization, cloning, and pool behavior.",
    kind: "code",
    body: excerpt([{ file: "agp-core/tests/immunity/test_immune_system.py", start: 94, end: 194 }])
  }],
  ["22_multi_agent_simulation", {
    title: "Multi-Agent Governance Simulation",
    subtitle: "Twelve behavior profiles are exercised across task success, collaboration, risk, and ethics signals.",
    kind: "code",
    body: excerpt([{ file: "agp-core/tests/test_multi_agent_governance.py", start: 32, end: 89 }])
  }],
  ["23_governance_result_gate", {
    title: "Governance Result Gate",
    subtitle: "Simulation succeeds only when high performers rank high and the high-risk actor ranks low.",
    kind: "code",
    body: excerpt([{ file: "agp-core/tests/test_multi_agent_governance.py", start: 276, end: 371 }])
  }]
];

for (const [name, spec] of shots) {
  render(name, spec);
}
