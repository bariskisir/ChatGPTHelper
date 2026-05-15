/** Keyboard and global shortcut actions. */

namespace ShortcutActions {
  // Handles in-window keyboard shortcuts.
  export function handleKeyboardShortcut(event: KeyboardEvent): void {
    if (isEditable(event.target)) return;
    if (event.ctrlKey && event.shiftKey && !event.altKey && !event.metaKey) {
      const key = event.key.toLocaleLowerCase("tr-TR");
      if (key === "t") { event.preventDefault(); void ScanActions.startScan("text", AppContext.model.appState?.settings.lastTextArea || null, true); }
      else if (key === "i" || event.key === "İ" || event.key === "ı") { event.preventDefault(); void ScanActions.startScan("image", AppContext.model.appState?.settings.lastImageArea || null, true); }
      else if (event.key === "1") { event.preventDefault(); void ScanActions.repeatScan("text"); }
      else if (event.key === "2") { event.preventDefault(); void ScanActions.repeatScan("image"); }
    }
  }

  // Handles shortcut actions emitted by the Rust backend.
  export function handleShortcutAction(action: string): void {
    if (!AppContext.isLoggedIn()) return;
    if (action === "scan-text") { void ScanActions.startScan("text", AppContext.model.appState?.settings.lastTextArea || null, true); }
    else if (action === "scan-image") { void ScanActions.startScan("image", AppContext.model.appState?.settings.lastImageArea || null, true); }
    else if (action === "repeat-text") { void ScanActions.repeatScan("text"); }
    else if (action === "repeat-image") { void ScanActions.repeatScan("image"); }
  }

  // Checks whether a keyboard event target accepts text input.
  function isEditable(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) return false;
    if (target.isContentEditable) return true;
    const tag = target.tagName.toLowerCase();
    return tag === "input" || tag === "textarea" || tag === "select";
  }
}
