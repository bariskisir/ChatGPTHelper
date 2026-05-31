/** Main browser entry point for the ChatGPT Helper UI. */

import { invokeState, model, refs, safeInvoke, saveSettings } from "./app-context.js";
import * as ClipboardActions from "./clipboard.js";
import * as ManualInputActions from "./manual-input.js";
import * as Renderer from "./render.js";
import * as ScanActions from "./scan.js";
import * as ShortcutActions from "./shortcuts.js";
import { listenAppEvents } from "./tauri-bridge.js";
import type { AppViewState, UiEventPayload } from "./types.js";

document.addEventListener("DOMContentLoaded", async () => {
  bindEvents();
  await listenAppEvents(handleUiEvent);
  await refreshState();
});

/** Connects DOM controls to application actions. */
function bindEvents(): void {
  refs.loginButton.addEventListener("click", () => invokeState("start_chatgpt_login"));
  refs.signOutButton.addEventListener("click", () => invokeState("sign_out_chatgpt"));
  refs.refreshButton.addEventListener("click", refreshChatgptData);
  refs.developerLink.addEventListener("click", () => safeInvoke("open_developer_site"));
  refs.sourceLink.addEventListener("click", () => safeInvoke("open_source_site"));

  refs.textModelSelect.addEventListener("change", saveSettings);
  refs.textThinkingSelect.addEventListener("change", saveSettings);
  refs.imageModelSelect.addEventListener("change", saveSettings);
  refs.imageThinkingSelect.addEventListener("change", saveSettings);
  refs.textPromptSelect.addEventListener("change", saveSettings);
  refs.imagePromptSelect.addEventListener("change", saveSettings);
  refs.textCustomPrompt.addEventListener("input", debounceSaveSettings);
  refs.imageCustomPrompt.addEventListener("input", debounceSaveSettings);

  refs.scanTextButton.addEventListener("click", () => ScanActions.startScan("text", model.appState?.settings.lastTextArea || null, true));
  refs.scanImageButton.addEventListener("click", () => ScanActions.startScan("image", model.appState?.settings.lastImageArea || null, true));
  refs.addManualButton.addEventListener("click", ManualInputActions.startManualDraft);
  refs.cancelManualButton.addEventListener("click", ManualInputActions.cancelManualDraft);
  refs.manualInput.addEventListener("input", () => Renderer.updateButtons(refs, model));
  refs.manualInput.addEventListener("paste", ManualInputActions.handleManualPaste);
  document.addEventListener("paste", ManualInputActions.handleManualPaste);
  refs.manualSendButton.addEventListener("click", ManualInputActions.submitManualInput);
  refs.compactButton.addEventListener("click", toggleCompactMode);
  refs.alwaysOnTopButton.addEventListener("click", toggleAlwaysOnTop);

  refs.historyPrevButton.addEventListener("click", () => invokeState("select_history_by_offset", { offset: -1 }));
  refs.historyNextButton.addEventListener("click", () => invokeState("select_history_by_offset", { offset: 1 }));
  refs.deleteHistoryButton.addEventListener("click", () => invokeState("delete_history"));
  refs.copyOutputButton.addEventListener("click", ClipboardActions.copyOutput);

  document.addEventListener("keydown", ShortcutActions.handleKeyboardShortcut);
}

/** Routes backend events to the appropriate frontend handler. */
function handleUiEvent(payload: UiEventPayload): void {
  if (payload.type === "state" && payload.state) {
    Renderer.renderState(refs, model, payload.state);
  } else if (payload.type === "answer") {
    Renderer.renderStreamingAnswer(refs, model, payload.answer || "");
  } else if (payload.type === "error") {
    Renderer.renderStatus(refs, payload.message || "Error", true);
    Renderer.updateButtons(refs, model);
  } else if (payload.type === "shortcut") {
    ShortcutActions.handleShortcutAction(payload.action || "");
  }
}

/** Loads the initial backend view state. */
async function refreshState(): Promise<void> {
  const state = await safeInvoke<AppViewState>("get_app_state");
  if (state) { Renderer.renderState(refs, model, state); }
}

/** Refreshes model and usage information for the signed-in account. */
async function refreshChatgptData(): Promise<void> {
  const modelState = await safeInvoke<AppViewState>("refresh_chatgpt_models");
  if (modelState) { Renderer.renderState(refs, model, modelState); }
  const limitState = await safeInvoke<AppViewState>("refresh_chatgpt_limits");
  if (limitState) { Renderer.renderState(refs, model, limitState); }
}

let saveTimer = 0;
/** Delays settings persistence while prompt text is changing. */
function debounceSaveSettings(): void {
  window.clearTimeout(saveTimer);
  saveTimer = window.setTimeout(saveSettings, 250);
}

/** Toggles the native window always-on-top setting. */
async function toggleAlwaysOnTop(): Promise<void> {
  const enabled = !model.appState?.settings.alwaysOnTop;
  await invokeState("set_always_on_top", { enabled });
}

/** Toggles compact UI rendering. */
function toggleCompactMode(): void {
  Renderer.setCompactMode(refs, model, !model.compactMode);
}
