/** Clipboard actions for copying assistant output. */

import { model, refs } from "./app-context.js";
import * as Renderer from "./render.js";
import { invokeCommand } from "./tauri-bridge.js";

/** Copies the visible assistant output to the clipboard. */
export async function copyOutput(): Promise<void> {
  const text = currentOutputText();
  if (!text) return;
  try {
    await writeClipboardText(text);
    Renderer.renderCopyFeedback(refs, model);
  } catch (error) {
    Renderer.renderStatus(refs, `Could not copy output: ${error}`, true);
  }
}

/** Chooses the output text currently visible to the user. */
function currentOutputText(): string {
  if (model.manualDraftActive) return "";
  const stateOutput = model.streamingAnswer || model.appState?.selectedHistory?.output || "";
  const visibleOutput = refs.historyOutput.classList.contains("ch-empty") ? "" : refs.historyOutput.textContent || "";
  return (stateOutput || visibleOutput).trim();
}

/** Writes clipboard text through native, browser, or fallback paths. */
async function writeClipboardText(text: string): Promise<void> {
  try { await invokeCommand("copy_text_to_clipboard", { text }); return; } catch { /* fall through */ }
  if (navigator.clipboard?.writeText) {
    try { await navigator.clipboard.writeText(text); return; } catch { /* fall through */ }
  }
  const textarea = document.createElement("textarea");
  textarea.value = text;
  textarea.setAttribute("readonly", "true");
  textarea.style.position = "fixed";
  textarea.style.left = "-9999px";
  textarea.style.top = "0";
  document.body.appendChild(textarea);
  textarea.focus();
  textarea.select();
  const copied = document.execCommand("copy");
  textarea.remove();
  if (!copied) { throw new Error("clipboard write failed"); }
}
