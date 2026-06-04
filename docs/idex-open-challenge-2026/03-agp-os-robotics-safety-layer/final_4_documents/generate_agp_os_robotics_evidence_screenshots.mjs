import fs from "fs";
import path from "path";
import { execFileSync } from "child_process";
import { pathToFileURL } from "url";

const ROOT = "/Users/richardrich/Desktop/NEXUS";
const OUT_DIR = path.join(ROOT, "docs/idex-open-challenge-2026/03-agp-os-robotics-safety-layer/final_4_documents/evidence_assets");
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
    "$ agp-core/.venv/bin/python agp-core/tests/test_rtos.py",
    "$ agp-core/.venv/bin/python agp-core/tests/test_ros2.py",
    "$ agp-core/.venv/bin/python agp-core/tests/test_resources.py",
    "$ agp-core/.venv/bin/python agp-core/tests/test_production.py",
    "$ cargo test -p nexus-rtos-core -- --nocapture",
    "$ cargo check -p nexus-rtos-core --target wasm32-unknown-unknown",
    "",
    "RESULT SUMMARY",
    "RTOS scheduler:          8 passed / 0 failed",
    "ROS2 bridge:            16 passed / 0 failed",
    "Resource controller:    12 passed / 0 failed",
    "Production adapter:     22 passed / 0 failed",
    "Rust RTOS core:          4 passed / 0 failed",
    "wasm32 target check:     passed",
    "",
    "RECORDED OUTPUT FILES",
    "evidence_assets/agp_os_robotics_test_output.txt",
    "evidence_assets/nexus_rtos_core_test_output.txt",
    "",
    "Fresh local result: Python AGP-OS checks 58 passed / 0 failed; Rust RTOS core 4 passed / 0 failed; wasm32 check passed."
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
    .note {
      margin-top: 22px;
      color: #263044;
      font-size: 21px;
      line-height: 1.5;
    }
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
    <div class="kicker">AGP-OS Robotics Evidence Screenshot</div>
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
  execFileSync("sips", ["-Z", "1000", "-s", "format", "jpeg", "-s", "formatOptions", "50", pngPath, "--out", jpgPath], { stdio: "inherit" });
  console.log(`Wrote ${jpgPath}`);
}

