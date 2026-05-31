/** Shared frontend data types exchanged with the Tauri backend. */

export interface SelectionArea {
  left: number;
  top: number;
  width: number;
  height: number;
  savedAt: string;
}

export type ScanKind = "text" | "image";
export type HistoryEntryType = "ask" | "text" | "image";
export type SystemPromptPreset = "solver" | "none" | "other";

export interface AppSettings {
  textScanModel: string;
  imageScanModel: string;
  textThinkingVariant: string;
  imageThinkingVariant: string;
  textSystemPromptPreset: SystemPromptPreset;
  imageSystemPromptPreset: SystemPromptPreset;
  textCustomSystemPrompt: string;
  imageCustomSystemPrompt: string;
  lastTextArea: SelectionArea | null;
  lastImageArea: SelectionArea | null;
  alwaysOnTop: boolean;
}

export interface ThinkingVariantOption {
  value: string;
  description: string;
}

export interface AvailableModel {
  id: string;
  model: string;
  displayName: string;
  description: string;
  hidden: boolean;
  isDefault: boolean;
  inputModalities: string[];
  defaultThinkingVariant: string;
  thinkingVariants: ThinkingVariantOption[];
}

export interface ChatGptViewState {
  loggedIn: boolean;
  accountEmail: string;
  limitLabel: string;
  error: string;
}

export interface HistoryEntry {
  input: string;
  inputImageDataUrl: string;
  output: string;
  entryType: HistoryEntryType;
  createdAt: string;
}

export interface AppViewState {
  settings: AppSettings;
  status: string;
  chatgpt: ChatGptViewState;
  models: AvailableModel[];
  textThinkingVariants: ThinkingVariantOption[];
  imageThinkingVariants: ThinkingVariantOption[];
  history: HistoryEntry[];
  historyIndex: number;
  selectedHistory: HistoryEntry | null;
}

export interface ScreenAreaCaptureResult {
  area: SelectionArea;
  imageDataUrl: string;
}

export interface FrontendSettings {
  textScanModel: string;
  imageScanModel: string;
  textThinkingVariant: string;
  imageThinkingVariant: string;
  textSystemPromptPreset: string;
  imageSystemPromptPreset: string;
  textCustomSystemPrompt: string;
  imageCustomSystemPrompt: string;
  alwaysOnTop: boolean;
}

export interface UiEventPayload {
  type: "state" | "answer" | "error" | "shortcut";
  state?: AppViewState;
  answer?: string;
  streaming?: boolean;
  message?: string;
  action?: string;
}

export interface TesseractGlobal {
  /** Creates a Tesseract OCR worker for the requested language. */
  createWorker(language?: string, oem?: number, options?: Record<string, unknown>): Promise<TesseractWorker>;
}

export interface TesseractWorker {
  /** Runs OCR against an image URL or data URL. */
  recognize(image: string): Promise<{ data: { text: string } }>;
  /** Applies OCR engine parameters before recognition. */
  setParameters(parameters: Record<string, string>): Promise<void>;
}

declare global {
  interface Window {
    Tesseract?: TesseractGlobal;
  }
}
