/** Main browser entry point for the ChatGPT Helper UI. */
/// <reference path="./types.d.ts" />
/// <reference path="./tauri.d.ts" />
/// <reference path="./dom.ts" />
/// <reference path="./tauri-bridge.ts" />
/// <reference path="./render.ts" />
/// <reference path="./app-context.ts" />
/// <reference path="./clipboard.ts" />
/// <reference path="./manual-input.ts" />
/// <reference path="./scan.ts" />
/// <reference path="./shortcuts.ts" />

namespace App {
  const refs = AppContext.refs;
  const model = AppContext.model;

  document.addEventListener("DOMContentLoaded", async () => {
    bindEvents();
    await TauriBridge.listenAppEvents(handleUiEvent);
    await refreshState();
  });

  // Connects DOM controls to application actions.
  function bindEvents(): void {
    refs.loginButton.addEventListener("click", () => AppContext.invokeState("start_chatgpt_login"));
    refs.signOutButton.addEventListener("click", () => AppContext.invokeState("sign_out_chatgpt"));
    refs.refreshButton.addEventListener("click", refreshChatgptData);
    refs.developerLink.addEventListener("click", () => AppContext.safeInvoke("open_developer_site"));
    refs.sourceLink.addEventListener("click", () => AppContext.safeInvoke("open_source_site"));

    refs.textModelSelect.addEventListener("change", AppContext.saveSettings);
    refs.textThinkingSelect.addEventListener("change", AppContext.saveSettings);
    refs.imageModelSelect.addEventListener("change", AppContext.saveSettings);
    refs.imageThinkingSelect.addEventListener("change", AppContext.saveSettings);
    refs.textPromptSelect.addEventListener("change", AppContext.saveSettings);
    refs.imagePromptSelect.addEventListener("change", AppContext.saveSettings);
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

    refs.historyPrevButton.addEventListener("click", () => AppContext.invokeState("select_history_by_offset", { offset: -1 }));
    refs.historyNextButton.addEventListener("click", () => AppContext.invokeState("select_history_by_offset", { offset: 1 }));
    refs.deleteHistoryButton.addEventListener("click", () => AppContext.invokeState("delete_history"));
    refs.copyOutputButton.addEventListener("click", ClipboardActions.copyOutput);

    document.addEventListener("keydown", ShortcutActions.handleKeyboardShortcut);
  }

  // Routes backend events to the appropriate frontend handler.
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

  // Loads the initial backend view state.
  async function refreshState(): Promise<void> {
    const state = await AppContext.safeInvoke<AppViewState>("get_app_state");
    if (state) { Renderer.renderState(refs, model, state); }
  }

  // Refreshes model and usage information for the signed-in account.
  async function refreshChatgptData(): Promise<void> {
    const modelState = await AppContext.safeInvoke<AppViewState>("refresh_chatgpt_models");
    if (modelState) { Renderer.renderState(refs, model, modelState); }
    const limitState = await AppContext.safeInvoke<AppViewState>("refresh_chatgpt_limits");
    if (limitState) { Renderer.renderState(refs, model, limitState); }
  }

  let saveTimer = 0;
  // Delays settings persistence while prompt text is changing.
  function debounceSaveSettings(): void {
    window.clearTimeout(saveTimer);
    saveTimer = window.setTimeout(AppContext.saveSettings, 250);
  }

  // Toggles the native window always-on-top setting.
  async function toggleAlwaysOnTop(): Promise<void> {
    const enabled = !model.appState?.settings.alwaysOnTop;
    await AppContext.invokeState("set_always_on_top", { enabled });
  }

  // Toggles compact UI rendering.
  function toggleCompactMode(): void {
    Renderer.setCompactMode(refs, model, !model.compactMode);
  }
}
