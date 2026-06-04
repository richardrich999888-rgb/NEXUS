import fs from "fs";
import path from "path";
import { execFileSync } from "child_process";
import { pathToFileURL } from "url";

const ROOT = "/Users/richardrich/Desktop/NEXUS";
const OUT_DIR = path.join(ROOT, "docs/idex-open-challenge-2026/01-nexus-guard/final_4_documents/evidence_assets");
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

function tailInterestingTestOutput() {
  const outputPath = path.join(OUT_DIR, "nexus_guard_red_team_test_output.txt");
  const raw = fs.readFileSync(outputPath, "utf8");
  const lines = raw.split("\n");
  const start = Math.max(0, lines.findIndex((line) => line.includes("running 10 tests")));
  return lines.slice(start).join("\n").trim();
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
    <div class="kicker">NEXUS Guard Evidence Screenshot</div>
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
  body: `Repository URL<code>${REPO_URL}</code><div class="note">Local remote verified using <strong>git remote -v</strong>. Evidence package references source files and test commands from this repository.</div>`
});

render("02_red_team_test_output", {
  title: "Red-Team Execution Tests",
  subtitle: "Fresh local run for NEXUS Guard denial-path evidence.",
  kind: "code",
  body: `$ cargo test -p nexus-executor --test red_team_execution -- --nocapture\n\n${tailInterestingTestOutput()}`
});

render("03_execution_guard_interface", {
  title: "ExecutionGuard Interface",
  subtitle: "Frozen pre-execution authorization contract in nexus-executor/src/guard.rs.",
  kind: "code",
  body: readLines("nexus-executor/src/guard.rs", 1, 28)
});

render("04_composite_first_deny_wins", {
  title: "Composite Guard First-Deny-Wins",
  subtitle: "Layered guard model: any denial blocks execution.",
  kind: "code",
  body: readLines("nexus-executor/src/guards/composite.rs", 35, 51)
});

render("05_executor_guard_gate", {
  title: "Executor Guard Gate",
  subtitle: "Guard check executes before cache lookup and protected execution.",
  kind: "code",
  body: readLines("nexus-executor/src/executor.rs", 145, 166)
});

render("06_no_cache_after_block_test", {
  title: "No Cache After Block Test",
  subtitle: "Red-team test proves blocked execution does not create a reusable cache hit.",
  kind: "code",
  body: readLines("nexus-executor/tests/red_team_execution.rs", 75, 96)
});

render("07_etk_offline_verifier", {
  title: "ETK Offline Verifier",
  subtitle: "Execution Truth Kernel CLI verifies proof, events, policy, and producer key without cloud dependency.",
  kind: "code",
  body: excerpt([
    { file: "etk/crates/etk-cli/src/main.rs", start: 36, end: 44 },
    "...",
    { file: "etk/crates/etk-cli/src/main.rs", start: 120, end: 140 }
  ])
});

render("08_telos_entropy_accounting", {
  title: "TELOS Consequence Accounting",
  subtitle: "Consequence tiers and entropy spend logic gate high-impact actions.",
  kind: "code",
  body: excerpt([
    { file: "agp-core/src/telos/membrane.py", start: 26, end: 32 },
    "...",
    { file: "agp-core/src/telos/membrane.py", start: 83, end: 112 }
  ])
});

render("09_guarded_request_schema", {
  title: "Guarded Request Context",
  subtitle: "ExecutionContext carries identity, limits, request ID, and risk inputs into the guard decision.",
  kind: "code",
  body: readLines("nexus-executor/src/types.rs", 9, 72)
});

render("10_denial_reason_codes", {
  title: "Typed Denial And Proof Errors",
  subtitle: "Blocked execution, identity failure, proof failure, and cache errors are explicit program states.",
  kind: "code",
  body: excerpt([
    { file: "nexus-executor/src/error.rs", start: 45, end: 57 },
    "...",
    { file: "nexus-executor/src/error.rs", start: 181, end: 206 }
  ])
});

render("11_no_success_proof_on_deny", {
  title: "No Success Proof On Deny",
  subtitle: "Integration test verifies blocked execution returns an error rather than a success proof.",
  kind: "code",
  body: readLines("nexus-executor/tests/integration_tests.rs", 330, 356)
});

