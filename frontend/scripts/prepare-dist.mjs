// Prepares frontend distribution assets for the Tauri build.
import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const dist = join(root, "dist");
const tesseractBrowser = join(root, "node_modules", "tesseract.js", "dist", "tesseract.min.js");

mkdirSync(dist, { recursive: true });
copyFileSync(join(root, "index.html"), join(dist, "index.html"));
copyFileSync(join(root, "styles.css"), join(dist, "styles.css"));
copyFileSync(join(root, "..", "icons", "icon.png"), join(dist, "icon.png"));

if (existsSync(tesseractBrowser)) {
  copyFileSync(tesseractBrowser, join(dist, "tesseract.min.js"));
}
