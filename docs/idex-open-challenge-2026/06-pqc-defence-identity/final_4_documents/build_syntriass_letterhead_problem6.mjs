import fs from "fs";
import { Buffer } from "buffer";

const templatePath = "/Users/richardrich/Desktop/NEXUS/docs/idex-open-challenge-2026/05-cyber-immune-soar/final_4_documents/build_syntriass_letterhead_problem5.mjs";
let code = fs.readFileSync(templatePath, "utf8");

code = code
  .replaceAll("05-cyber-immune-soar", "06-pqc-defence-identity")
  .replaceAll("Cyber_Immune_SOAR", "PQC_Defence_Identity")
  .replaceAll("Cyber Immune SOAR", "PQC Defence Identity")
  .replaceAll("problem5", "problem6");

await import(`data:text/javascript;base64,${Buffer.from(code).toString("base64")}`);
