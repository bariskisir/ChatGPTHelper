/** Shared frontend state and Tauri invocation helpers. */

import type { Refs } from "./dom.js";
import type { UiModel } from "./render.js";
import type { AppViewState } from "./types.js";
import { getRefs } from "./dom.js";
import * as Renderer from "./render.js";
import { invokeCommand } from "./tauri-bridge.js";

let cachedRefs: Refs | null = null;

/** Resolves DOM references after the DOM helper module is available. */
function currentRefs(): Refs {
  if (!cachedRefs) {
    cachedRefs = getRefs();
  }
  return cachedRefs;
}

export const refs = new Proxy({} as Refs, {
  get(_target, property: string | symbol) {
    return Reflect.get(currentRefs(), property);
  },
});

export const model: UiModel = {
  appState: null,
  compactMode: false,
  manualDraftActive: false,
  manualImageDataUrl: "",
  streamingAnswer: "",
  copyResetTimer: 0,
};

/** Invokes a backend command that returns and renders app state. */
export async function invokeState(command: string, args?: Record<string, unknown>): Promise<void> {
  const state = await safeInvoke<AppViewState>(command, args);
  if (state) { Renderer.renderState(refs, model, state); }
}

/** Invokes a backend command and renders errors without throwing. */
export async function safeInvoke<T = void>(command: string, args?: Record<string, unknown>): Promise<T | null> {
  try { return await invokeCommand<T>(command, args); }
  catch (error) {
    Renderer.renderStatus(refs, String(error), true);
    Renderer.updateButtons(refs, model);
    return null;
  }
}

/** Saves frontend settings and renders the refreshed state. */
export async function saveSettings(): Promise<void> {
  const state = await safeInvoke<AppViewState>("save_settings", { settings: Renderer.collectSettings(refs) });
  if (state) { Renderer.renderState(refs, model, state); }
}

/** Reports whether the frontend model has an active ChatGPT session. */
export function isLoggedIn(): boolean {
  return Boolean(model.appState?.chatgpt.loggedIn);
}
