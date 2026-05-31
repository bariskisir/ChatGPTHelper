/** Rendering helpers for ChatGPT Helper. */

import type { Refs } from "./dom.js";
import type {
  AppViewState,
  AvailableModel,
  FrontendSettings,
  SystemPromptPreset,
} from "./types.js";

const TEXT_SOLVER_PROMPT = "You are a careful problem solver. Read the selected content, solve accurately, and give the final answer clearly.";
const IMAGE_SOLVER_PROMPT = "You are a careful image problem solver. Analyze the selected image area, solve math accurately, interpret charts, diagrams, UI, or other image content when present, and give the key answer concisely and clearly.";

export interface UiModel {
  appState: AppViewState | null;
  compactMode: boolean;
  manualDraftActive: boolean;
  manualImageDataUrl: string;
  streamingAnswer: string;
  copyResetTimer: number;
}

/** Renders a full backend state update into the UI. */
export function renderState(refs: Refs, model: UiModel, state: AppViewState): void {
  model.appState = state;
  model.streamingAnswer = "";
  populateOptions(refs, state);
  renderStatus(refs, state.status);
  renderAccount(refs, state);
  renderHistory(refs, model);
  updateButtons(refs, model);
}

/** Updates signed-in and signed-out status text. */
export function renderStatus(refs: Refs, message: string, isError = false): void {
  const text = message || "Ready.";
  refs.statusText.textContent = text;
  refs.signedOutStatusText.textContent = text;
  refs.statusRow.classList.toggle("is-error", isError);
  refs.signedOutStatusText.classList.toggle("is-error", isError);
}

/** Renders partial answer text while a response is streaming. */
export function renderStreamingAnswer(refs: Refs, model: UiModel, answer: string): void {
  model.streamingAnswer = answer;
  refs.historyOutput.textContent = answer || "";
  refs.historyOutput.classList.toggle("ch-empty", !answer);
  refs.historyOutput.scrollTop = refs.historyOutput.scrollHeight;
  updateButtons(refs, model);
  renderStatus(refs, "Streaming answer...");
}

/** Shows short-lived copy confirmation on the copy button. */
export function renderCopyFeedback(refs: Refs, model: UiModel): void {
  window.clearTimeout(model.copyResetTimer);
  refs.copyOutputButton.classList.add("is-copied");
  refs.copyOutputButton.textContent = "✓ copied";
  model.copyResetTimer = window.setTimeout(() => {
    refs.copyOutputButton.classList.remove("is-copied");
    refs.copyOutputButton.textContent = "copy";
    updateButtons(refs, model);
  }, 1000);
}

/** Reads frontend controls into the settings payload. */
export function collectSettings(refs: Refs): FrontendSettings {
  return {
    textScanModel: refs.textModelSelect.value,
    imageScanModel: refs.imageModelSelect.value,
    textThinkingVariant: refs.textThinkingSelect.value,
    imageThinkingVariant: refs.imageThinkingSelect.value,
    textSystemPromptPreset: refs.textPromptSelect.value,
    imageSystemPromptPreset: refs.imagePromptSelect.value,
    textCustomSystemPrompt: refs.textPromptSelect.value === "other" && !refs.textCustomPrompt.readOnly ? refs.textCustomPrompt.value : "",
    imageCustomSystemPrompt: refs.imagePromptSelect.value === "other" && !refs.imageCustomPrompt.readOnly ? refs.imageCustomPrompt.value : "",
    alwaysOnTop: refs.alwaysOnTopButton.classList.contains("is-active"),
  };
}

/** Shows or clears the pasted manual image preview. */
export function renderManualImage(refs: Refs, dataUrl: string): void {
  refs.historyInputImage.hidden = !dataUrl;
  refs.historyInput.classList.toggle("ch-history-box-manual-image", Boolean(dataUrl));
  refs.manualSendButton.classList.toggle("ch-image-button", Boolean(dataUrl));
  refs.manualSendButton.textContent = dataUrl ? "Send Image" : "Send Text";
  if (dataUrl) {
    refs.historyInputImage.src = dataUrl;
    refs.manualInput.placeholder = "";
  } else {
    refs.historyInputImage.removeAttribute("src");
    refs.manualInput.placeholder = "Enter text or paste image";
  }
}

