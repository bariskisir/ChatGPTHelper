/** Shared frontend state and Tauri invocation helpers. */
/// <reference path="./types.d.ts" />
/// <reference path="./dom.ts" />
/// <reference path="./render.ts" />
/// <reference path="./tauri-bridge.ts" />

namespace AppContext {
  let cachedRefs: DomRefs.Refs | null = null;

  // Resolves DOM references after the DOM helper namespace is available.
  function currentRefs(): DomRefs.Refs {
    if (!cachedRefs) {
      cachedRefs = DomRefs.getRefs();
    }
    return cachedRefs;
  }

  export const refs = new Proxy({} as DomRefs.Refs, {
    get(_target, property: string | symbol) {
      return Reflect.get(currentRefs(), property);
    },
  });

  export const model: Renderer.UiModel = {
    appState: null,
    compactMode: false,
    manualDraftActive: false,
    manualImageDataUrl: "",
    streamingAnswer: "",
    copyResetTimer: 0,
  };

  // Invokes a backend command that returns and renders app state.
  export async function invokeState(command: string, args?: Record<string, unknown>): Promise<void> {
    const state = await safeInvoke<AppViewState>(command, args);
    if (state) { Renderer.renderState(refs, model, state); }
  }

  // Invokes a backend command and renders errors without throwing.
  export async function safeInvoke<T = void>(command: string, args?: Record<string, unknown>): Promise<T | null> {
    try { return await TauriBridge.invokeCommand<T>(command, args); }
    catch (error) {
      Renderer.renderStatus(refs, String(error), true);
      Renderer.updateButtons(refs, model);
      return null;
    }
  }

  // Saves frontend settings and renders the refreshed state.
  export async function saveSettings(): Promise<void> {
    const state = await safeInvoke<AppViewState>("save_settings", { settings: Renderer.collectSettings(refs) });
    if (state) { Renderer.renderState(refs, model, state); }
  }

  // Reports whether the frontend model has an active ChatGPT session.
  export function isLoggedIn(): boolean {
    return Boolean(model.appState?.chatgpt.loggedIn);
  }
}
