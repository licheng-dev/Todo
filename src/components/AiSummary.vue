<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Todo } from "../types";
import { dateKey, formatDateLabel, todayKey } from "../utils/date";
import { SYSTEM_PROMPT } from "../ai/prompt";
import type { AiEvent, AiSettings } from "../ai/config";

const todos = ref<Todo[]>([]);
const summary = ref("");
const running = ref(false);
const editing = ref(false);
const error = ref("");
const copied = ref(false);
const hasKey = ref<boolean | null>(null);
const bodyEl = ref<HTMLElement | null>(null);
const textareaEl = ref<HTMLTextAreaElement | null>(null);
let unlisten: UnlistenFn | null = null;
let unlistenTodos: UnlistenFn | null = null;
let unlistenFocus: UnlistenFn | null = null;
let copyTimer: number | null = null;

const today = todayKey();
const dateLabel = computed(() => formatDateLabel(today));

const todayDone = computed(() =>
  todos.value
    .filter((t) => t.completed_at && dateKey(t.completed_at) === today)
    .sort((a, b) => (a.completed_at ?? 0) - (b.completed_at ?? 0))
);

const canGenerate = computed(
  () => hasKey.value === true && todayDone.value.length > 0
);

async function loadTodos() {
  todos.value = await invoke<Todo[]>("get_todos");
}

async function loadAiSettings() {
  try {
    const settings = await invoke<AiSettings>("get_ai_settings");
    hasKey.value = !!settings.api_key.trim();
  } catch {
    hasKey.value = false;
  }
}

watch(summary, async () => {
  if (!running.value) return;
  await nextTick();
  if (bodyEl.value) bodyEl.value.scrollTop = bodyEl.value.scrollHeight;
});

function buildUserPrompt(): string {
  const lines = todayDone.value.map((t, i) => `${i + 1}. ${t.text}`);
  return `日期：${dateLabel.value}\n今日已完成事项（共 ${lines.length} 条）：\n${lines.join("\n")}`;
}

async function generate() {
  if (running.value || !canGenerate.value) return;
  editing.value = false;
  summary.value = "";
  error.value = "";
  copied.value = false;
  running.value = true;
  try {
    await invoke("start_summary", {
      systemPrompt: SYSTEM_PROMPT,
      userPrompt: buildUserPrompt(),
    });
  } catch (err) {
    running.value = false;
    error.value = String(err);
  }
}

async function startEdit() {
  if (!summary.value) return;
  editing.value = true;
  await nextTick();
  const el = textareaEl.value;
  if (el) {
    el.focus();
    el.setSelectionRange(el.value.length, el.value.length);
  }
}

function finishEdit() {
  editing.value = false;
}

async function stop() {
  await invoke("cancel_summary");
}

async function copy() {
  if (!summary.value) return;
  await navigator.clipboard.writeText(summary.value);
  copied.value = true;
  if (copyTimer) window.clearTimeout(copyTimer);
  copyTimer = window.setTimeout(() => (copied.value = false), 1600);
}

async function openSettings() {
  await invoke("open_settings");
}

async function close() {
  await getCurrentWindow().hide();
}

onMounted(async () => {
  await loadTodos();
  await loadAiSettings();
  unlisten = await listen<AiEvent>("ai-summary", (event) => {
    const payload = event.payload;
    if (payload.kind === "chunk") {
      summary.value += payload.text ?? "";
    } else if (payload.kind === "error") {
      error.value = payload.message ?? "生成失败";
      running.value = false;
    } else {
      running.value = false;
    }
  });
  unlistenTodos = await listen<Todo[]>("todos-changed", (event) => {
    todos.value = event.payload;
  });
  unlistenFocus = await getCurrentWindow().onFocusChanged(({ payload }) => {
    if (payload) {
      void loadTodos();
      void loadAiSettings();
    }
  });
});

onUnmounted(() => {
  unlisten?.();
  unlistenTodos?.();
  unlistenFocus?.();
  if (copyTimer) window.clearTimeout(copyTimer);
});
</script>

<template>
  <div class="ai" data-tauri-drag-region="deep">
    <div class="glass">
      <header class="head" data-tauri-drag-region="deep">
        <div class="titles" data-tauri-drag-region="deep">
          <h2>AI 日报</h2>
          <span class="date">{{ dateLabel }}</span>
        </div>
        <button class="close" type="button" aria-label="关闭" @click="close">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M7 7l10 10M17 7L7 17" />
          </svg>
        </button>
      </header>

      <p class="meta">今日已办 {{ todayDone.length }} 条</p>

      <p v-if="hasKey === false" class="notice">
        尚未配置 AI 密钥，点击
        <button class="link" type="button" @click="openSettings">前往设置</button>
        填写后即可生成日报。
      </p>

      <section ref="bodyEl" class="body">
        <div v-if="error" class="state error">
          <p>{{ error }}</p>
          <button class="link" type="button" @click="openSettings">打开设置</button>
        </div>

        <textarea
          v-else-if="editing"
          ref="textareaEl"
          v-model="summary"
          class="summary-input"
          spellcheck="false"
        ></textarea>

        <p v-else-if="running || summary" class="summary">{{ summary }}<span v-if="running" class="cursor"></span></p>

        <div v-else-if="todayDone.length" class="preview">
          <p v-for="(t, i) in todayDone" :key="t.id" class="preview-item">
            <span class="idx">{{ i + 1 }}</span>
            <span class="txt">{{ t.text }}</span>
          </p>
        </div>

        <p v-else class="state empty">今日还没有已完成事项</p>
      </section>

      <footer class="foot">
        <button v-if="running" class="btn ghost" type="button" @click="stop">停止</button>

        <template v-else-if="summary">
          <button
            class="btn ghost"
            type="button"
            @click="editing ? finishEdit() : startEdit()"
          >
            {{ editing ? "完成" : "编辑" }}
          </button>
          <button
            class="btn ghost"
            type="button"
            :disabled="editing || !summary"
            @click="copy"
          >
            {{ copied ? "已复制" : "复制" }}
          </button>
          <button
            class="btn primary"
            type="button"
            :disabled="!canGenerate"
            @click="generate"
          >
            重新生成
          </button>
        </template>

        <button
          v-else
          class="btn primary"
          type="button"
          :disabled="!canGenerate"
          @click="generate"
        >
          生成日报
        </button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.ai {
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  border-radius: var(--page-radius);
  overflow: hidden;
  background-color: var(--bg-a);
  background-image: url("../assets/bg.png");
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
  color: var(--ink);
}

