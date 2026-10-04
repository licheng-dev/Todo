export type AiProvider = "deepseek" | "qwen" | "kimi";

export interface ProviderMeta {
  value: AiProvider;
  label: string;
  model: string;
}

export const PROVIDERS: ProviderMeta[] = [
  { value: "deepseek", label: "DeepSeek", model: "deepseek-chat" },
  { value: "qwen", label: "千问", model: "qwen-plus" },
  { value: "kimi", label: "Kimi", model: "moonshot-v1-8k" },
];

export const DEFAULT_PROVIDER: AiProvider = "deepseek";

export interface AiSettings {
  provider: AiProvider;
  api_key: string;
}

export interface AiEvent {
  kind: "chunk" | "done" | "error";
  text?: string;
  message?: string;
}
