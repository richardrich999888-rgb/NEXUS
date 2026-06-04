import fs from "fs";
import path from "path";
import { execFileSync } from "child_process";
import { pathToFileURL } from "url";

const ROOT = "/Users/richardrich/Desktop/NEXUS/docs/idex-open-challenge-2026/05-cyber-immune-soar/final_4_documents";
const HTML_DIR = path.join(ROOT, "html");
const LOGO_IMAGE = "/Users/richardrich/WhatsApp Image 2025-12-23 at 16.05.08.jpeg";
const CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";

const COMPANY = {
  legalName: "Syntriass Labs Private Limited",
  brand: "SYNTRIASS LABS",
  tagline: "Replacing Digital Laws. Rebuilding Civilization.",
  cin: "U62011AP2025PTC120239",
  pan: "ABQCS7152R",
  tan: "VPNS31351F",
  office: "12-50, SLV Market, 12 Ward, Dharmavaram, Ananthapur - 515671, Andhra Pradesh, India",
  email: "kattanaga5555@gmail.com",
  phone: "+91 88864 68060"
};

const documents = [
  [
    "01_SYNTRIASS_Cyber_Immune_SOAR_Annexure_1_Applicant_Details_and_Solution_Summary.md",
    "01_SYNTRIASS_Cyber_Immune_SOAR_Annexure_1_Applicant_Details_and_Solution_Summary.pdf",
    "Annexure Outline",
    "Company identification and section outline"
  ],
  [
    "02_SYNTRIASS_Cyber_Immune_SOAR_Annexure_2_Technical_Architecture.md",
    "02_SYNTRIASS_Cyber_Immune_SOAR_Annexure_2_Technical_Architecture.pdf",
    "Annexure-2",
    "Technical architecture and implementation approach"
  ],
  [
    "03_SYNTRIASS_Cyber_Immune_SOAR_Annexure_3_Advantages_and_Competencies.md",
    "03_SYNTRIASS_Cyber_Immune_SOAR_Annexure_3_Advantages_and_Competencies.pdf",
    "Annexure-3",
    "Advantages, capabilities, and competencies"
  ],
  [
    "04_SYNTRIASS_Cyber_Immune_SOAR_Annexure_4_Supporting_Evidence_and_Screenshots.md",
    "04_SYNTRIASS_Cyber_Immune_SOAR_Annexure_4_Supporting_Evidence_and_Screenshots.pdf",
    "Annexure-4",
    "Supporting evidence and screenshots checklist"
  ]
];

fs.mkdirSync(HTML_DIR, { recursive: true });

