/** Manual text and image input actions. */

import { model, refs, safeInvoke, saveSettings } from "./app-context.js";
import * as Renderer from "./render.js";
import type { AppViewState } from "./types.js";

/** Opens manual input mode with an empty image draft. */
export function startManualDraft(): void {
  model.manualImageDataUrl = "";
  Renderer.setManualDraftMode(refs, model, true);
  Renderer.updateButtons(refs, model);
}

/** Closes manual input mode and restores history display. */
export function cancelManualDraft(): void {
  Renderer.setManualDraftMode(refs, model, false);
}

/** Submits the manual text and/or image input to the backend. */
export async function submitManualInput(): Promise<void> {
  if (!model.manualDraftActive) return;
  const text = refs.manualInput.value.trim();
  if (!text && !model.manualImageDataUrl) {
    Renderer.renderStatus(refs, "Enter text or paste an image first.", true);
    return;
  }
  await saveSettings();
  const state = await safeInvoke<AppViewState>("submit_manual_input", {
    input: { text, imageDataUrl: model.manualImageDataUrl },
  });
  if (state) {
    refs.manualInput.value = "";
    model.manualImageDataUrl = "";
    model.manualDraftActive = false;
    Renderer.renderManualImage(refs, "");
    Renderer.renderState(refs, model, state);
  }
}

/** Captures pasted images while manual input mode is active. */
export function handleManualPaste(event: ClipboardEvent): void {
  if (event.defaultPrevented || !model.manualDraftActive) return;
  const items = Array.from(event.clipboardData?.items || []);
  const imageItem = items.find((item) => item.type.startsWith("image/"));
  const file = imageItem?.getAsFile();
  if (!file) return;
  event.preventDefault();
  const reader = new FileReader();
  reader.onload = () => {
    model.manualImageDataUrl = String(reader.result || "");
    Renderer.renderManualImage(refs, model.manualImageDataUrl);
    Renderer.updateButtons(refs, model);
  };
  reader.readAsDataURL(file);
}