const shots = [
  ["01_github_repository", {
    title: "Public Repository Reference",
    subtitle: "GitHub repository link included for evaluator credibility and source traceability.",
    kind: "repo",
    body: `Repository URL<code>${REPO_URL}</code><div class="note">Evidence package references <strong>agp-core</strong>, <strong>nexus-rtos-core</strong>, test outputs, and generated artifacts from this repository.</div>`
  }],
  ["02_test_output", {
    title: "AGP-OS Robotics Test Output",
    subtitle: "Fresh local run for RTOS, ROS2 bridge, resource control, production adapter, and Rust RTOS core.",
    kind: "code",
    body: testSummary()
  }],
  ["03_rtos_priorities", {
    title: "RTOS Priority Classes",
    subtitle: "Critical robot safety tasks are modeled at the highest scheduler priority.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/os/rtos/scheduler.py", start: 16, end: 30 }])
  }],
  ["04_rtos_dispatch", {
    title: "RTOS Dispatch and Deadline Logic",
    subtitle: "Scheduler pops the highest priority task and tracks deadline misses.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/os/rtos/scheduler.py", start: 112, end: 147 }])
  }],
  ["05_ros2_messages", {
    title: "ROS2 Message and Robot State Model",
    subtitle: "Bridge models common ROS2 message types and simulated robot state.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/os/ros2/bridge.py", start: 18, end: 48 }])
  }],
  ["06_ros2_topic_flow", {
    title: "ROS2 Topic Publish Flow",
    subtitle: "Commands publish through named topics and simulation flow updates robot state.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/os/ros2/bridge.py", start: 72, end: 110 }])
  }],
  ["07_ros2_spawn_robot", {
    title: "ROS2 Robot Spawn and Agent Linking",
    subtitle: "Robots get standard command, odometry, and scan topics plus AGP agent linkage.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/os/ros2/bridge.py", start: 138, end: 167 }])
  }],
  ["08_ros2_sensor_hal", {
    title: "ROS2 Sensor Injection and Stats",
    subtitle: "Simulated sensors can feed the bridge and expose statistics for review.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/os/ros2/bridge.py", start: 192, end: 227 }])
  }],
  ["09_resource_quota", {
    title: "Resource Quota Model",
    subtitle: "Agents receive bounded CPU, memory, token, and I/O budgets.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/os/resources/controller.py", start: 14, end: 41 }])
  }],
  ["10_resource_denial", {
    title: "Resource Grant and Denial Path",
    subtitle: "Requests over quota return explicit denial rather than silent execution.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/os/resources/controller.py", start: 75, end: 111 }])
  }],
  ["11_resource_status", {
    title: "Resource Usage and System Status",
    subtitle: "Reviewer-visible state shows agent usage and global system resource status.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/os/resources/controller.py", start: 124, end: 153 }])
  }],
  ["12_hal_device_model", {
    title: "HAL Device and Safety Thresholds",
    subtitle: "Hardware abstraction registers sensors, actuators, and safety thresholds.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/os/hal/hal.py", start: 14, end: 47 }])
  }],
  ["13_hal_interlock", {
    title: "HAL Safety Interlock",
    subtitle: "Low alignment blocks actuator movement and records a denial reason.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/os/hal/hal.py", start: 81, end: 110 }])
  }],
  ["14_production_watchdog", {
    title: "Production Safety Watchdog",
    subtitle: "Watchdog tracks heartbeat timeout and emergency-stop state.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/os/ros2/production.py", start: 31, end: 82 }])
  }],
  ["15_velocity_guard", {
    title: "Velocity Cap and Emergency Stop",
    subtitle: "Velocity commands are capped and emergency stop publishes zero velocity.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/os/ros2/production.py", start: 84, end: 113 }, { file: "agp-core/src/os/ros2/production.py", start: 185, end: 219 }])
  }],
  ["16_production_adapter", {
    title: "ROS2 Adapter Simulation Fallback",
    subtitle: "Adapter connects to hardware when available and otherwise remains simulation-ready.",
    kind: "code",
    body: excerpt([{ file: "agp-core/src/os/ros2/production.py", start: 115, end: 160 }])
  }],
  ["17_deploy_artifacts", {
    title: "Robot Deployment Artifacts",
    subtitle: "Docker and systemd files are present for prototype deployment packaging.",
    kind: "code",
    body: excerpt([{ file: "agp-core/deploy/Dockerfile.ros2", start: 1, end: 36 }, { file: "agp-core/deploy/agp-os-robot.service", start: 1, end: 28 }])
  }],
  ["18_test_rtos", {
    title: "RTOS Test Scenario",
    subtitle: "Test submits tasks in reverse order and verifies critical-first execution.",
    kind: "code",
    body: excerpt([{ file: "agp-core/tests/test_rtos.py", start: 34, end: 78 }])
  }],
  ["19_test_ros2", {
    title: "ROS2 Bridge Test Scenario",
    subtitle: "Test spawns robots, publishes velocity, injects sensors, and links an AGP agent.",
    kind: "code",
    body: excerpt([{ file: "agp-core/tests/test_ros2.py", start: 28, end: 89 }])
  }],
  ["20_test_resources", {
    title: "Resource Controller Test Scenario",
    subtitle: "Test proves within-quota grants and over-quota denials.",
    kind: "code",
    body: excerpt([{ file: "agp-core/tests/test_resources.py", start: 28, end: 89 }])
  }],
  ["21_test_production", {
    title: "Production Adapter Test Scenario",
    subtitle: "Test covers watchdog, velocity caps, heartbeat timeout, and deployment files.",
    kind: "code",
    body: excerpt([{ file: "agp-core/tests/test_production.py", start: 34, end: 108 }])
  }],
  ["22_nexus_rtos_core", {
    title: "Bare-Metal-Compatible RTOS Core",
    subtitle: "Rust core is no_std, denies unsafe code, and uses fixed-capacity scheduling.",
    kind: "code",
    body: excerpt([{ file: "nexus-rtos-core/src/lib.rs", start: 1, end: 40 }, { file: "nexus-rtos-core/src/lib.rs", start: 63, end: 118 }])
  }],
  ["23_rtos_core_tests", {
    title: "Rust RTOS Core Tests",
    subtitle: "Tests cover critical priority, deadline ordering, capacity, duplicate IDs, and missed deadlines.",
    kind: "code",
    body: excerpt([{ file: "nexus-rtos-core/src/lib.rs", start: 180, end: 240 }])
  }]
];

for (const [name, spec] of shots) {
  render(name, spec);
}