function esc(value) {
  return String(value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function inline(value) {
  let text = esc(value);
  text = text.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");
  text = text.replace(/`([^`]+)`/g, "<code>$1</code>");
  return text;
}

function parseTable(lines, start) {
  const rows = [];
  let i = start;
  while (i < lines.length && /^\s*\|.*\|\s*$/.test(lines[i])) {
    const raw = lines[i].trim();
    rows.push(raw.slice(1, -1).split("|").map((cell) => cell.trim()));
    i++;
  }
  const hasSeparator = rows.length > 1 && rows[1].every((cell) => /^:?-{3,}:?$/.test(cell));
  const header = rows[0] ?? [];
  const bodyRows = hasSeparator ? rows.slice(2) : rows.slice(1);
  return {
    next: i,
    html: `<table><thead><tr>${header.map((cell) => `<th>${inline(cell)}</th>`).join("")}</tr></thead><tbody>${bodyRows.map((row) => `<tr>${row.map((cell) => `<td>${inline(cell)}</td>`).join("")}</tr>`).join("")}</tbody></table>`
  };
}

function markdownToHtml(markdown) {
  const lines = markdown.replace(/\r\n/g, "\n").split("\n");
  const html = [];
  let i = 0;
  let inList = false;
  let inOrderedList = false;

  function isBlockStart(index) {
    const value = lines[index] ?? "";
    const trimmed = value.trim();
    return !trimmed ||
      trimmed.startsWith("```") ||
      /^(#{1,4})\s+/.test(trimmed) ||
      /^[-*]\s+/.test(trimmed) ||
      /^\d+\.\s+/.test(trimmed) ||
      trimmed === "---" ||
      (/^\s*\|.*\|\s*$/.test(value) && index + 1 < lines.length && /^\s*\|[\s:|-]+\|\s*$/.test(lines[index + 1]));
  }

  function closeLists() {
    if (inList) {
      html.push("</ul>");
      inList = false;
    }
    if (inOrderedList) {
      html.push("</ol>");
      inOrderedList = false;
    }
  }

  while (i < lines.length) {
    const line = lines[i];
    const trimmed = line.trim();
    if (!trimmed) {
      closeLists();
      i++;
      continue;
    }

    if (trimmed === "---") {
      closeLists();
      html.push(`<hr />`);
      i++;
      continue;
    }

    const image = /^!\[([^\]]*)\]\(([^)]+)\)$/.exec(trimmed);
    if (image) {
      closeLists();
      const alt = image[1];
      const target = image[2];
      const imagePath = path.isAbsolute(target) ? target : path.join(ROOT, target);
      const src = pathToFileURL(imagePath).href;
      html.push(`<figure class="evidence-shot"><img src="${src}" alt="${esc(alt)}" /><figcaption>${inline(alt)}</figcaption></figure>`);
      i++;
      continue;
    }

    if (trimmed.startsWith("```")) {
      closeLists();
      const lang = trimmed.slice(3).trim();
      i++;
      const code = [];
      while (i < lines.length && !lines[i].trim().startsWith("```")) {
        code.push(lines[i]);
        i++;
      }
      i++;
      const body = code.join("\n").trim();
      if (lang === "{=typst}" && body === "#pagebreak()") {
        html.push(`<div class="page-break"></div>`);
      } else {
        html.push(`<pre data-lang="${esc(lang)}"><code>${esc(code.join("\n"))}</code></pre>`);
      }
      continue;
    }

    if (/^\s*\|.*\|\s*$/.test(line) && i + 1 < lines.length && /^\s*\|[\s:|-]+\|\s*$/.test(lines[i + 1])) {
      closeLists();
      const table = parseTable(lines, i);
      html.push(table.html);
      i = table.next;
      continue;
    }

    const heading = /^(#{1,4})\s+(.*)$/.exec(trimmed);
    if (heading) {
      closeLists();
      const level = Math.min(heading[1].length, 4);
      html.push(`<h${level}>${inline(heading[2])}</h${level}>`);
      i++;
      continue;
    }

    const bullet = /^[-*]\s+(.*)$/.exec(trimmed);
    if (bullet) {
      if (inOrderedList) {
        html.push("</ol>");
        inOrderedList = false;
      }
      if (!inList) {
        html.push("<ul>");
        inList = true;
      }
      html.push(`<li>${inline(bullet[1])}</li>`);
      i++;
      continue;
    }

    const numbered = /^\d+\.\s+(.*)$/.exec(trimmed);
    if (numbered) {
      if (inList) {
        html.push("</ul>");
        inList = false;
      }
      if (!inOrderedList) {
        html.push("<ol>");
        inOrderedList = true;
      }
      html.push(`<li>${inline(numbered[1])}</li>`);
      i++;
      continue;
    }

    closeLists();
    const paragraph = [trimmed.replace(/\s{2,}$/, "")];
    i++;
    while (i < lines.length && !isBlockStart(i)) {
      paragraph.push(lines[i].trim().replace(/\s{2,}$/, ""));
      i++;
    }
    html.push(`<p>${inline(paragraph.join(" "))}</p>`);
  }
  closeLists();
  return html.join("\n");
}

