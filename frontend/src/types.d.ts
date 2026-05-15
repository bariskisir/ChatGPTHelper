// Shared frontend data types exchanged with the Tauri backend.
interface SelectionArea {
  left: number;
  top: number;
  width: number;
  height: number;
  savedAt: string;
}

type ScanKind = "text" | "image";
type HistoryEntryType = "ask" | "text" | "image";
type SystemPromptPreset = "solver" | "none" | "other";

interface AppSettings {
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

interface ThinkingVariantOption {
  value: string;
  description: string;
}

interface AvailableModel {
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

interface ChatGptViewState {
  loggedIn: boolean;
  accountEmail: string;
  limitLabel: string;
  error: string;
}

interface HistoryEntry {
  input: string;
  inputImageDataUrl: string;
  output: string;
  entryType: HistoryEntryType;
  createdAt: string;
}

interface AppViewState {
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

interface ScreenAreaCaptureResult {
  area: SelectionArea;
  imageDataUrl: string;
}

interface FrontendSettings {
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

interface UiEventPayload {
  type: "state" | "answer" | "error" | "shortcut";
  state?: AppViewState;
  answer?: string;
  streaming?: boolean;
  message?: string;
  action?: string;
}

interface TesseractGlobal {
  // Creates a Tesseract OCR worker for the requested language.
  createWorker(language?: string, oem?: number, options?: Record<string, unknown>): Promise<TesseractWorker>;
}

interface TesseractWorker {
  // Runs OCR against an image URL or data URL.
  recognize(image: string): Promise<{ data: { text: string } }>;
  // Applies OCR engine parameters before recognition.
  setParameters(parameters: Record<string, string>): Promise<void>;
}

interface Window {
  Tesseract?: TesseractGlobal;
}
