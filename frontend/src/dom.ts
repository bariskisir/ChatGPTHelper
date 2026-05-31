/** DOM reference collection for ChatGPT Helper. */

export interface Refs {
  appShell: HTMLElement;
  signedOutView: HTMLElement;
  signedInView: HTMLElement;
  statusRow: HTMLElement;
  statusText: HTMLElement;
  signedOutStatusText: HTMLElement;
  accountLabel: HTMLElement;
  limitText: HTMLElement;
  refreshButton: HTMLButtonElement;
  loginButton: HTMLButtonElement;
  signOutButton: HTMLButtonElement;
  textModelSelect: HTMLSelectElement;
  textThinkingSelect: HTMLSelectElement;
  imageModelSelect: HTMLSelectElement;
  imageThinkingSelect: HTMLSelectElement;
  scanTextButton: HTMLButtonElement;
  scanImageButton: HTMLButtonElement;
  compactButton: HTMLButtonElement;
  alwaysOnTopButton: HTMLButtonElement;
  manualInput: HTMLTextAreaElement;
  manualActionRow: HTMLElement;
  manualSendButton: HTMLButtonElement;
  textPromptSelect: HTMLSelectElement;
  imagePromptSelect: HTMLSelectElement;
  textCustomPrompt: HTMLTextAreaElement;
  imageCustomPrompt: HTMLTextAreaElement;
  historyPrevButton: HTMLButtonElement;
  historyNextButton: HTMLButtonElement;
  historyCounter: HTMLElement;
  addManualButton: HTMLButtonElement;
  cancelManualButton: HTMLButtonElement;
  deleteHistoryButton: HTMLButtonElement;
  copyOutputButton: HTMLButtonElement;
  historyInput: HTMLElement;
  historyInputImage: HTMLImageElement;
  historyInputText: HTMLElement;
  historyOutput: HTMLElement;
  developerLink: HTMLButtonElement;
  sourceLink: HTMLButtonElement;
}

/** Collects and validates all required DOM references. */
export function getRefs(): Refs {
  return {
    appShell: get("appShell"),
    signedOutView: get("signedOutView"),
    signedInView: get("signedInView"),
    statusRow: get("statusRow"),
    statusText: get("statusText"),
    signedOutStatusText: get("signedOutStatusText"),
    accountLabel: get("accountLabel"),
    limitText: get("limitText"),
    refreshButton: get("refreshButton"),
    loginButton: get("loginButton"),
    signOutButton: get("signOutButton"),
    textModelSelect: get("textModelSelect"),
    textThinkingSelect: get("textThinkingSelect"),
    imageModelSelect: get("imageModelSelect"),
    imageThinkingSelect: get("imageThinkingSelect"),
    scanTextButton: get("scanTextButton"),
    scanImageButton: get("scanImageButton"),
    compactButton: get("compactButton"),
    alwaysOnTopButton: get("alwaysOnTopButton"),
    manualInput: get("manualInput"),
    manualActionRow: get("manualActionRow"),
    manualSendButton: get("manualSendButton"),
    textPromptSelect: get("textPromptSelect"),
    imagePromptSelect: get("imagePromptSelect"),
    textCustomPrompt: get("textCustomPrompt"),
    imageCustomPrompt: get("imageCustomPrompt"),
    historyPrevButton: get("historyPrevButton"),
    historyNextButton: get("historyNextButton"),
    historyCounter: get("historyCounter"),
    addManualButton: get("addManualButton"),
    cancelManualButton: get("cancelManualButton"),
    deleteHistoryButton: get("deleteHistoryButton"),
    copyOutputButton: get("copyOutputButton"),
    historyInput: get("historyInput"),
    historyInputImage: get("historyInputImage"),
    historyInputText: get("historyInputText"),
    historyOutput: get("historyOutput"),
    developerLink: get("developerLink"),
    sourceLink: get("sourceLink"),
  };
}

/** Returns a typed DOM element or throws when it is missing. */
function get<T extends HTMLElement>(id: string): T {
  const element = document.getElementById(id);
  if (!element) {
    throw new Error(`Missing element #${id}`);
  }
  return element as T;
}