/** Switches the history pane between manual input and history display. */
export function setManualDraftMode(refs: Refs, model: UiModel, enabled: boolean): void {
  const wasActive = model.manualDraftActive;
  model.manualDraftActive = enabled;
  refs.historyInput.classList.toggle("ch-history-box-manual", enabled);
  refs.manualInput.hidden = !enabled;
  refs.manualActionRow.hidden = !enabled;
  refs.historyInputText.hidden = enabled;
  refs.historyOutput.classList.toggle("ch-empty", enabled);
  refs.addManualButton.disabled = enabled;
  refs.cancelManualButton.disabled = !enabled;
  refs.historyPrevButton.disabled = enabled || !model.appState?.history.length;
  refs.historyNextButton.disabled = enabled || !model.appState?.history.length;
  refs.deleteHistoryButton.disabled = enabled || !model.appState?.history.length;
  if (enabled) {
    refs.historyInputText.textContent = "";
    refs.historyOutput.textContent = "";
    if (!wasActive) { refs.manualInput.value = ""; }
    renderManualImage(refs, model.manualImageDataUrl);
    if (!wasActive) { refs.manualInput.focus(); }
  } else {
    model.manualImageDataUrl = "";
    refs.manualInput.value = "";
    renderManualImage(refs, "");
    renderHistory(refs, model);
    updateButtons(refs, model);
  }
}

/** Toggles compact UI mode on the app shell. */
export function setCompactMode(refs: Refs, model: UiModel, enabled: boolean): void {
  model.compactMode = enabled;
  refs.appShell.classList.toggle("is-compact", enabled);
  refs.compactButton.textContent = enabled ? "Full" : "Compact";
}

/** Synchronizes button and control disabled states with the model. */
export function updateButtons(refs: Refs, model: UiModel): void {
  const state = model.appState;
  if (!state) return;
  const loggedIn = state.chatgpt.loggedIn;
  refs.signedOutView.hidden = loggedIn;
  refs.signedInView.hidden = !loggedIn;
  refs.loginButton.disabled = loggedIn;
  refs.signOutButton.disabled = !loggedIn;
  refs.refreshButton.disabled = !loggedIn;
  refs.scanTextButton.disabled = !loggedIn;
  refs.scanImageButton.disabled = !loggedIn;
  refs.manualSendButton.disabled = !loggedIn || (model.manualDraftActive && !refs.manualInput.value.trim() && !model.manualImageDataUrl);
  refs.textModelSelect.disabled = !loggedIn;
  refs.textThinkingSelect.disabled = !loggedIn;
  refs.imageModelSelect.disabled = !loggedIn;
  refs.imageThinkingSelect.disabled = !loggedIn;
  refs.alwaysOnTopButton.classList.toggle("is-active", state.settings.alwaysOnTop);
  refs.alwaysOnTopButton.setAttribute("aria-pressed", String(state.settings.alwaysOnTop));
  refs.historyPrevButton.disabled = model.manualDraftActive || !state.history.length || state.historyIndex <= 0;
  refs.historyNextButton.disabled = model.manualDraftActive || !state.history.length || state.historyIndex >= state.history.length - 1;
  refs.addManualButton.disabled = model.manualDraftActive;
  refs.cancelManualButton.disabled = !model.manualDraftActive;
  refs.deleteHistoryButton.disabled = model.manualDraftActive || !state.history.length;
  refs.copyOutputButton.disabled = !visibleOutput(model);
}

/** Renders sign-in, account, and usage-limit status. */
function renderAccount(refs: Refs, state: AppViewState): void {
  refs.accountLabel.textContent = state.chatgpt.loggedIn ? state.chatgpt.accountEmail || "Signed in" : "Not signed in";
  refs.limitText.textContent = state.chatgpt.limitLabel || "--";
  if (!state.chatgpt.loggedIn && !state.chatgpt.error) {
    refs.signedOutStatusText.textContent = "Sign in with ChatGPT to use scans and helper actions.";
    refs.signedOutStatusText.classList.remove("is-error");
  }
  if (state.chatgpt.error) { renderStatus(refs, state.chatgpt.error, true); }
}

