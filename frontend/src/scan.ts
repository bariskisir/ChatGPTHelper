/** Screen scan and OCR actions. */

import { isLoggedIn, model, refs, safeInvoke, saveSettings } from "./app-context.js";
import * as Renderer from "./render.js";
import { invokeCommand } from "./tauri-bridge.js";
import type { AppViewState, ScanKind, ScreenAreaCaptureResult, SelectionArea, TesseractWorker } from "./types.js";

let tesseractWorker: Promise<TesseractWorker> | null = null;

/** Repeats the last saved scan area for the requested kind. */
export async function repeatScan(kind: ScanKind): Promise<void> {
  if (!isLoggedIn()) return;
  const area = await safeInvoke<SelectionArea | null>("repeat_scan", { kind });
  if (!area) {
    Renderer.renderStatus(refs, `No previous ${kind} scan area was saved.`, true);
    return;
  }
  await startScan(kind, area, false);
}

/** Starts a screen selection or reused-area scan. */
export async function startScan(kind: ScanKind, previousArea: SelectionArea | null, confirmPrevious = true): Promise<void> {
  if (!isLoggedIn()) return;
  await saveSettings();
  try {
    Renderer.renderStatus(refs, "Select a screen area...");
    const result = await invokeCommand<ScreenAreaCaptureResult | null>("select_screen_area", { kind, previousArea, confirmPrevious });
    if (!result) {
      Renderer.renderStatus(refs, "Ready.");
      Renderer.updateButtons(refs, model);
      return;
    }
    await submitCapturedSelection(kind, result);
  } catch (error) {
    Renderer.renderStatus(refs, String(error), true);
    Renderer.updateButtons(refs, model);
  }
}

/** Runs OCR when needed and submits the captured selection. */
async function submitCapturedSelection(kind: ScanKind, result: ScreenAreaCaptureResult): Promise<void> {
  try {
    Renderer.renderStatus(refs, kind === "text" ? "Reading text with OCR..." : "Cropping image...");
    let extractedText = "";
    if (kind === "text") {
      const worker = await getTesseractWorker();
      const ocr = await worker.recognize(result.imageDataUrl);
      extractedText = ocr.data.text.trim();
    }
    const state = await safeInvoke<AppViewState>("submit_scan", {
      input: { kind, text: extractedText, imageDataUrl: kind === "image" ? result.imageDataUrl : "", area: result.area },
    });
    if (state) { Renderer.renderState(refs, model, state); }
  } catch (error) {
    Renderer.renderStatus(refs, `Scan failed: ${error}`, true);
    Renderer.updateButtons(refs, model);
  }
}

/** Creates or reuses the frontend Tesseract OCR worker. */
async function getTesseractWorker(): Promise<TesseractWorker> {
  if (!window.Tesseract) { throw new Error("Tesseract assets are not loaded."); }
  if (!tesseractWorker) {
    tesseractWorker = window.Tesseract.createWorker("eng", 1).then(async (worker) => {
      await worker.setParameters({ user_defined_dpi: "300", preserve_interword_spaces: "1", tessedit_pageseg_mode: "6" });
      return worker;
    });
  }
  return tesseractWorker;
}
