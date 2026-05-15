/** Manual text and image input actions. */

namespace ManualInputActions {
  // Opens manual input mode with an empty image draft.
  export function startManualDraft(): void {
    AppContext.model.manualImageDataUrl = "";
    Renderer.setManualDraftMode(AppContext.refs, AppContext.model, true);
    Renderer.updateButtons(AppContext.refs, AppContext.model);
  }

  // Closes manual input mode and restores history display.
  export function cancelManualDraft(): void {
    Renderer.setManualDraftMode(AppContext.refs, AppContext.model, false);
  }

  // Documents the s ub mi tm an ua li np ut function.
  export async function submitManualInput(): Promise<void> {
    if (!AppContext.model.manualDraftActive) return;
    const text = AppContext.refs.manualInput.value.trim();
    if (!text && !AppContext.model.manualImageDataUrl) {
      Renderer.renderStatus(AppContext.refs, "Enter text or paste an image first.", true);
      return;
    }
    await AppContext.saveSettings();
    const state = await AppContext.safeInvoke<AppViewState>("submit_manual_input", {
      input: { text, imageDataUrl: AppContext.model.manualImageDataUrl },
    });
    if (state) {
      AppContext.refs.manualInput.value = "";
      AppContext.model.manualImageDataUrl = "";
      AppContext.model.manualDraftActive = false;
      Renderer.renderManualImage(AppContext.refs, "");
      Renderer.renderState(AppContext.refs, AppContext.model, state);
    }
  }

  // Captures pasted images while manual input mode is active.
  export function handleManualPaste(event: ClipboardEvent): void {
    if (event.defaultPrevented || !AppContext.model.manualDraftActive) return;
    const items = Array.from(event.clipboardData?.items || []);
    const imageItem = items.find((item) => item.type.startsWith("image/"));
    const file = imageItem?.getAsFile();
    if (!file) return;
    event.preventDefault();
    const reader = new FileReader();
    reader.onload = () => {
      AppContext.model.manualImageDataUrl = String(reader.result || "");
      Renderer.renderManualImage(AppContext.refs, AppContext.model.manualImageDataUrl);
      Renderer.updateButtons(AppContext.refs, AppContext.model);
    };
    reader.readAsDataURL(file);
  }
}