/** Renders the selected history entry or empty history state. */
function renderHistory(refs: Refs, model: UiModel): void {
  const state = model.appState;
  if (!state) return;
  refs.historyCounter.textContent = state.history.length ? `${state.historyIndex + 1}/${state.history.length}` : "0/0";
  const entry = state.selectedHistory;
  if (model.manualDraftActive) { setManualDraftMode(refs, model, true); return; }
  refs.historyInput.classList.remove("ch-history-box-manual", "ch-history-box-manual-image");
  refs.manualInput.hidden = true;
  refs.manualActionRow.hidden = true;
  refs.historyInputImage.hidden = true;
  refs.historyInputImage.removeAttribute("src");
  refs.historyInputText.hidden = false;
  refs.historyInputText.textContent = "";
  refs.historyOutput.textContent = "";
  if (!entry) {
    refs.historyInputText.textContent = "No history yet.";
    refs.historyOutput.textContent = "No history yet.";
    refs.historyInput.classList.add("ch-empty");
    refs.historyOutput.classList.add("ch-empty");
    return;
  }
  refs.historyInput.classList.remove("ch-empty");
  refs.historyOutput.classList.toggle("ch-empty", !entry.output);
  if (entry.inputImageDataUrl) {
    refs.historyInputImage.src = entry.inputImageDataUrl;
    refs.historyInputImage.hidden = false;
  }
  refs.historyInputText.textContent = entry.input || (entry.inputImageDataUrl ? "" : "Image input");
  refs.historyInputText.hidden = !refs.historyInputText.textContent;
  refs.historyOutput.textContent = model.streamingAnswer || entry.output || "No answer yet.";
  refs.copyOutputButton.disabled = !visibleOutput(model);
}

/** Rebuilds model, reasoning, and prompt controls from state. */
function populateOptions(refs: Refs, state: AppViewState): void {
  replaceOptions(refs.textModelSelect, state.models.filter((i) => !i.hidden).sort(compareModels).map((i) => ({ value: i.model, label: i.displayName || i.model, title: i.description || i.model })));
  refs.textModelSelect.value = state.settings.textScanModel;
  replaceOptions(refs.imageModelSelect, state.models.filter((i) => !i.hidden && i.inputModalities.includes("image")).sort(compareModels).map((i) => ({ value: i.model, label: i.displayName || i.model, title: i.description || i.model })));
  refs.imageModelSelect.value = state.settings.imageScanModel;
  replaceOptions(refs.textThinkingSelect, state.textThinkingVariants.map((i) => ({ value: i.value, label: i.value, title: i.description })));
  refs.textThinkingSelect.value = state.settings.textThinkingVariant;
  replaceOptions(refs.imageThinkingSelect, state.imageThinkingVariants.map((i) => ({ value: i.value, label: i.value, title: i.description })));
  refs.imageThinkingSelect.value = state.settings.imageThinkingVariant;
  refs.textPromptSelect.value = state.settings.textSystemPromptPreset;
  refs.imagePromptSelect.value = state.settings.imageSystemPromptPreset;
  renderPromptField(refs.textCustomPrompt, state.settings.textSystemPromptPreset, state.settings.textCustomSystemPrompt, TEXT_SOLVER_PROMPT);
  renderPromptField(refs.imageCustomPrompt, state.settings.imageSystemPromptPreset, state.settings.imageCustomSystemPrompt, IMAGE_SOLVER_PROMPT);
}

/** Updates a custom prompt field for the selected preset. */
function renderPromptField(textarea: HTMLTextAreaElement, preset: SystemPromptPreset, customValue: string, solverPrompt: string): void {
  textarea.hidden = false;
  textarea.readOnly = preset !== "other";
  textarea.classList.toggle("is-solver", preset === "solver");
  textarea.value = preset === "solver" ? solverPrompt : preset === "none" ? "" : customValue || "";
}

/** Returns the currently copyable output text. */
function visibleOutput(model: UiModel): string {
  if (model.manualDraftActive) return "";
  return (model.streamingAnswer || model.appState?.selectedHistory?.output || "").trim();
}

/** Replaces a select element's options while preserving selection when possible. */
function replaceOptions(select: HTMLSelectElement, options: Array<{ value: string; label: string; title?: string }>): void {
  const previous = select.value;
  select.innerHTML = "";
  for (const option of options) {
    const element = document.createElement("option");
    element.value = option.value;
    element.textContent = option.label;
    element.title = option.title || option.label;
    select.appendChild(element);
  }
  if (options.some((o) => o.value === previous)) { select.value = previous; }
}

/** Sorts model choices with newer and non-mini models first. */
function compareModels(a: AvailableModel, b: AvailableModel): number {
  const left = modelSortParts(a.model || a.displayName);
  const right = modelSortParts(b.model || b.displayName);
  for (let index = 0; index < Math.max(left.numbers.length, right.numbers.length); index += 1) {
    const diff = (right.numbers[index] || 0) - (left.numbers[index] || 0);
    if (diff !== 0) return diff;
  }
  if (left.mini !== right.mini) return left.mini ? 1 : -1;
  return (a.displayName || a.model).localeCompare(b.displayName || b.model);
}

/** Extracts model-number and mini flags for sorting. */
function modelSortParts(value: string): { numbers: number[]; mini: boolean } {
  return {
    numbers: (String(value).match(/\d+(?:\.\d+)?/g) || []).map(Number),
    mini: /\bmini\b/i.test(value),
  };
}