function buildHtml(markdown, title, subtitle) {
  const logoUrl = pathToFileURL(LOGO_IMAGE).href;
  const content = markdownToHtml(markdown);
  return `<!doctype html>
<html>
<head>
  <meta charset="utf-8" />
  <title>${esc(title)} - ${esc(COMPANY.legalName)}</title>
  <style>
    @page { size: A4; margin: 14mm 14mm 24mm 14mm; }
    * { box-sizing: border-box; }
    html, body { margin: 0; padding: 0; background: #ffffff; color: #172033; font-family: "Aptos", "Calibri", "Arial", sans-serif; }
    body { -webkit-print-color-adjust: exact; print-color-adjust: exact; }
    .brand { display: flex; align-items: center; gap: 4mm; min-width: 90mm; }
    .brand img { width: 24mm; height: 14mm; object-fit: cover; border-radius: 2mm; }
    .brand-name { font-size: 12px; font-weight: 800; letter-spacing: 1.8px; color: #12123f; text-transform: uppercase; }
    .tagline { margin-top: 1mm; font-size: 7.5px; color: #677089; letter-spacing: 0.5px; }
    .cover {
      min-height: 224mm; margin: -12mm -2mm 0 -2mm; padding: 24mm 18mm;
      border: 1px solid #d7dce8; border-radius: 4mm;
      background:
        radial-gradient(circle at 15% 20%, rgba(37, 223, 255, 0.16), transparent 32%),
        radial-gradient(circle at 90% 12%, rgba(216, 43, 241, 0.12), transparent 26%),
        linear-gradient(180deg, #ffffff, #f7f8fc);
      page-break-after: always;
      position: relative;
      overflow: hidden;
    }
    .cover::after {
      content: ""; position: absolute; inset: auto -30mm -45mm auto; width: 105mm; height: 105mm;
      background: url("${logoUrl}") center/cover no-repeat; opacity: 0.045; border-radius: 50%;
    }
    .cover-logo { width: 72mm; height: 42mm; object-fit: cover; border-radius: 4mm; border: 1px solid rgba(18, 18, 63, 0.12); display: block; margin-bottom: 24mm; }
    .kicker { font-size: 10px; text-transform: uppercase; color: #6a4dff; font-weight: 800; letter-spacing: 2px; margin-bottom: 4mm; }
    .cover h1 { margin: 0; font-size: 34px; line-height: 1.04; color: #11152a; letter-spacing: -0.5px; max-width: 155mm; }
    .cover .subtitle { margin-top: 7mm; font-size: 15px; line-height: 1.35; color: #3e475c; max-width: 155mm; }
    .cover-meta { position: absolute; left: 18mm; right: 18mm; bottom: 18mm; border-top: 1px solid #d7dce8; padding-top: 6mm; display: grid; grid-template-columns: 1.1fr 0.9fr; gap: 8mm; font-size: 9px; color: #4d566d; }
    .cover-meta b { color: #11152a; }
    .company-box { margin-top: 8mm; display: grid; grid-template-columns: repeat(3, 1fr); gap: 3mm; }
    .company-box div { border: 1px solid #dfe5f1; border-radius: 2mm; padding: 4mm; background: rgba(255,255,255,0.76); font-size: 8px; color: #4d566d; }
    .company-box b { display: block; color: #11152a; font-size: 9px; margin-bottom: 1mm; }
    main { position: relative; }
    main::before {
      content: ""; position: fixed; right: -28mm; bottom: 8mm; width: 72mm; height: 72mm;
      background: url("${logoUrl}") center/cover no-repeat; opacity: 0.026; border-radius: 50%; pointer-events: none;
    }
    .document-letterhead {
      display: grid; grid-template-columns: 28mm minmax(0, 1fr) 74mm; gap: 6mm;
      align-items: center; min-height: 22mm;
      border-bottom: 1px solid #d7dce8; padding-bottom: 5mm; margin-bottom: 8mm;
      page-break-inside: avoid;
    }
    .doc-logo {
      width: 28mm; height: 16mm; object-fit: cover; border-radius: 2mm;
      border: 1px solid rgba(18, 18, 63, 0.14);
    }
    .doc-brand-title {
      font-size: 12.5px; font-weight: 800; letter-spacing: 1.5px;
      color: #12123f; text-transform: uppercase; line-height: 1.12;
    }
    .doc-brand-subtitle {
      margin-top: 1.2mm; font-size: 7.4px; line-height: 1.3;
      color: #677089; letter-spacing: 0.35px;
    }
    .doc-id-block {
      text-align: right; font-size: 7.1px; line-height: 1.42;
      color: #3d4558; overflow-wrap: anywhere;
    }
    .doc-id-block strong {
      display: block; margin-bottom: 1mm; color: #12123f; font-size: 7.8px;
      text-transform: uppercase; letter-spacing: 0.35px;
    }
    h1, h2, h3, h4, p, ul, ol, table, pre, hr { position: relative; z-index: 1; }
    main > h1:first-of-type { display: none; }
    h1:first-child { margin-top: 0; }
    h1 { color: #11152a; font-size: 24px; line-height: 1.12; margin: 13px 0 10px; page-break-after: avoid; }
    h2 { color: #11152a; font-size: 15.5px; line-height: 1.22; margin: 18px 0 9px; padding-top: 8px; border-top: 2px solid #e6e9f2; page-break-after: avoid; }
    h3 { color: #2858c8; font-size: 12.5px; margin: 14px 0 6px; page-break-after: avoid; }
    h4 { color: #8c2bd9; font-size: 10.5px; margin: 11px 0 5px; page-break-after: avoid; }
    p { color: #263044; font-size: 9.2px; line-height: 1.42; margin: 0 0 7px; orphans: 2; widows: 2; }
    strong { color: #11152a; }
    ul, ol { margin: 4px 0 9px 16px; padding: 0; }
    li { color: #263044; font-size: 9.2px; line-height: 1.38; margin: 2.5px 0; padding-left: 3px; }
    li::marker { color: #2858c8; font-weight: 700; }
    code { font-family: Consolas, Menlo, monospace; color: #1b52bd; background: #eef4ff; padding: 1px 3px; border-radius: 2px; }
    pre { background: #f5f7fb; border: 1px solid #dfe5f1; border-left: 4px solid #2fdfff; color: #263044; padding: 9px 11px; font-family: Consolas, Menlo, monospace; font-size: 7.7px; line-height: 1.28; white-space: pre-wrap; page-break-inside: avoid; margin: 7px 0 11px; }
    .evidence-shot { margin: 8px 0 12px; page-break-inside: avoid; }
    .evidence-shot img { width: 100%; max-height: 158mm; object-fit: contain; border: 1px solid #dfe5f1; border-radius: 3mm; box-shadow: 0 12px 35px rgba(22, 32, 55, 0.10); background: #ffffff; }
    .evidence-shot figcaption { margin-top: 4px; color: #4d566d; font-size: 8px; line-height: 1.3; }
    table { width: 100%; border-collapse: collapse; margin: 7px 0 12px; background: #ffffff; border: 1px solid #dfe5f1; page-break-inside: auto; }
    thead { display: table-header-group; }
    tr { page-break-inside: avoid; }
    th { background: linear-gradient(90deg, #101440, #1b4fbc); color: #ffffff; font-size: 7.6px; text-transform: uppercase; letter-spacing: 0.6px; padding: 6px 7px; border: 1px solid #ccd5e6; text-align: left; vertical-align: middle; }
    td { color: #253044; font-size: 7.8px; line-height: 1.30; padding: 6px 7px; border: 1px solid #dfe5f1; vertical-align: top; overflow-wrap: anywhere; }
    td:first-child { color: #11152a; font-weight: 700; }
    hr { border: 0; border-top: 2px solid #e6e9f2; margin: 12px 0 16px; }
    .page-break { break-after: page; page-break-after: always; height: 0; }
  </style>
</head>
<body>
  <section class="cover">
    <img class="cover-logo" src="${logoUrl}" alt="SYNTRIASS logo" />
    <div class="kicker">iDEX Open Challenge Submission</div>
    <h1>${esc(title)}</h1>
    <div class="subtitle">${esc(subtitle)}</div>
    <div class="company-box">
      <div><b>CIN</b>${esc(COMPANY.cin)}</div>
      <div><b>PAN</b>${esc(COMPANY.pan)}</div>
      <div><b>TAN</b>${esc(COMPANY.tan)}</div>
    </div>
    <div class="cover-meta">
      <div><b>Applicant Entity</b><br />${esc(COMPANY.legalName)}<br />${esc(COMPANY.office)}</div>
      <div><b>Contact</b><br />${esc(COMPANY.email)}<br />${esc(COMPANY.phone)}</div>
    </div>
  </section>
  <main>
    <div class="document-letterhead">
      <img class="doc-logo" src="${logoUrl}" alt="SYNTRIASS logo" />
      <div>
        <div class="doc-brand-title">${esc(COMPANY.brand)}</div>
        <div class="doc-brand-subtitle">${esc(COMPANY.tagline)}</div>
      </div>
      <div class="doc-id-block">
        <strong>${esc(COMPANY.legalName)}</strong><br />
        CIN: ${esc(COMPANY.cin)}<br />
        PAN: ${esc(COMPANY.pan)} &nbsp;|&nbsp; TAN: ${esc(COMPANY.tan)}<br />
        ${esc(COMPANY.email)} &nbsp;|&nbsp; ${esc(COMPANY.phone)}
      </div>
    </div>
    ${content}
  </main>
</body>
</html>`;
}

function chromePdf(htmlPath, pdfPath) {
  execFileSync(CHROME, [
    "--headless=new",
    "--disable-gpu",
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-print-preview",
    "--no-pdf-header-footer",
    "--print-to-pdf=" + pdfPath,
    "--print-to-pdf-no-header",
    pathToFileURL(htmlPath).href
  ], { stdio: "inherit" });
}

for (const [source, pdfName, title, subtitle] of documents) {
  const sourcePath = path.join(ROOT, source);
  if (!fs.existsSync(sourcePath)) {
    throw new Error(`Missing source file: ${sourcePath}`);
  }
  const markdown = fs.readFileSync(sourcePath, "utf8");
  const html = buildHtml(markdown, title, subtitle);
  const htmlPath = path.join(HTML_DIR, pdfName.replace(/\.pdf$/, ".html"));
  const pdfPath = path.join(ROOT, pdfName);
  fs.writeFileSync(htmlPath, html);
  chromePdf(htmlPath, pdfPath);
  console.log(`Wrote ${pdfPath}`);
}