.glass {
  width: 100%;
  height: 100%;
  padding: 18px 22px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  border-radius: 16px;
  user-select: none;
  background: linear-gradient(
    135deg,
    rgba(255, 255, 255, 0.3),
    rgba(255, 255, 255, 0.12)
  );
  border: 1px solid rgba(255, 255, 255, 0.55);
  box-shadow:
    0 12px 28px -22px rgba(201, 190, 178, 0.95),
    inset 0 1px 0 rgba(255, 255, 255, 0.6);
  -webkit-backdrop-filter: blur(8px) saturate(130%);
  backdrop-filter: blur(8px) saturate(130%);
}

.head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}

.titles {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.date {
  font-size: 11.5px;
  opacity: 0.6;
}

.close {
  width: 24px;
  height: 24px;
  padding: 0;
  border: none;
  border-radius: 50%;
  background: rgba(128, 128, 128, 0.12);
  color: var(--ink-4);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 180ms ease, color 180ms ease;
}

.close:hover {
  background: rgba(128, 128, 128, 0.22);
  color: var(--gold);
}

.close svg {
  width: 13px;
  height: 13px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
}

.meta {
  margin: 0;
  font-size: 12px;
  opacity: 0.65;
}

.notice {
  margin: 0;
  font-size: 12px;
  line-height: 1.5;
  color: #c0564f;
}

.body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 12px 14px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.28);
  border: 1px solid rgba(255, 255, 255, 0.45);
  -webkit-user-select: text;
  user-select: text;
}

.summary {
  margin: 0;
  font-size: 13px;
  line-height: 1.75;
  white-space: pre-wrap;
  word-break: break-word;
}

.summary-input {
  width: 100%;
  height: 100%;
  min-height: 100%;
  margin: 0;
  padding: 0;
  border: none;
  outline: none;
  background: transparent;
  resize: none;
  color: var(--ink);
  font-family: inherit;
  font-size: 13px;
  line-height: 1.75;
  white-space: pre-wrap;
  word-break: break-word;
  overflow: auto;
}

.cursor {
  display: inline-block;
  width: 7px;
  height: 14px;
  margin-left: 2px;
  vertical-align: -2px;
  background: var(--gold);
  border-radius: 2px;
  animation: blink 1s steps(2, start) infinite;
}

@keyframes blink {
  to {
    visibility: hidden;
  }
}

.preview {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.preview-item {
  margin: 0;
  display: flex;
  gap: 8px;
  font-size: 12.5px;
  line-height: 1.5;
  color: var(--ink-4);
}

.idx {
  flex-shrink: 0;
  width: 17px;
  height: 17px;
  margin-top: 1px;
  border-radius: 50%;
  background: rgba(232, 196, 155, 0.35);
  color: var(--ink-4);
  font-size: 10.5px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.state {
  margin: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  text-align: center;
  font-size: 13px;
  color: var(--ink-3);
}

.state.error {
  color: #c0564f;
}

.link {
  border: none;
  background: none;
  color: var(--gold);
  font-size: 12.5px;
  cursor: pointer;
  padding: 0;
  text-decoration: underline;
}

.foot {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
}

.btn {
  padding: 7px 18px;
  border-radius: 8px;
  font-size: 13px;
  cursor: pointer;
  transition: opacity 180ms ease, background 180ms ease, color 180ms ease;
}

.btn.ghost {
  border: 1px solid rgba(128, 128, 128, 0.25);
  background: rgba(128, 128, 128, 0.1);
  color: inherit;
}

.btn.ghost:hover:not(:disabled) {
  border-color: var(--gold);
  color: var(--gold);
}

.btn.primary {
  border: none;
  background: linear-gradient(135deg, #eccfa9, #e0b184);
  color: #5a4632;
  font-weight: 500;
  box-shadow: 0 10px 20px -14px rgba(201, 160, 110, 1);
}

.btn.primary:hover:not(:disabled) {
  filter: brightness(1.04);
}

.btn:disabled {
  opacity: 0.5;
  cursor: default;
}
</style>