render("12_no_cache_artifact_on_deny", {
  title: "No Cache Artifact On Deny",
  subtitle: "A second request after a denial remains blocked, showing no successful cache entry was created.",
  kind: "code",
  body: readLines("nexus-executor/tests/red_team_execution.rs", 75, 96)
});

render("13_allowed_execution_audit_record", {
  title: "Allowed Execution Proof Path",
  subtitle: "Only the successful execution branch generates a proof, writes cache, and returns ExecutionResponse.",
  kind: "code",
  body: readLines("nexus-executor/src/executor.rs", 258, 277)
});

render("14_unauthorized_action_demo", {
  title: "Unauthorized Flood Demo",
  subtitle: "Repeated unauthorized requests remain blocked under the guard.",
  kind: "code",
  body: readLines("nexus-executor/tests/red_team_execution.rs", 22, 42)
});

render("15_authorized_action_demo", {
  title: "Authorized Baseline Demo",
  subtitle: "A signed identity can execute only when no guard constraint is installed; production uses a guarded builder.",
  kind: "code",
  body: readLines("nexus-executor/tests/red_team_execution.rs", 44, 73)
});

render("16_replay_attempt_demo", {
  title: "Replay And Cache Behavior",
  subtitle: "Adversarial test documents repeated-PCU behavior and cache-path expectations.",
  kind: "code",
  body: readLines("nexus-executor/tests/adversarial.rs", 171, 186)
});

render("17_policy_bypass_demo", {
  title: "Malformed WASM Bypass Attempt",
  subtitle: "Guard runs before WASM validation, so malformed code cannot bypass the choke point.",
  kind: "code",
  body: readLines("nexus-executor/tests/red_team_execution.rs", 155, 180)
});

render("18_consequence_budget_demo", {
  title: "Consequence Budget Demo",
  subtitle: "TELOS entropy meter spends consequence-scaled budget and denies when insufficient.",
  kind: "code",
  body: readLines("agp-core/src/telos/membrane.py", 101, 120)
});

render("19_console_summary_view", {
  title: "Evidence Console Summary",
  subtitle: "Current NEXUS Guard software evidence state prepared for portal upload.",
  kind: "code",
  body: `NEXUS Guard Evidence Summary\n\nRepository: ${REPO_URL}\nTest command: cargo test -p nexus-executor --test red_team_execution -- --nocapture\nResult: 10 passed; 0 failed\nPrimary denial property: first-deny-wins guard chain\nNo-success-proof-on-deny evidence: integration_tests.rs\nNo-cache-after-deny evidence: red_team_execution.rs\nOffline audit path: ETK verify CLI\nReadiness language: software subsystem TRL 3-4; hardware-in-loop proposed`
});

render("20_execution_proof_schema", {
  title: "Execution Proof Schema",
  subtitle: "Allowed executions carry PCU hash, input hashes, output hash, identity hash, node ID, and attestation.",
  kind: "code",
  body: readLines("nexus-executor/src/proof.rs", 90, 178)
});

render("21_offline_review_workflow", {
  title: "Offline Review Workflow",
  subtitle: "ETK README documents local verify command, evidence files, and regulator-grade build checks.",
  kind: "code",
  body: readLines("etk/README.md", 46, 83)
});

render("22_deployment_profile", {
  title: "Executor Deployment Profile",
  subtitle: "Release profile and hardware attestation feature flags are explicit in nexus-executor.",
  kind: "code",
  body: excerpt([
    { file: "nexus-executor/Cargo.toml", start: 92, end: 103 },
    "...",
    { file: "nexus-executor/src/proof.rs", start: 56, end: 77 }
  ])
});

render("23_api_contract", {
  title: "Protected Execution API Contract",
  subtitle: "Library exports the executor, guard, context, result, proof, and host interface used by integrators.",
  kind: "code",
  body: excerpt([
    { file: "nexus-executor/src/lib.rs", start: 84, end: 102 },
    "...",
    { file: "nexus-executor/src/lib.rs", start: 113, end: 132 }
  ])
});
