<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, onUnmounted, ref } from "vue";
import {
  DEFAULT_PROVIDER,
  PROVIDERS,
  type AiProvider,
  type AiSettings,
} from "../ai/config";

interface ShortcutSetting {
  code: string;
  meta: boolean;
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
}

const alwaysOnTop = ref(false);
const busy = ref(false);

const addShortcut = ref<ShortcutSetting | null>(null);
const recording = ref(false);
const shortcutBusy = ref(false);
const shortcutError = ref("");

const reminderEnabled = ref(false);
const reminderTime = ref("17:00");
const reminderBusy = ref(false);

const aiProvider = ref<AiProvider>(DEFAULT_PROVIDER);
const aiApiKey = ref("");
const aiBusy = ref(false);
const aiSaved = ref(false);
let aiSavedTimer: number | null = null;

interface UpdateInfo {
  current: string;
  latest: string;
  has_update: boolean;
  url: string;
}

type UpdateState = "idle" | "checking" | "latest" | "available" | "error";

const version = ref("");
const updateState = ref<UpdateState>("idle");
const updateUrl = ref("");
const updateError = ref("");
const updateMessage = ref("");
let updateTimer: number | null = null;

function clearUpdateMessageSoon() {
  if (updateTimer) window.clearTimeout(updateTimer);
  updateTimer = window.setTimeout(() => {
    updateMessage.value = "";
    updateTimer = null;
  }, 3000);
}

const KEY_LABELS: Record<string, string> = {
  Comma: ",",
  Period: ".",
  Slash: "/",
  Semicolon: ";",
  Quote: "'",
  BracketLeft: "[",
  BracketRight: "]",
  Backslash: "\\",
  Backquote: "`",
  Minus: "-",
  Equal: "=",
  Space: "空格",
  Enter: "↩",
  Tab: "⇥",
};

function keyLabel(code: string): string {
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  return KEY_LABELS[code] ?? code;
}

const shortcutLabel = computed(() => {
  const s = addShortcut.value;
  if (!s) return "";
  let label = "";
  if (s.ctrl) label += "⌃";
  if (s.alt) label += "⌥";
  if (s.shift) label += "⇧";
  if (s.meta) label += "⌘";
  return label + keyLabel(s.code);
});

async function sync() {
  if (busy.value) return;
  busy.value = true;
  try {
    await invoke("set_always_on_top", { enabled: alwaysOnTop.value });
  } finally {
    busy.value = false;
  }
}

async function toggle() {
  alwaysOnTop.value = !alwaysOnTop.value;
  await sync();
}

function startRecording() {
  shortcutError.value = "";
  recording.value = true;
}

async function saveShortcut(next: ShortcutSetting) {
  shortcutBusy.value = true;
  try {
    await invoke("set_add_shortcut", { shortcut: next });
    addShortcut.value = next;
    recording.value = false;
    shortcutError.value = "";
  } catch (err) {
    shortcutError.value = String(err);
  } finally {
    shortcutBusy.value = false;
  }
}

function onKeyDown(event: KeyboardEvent) {
  if (!recording.value) {
    if (event.key === "Escape") void close();
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  if (event.key === "Escape") {
    recording.value = false;
    return;
  }
  if (["Meta", "Shift", "Control", "Alt", "CapsLock"].includes(event.key)) return;
  void saveShortcut({
    code: event.code,
    meta: event.metaKey,
    ctrl: event.ctrlKey,
    alt: event.altKey,
    shift: event.shiftKey,
  });
}

async function close() {
  await getCurrentWindow().hide();
}

async function saveReminder() {
  if (reminderBusy.value) return;
  reminderBusy.value = true;
  try {
    await invoke("set_reminder_settings", {
      enabled: reminderEnabled.value,
      time: reminderTime.value,
    });
  } finally {
    reminderBusy.value = false;
  }
}

async function toggleReminder() {
  reminderEnabled.value = !reminderEnabled.value;
  await saveReminder();
}

async function saveAi() {
  if (aiBusy.value) return;
  aiBusy.value = true;
  try {
    await invoke("set_ai_settings", {
      provider: aiProvider.value,
      apiKey: aiApiKey.value.trim(),
    });
    aiSaved.value = true;
    if (aiSavedTimer) window.clearTimeout(aiSavedTimer);
    aiSavedTimer = window.setTimeout(() => (aiSaved.value = false), 1600);
  } finally {
    aiBusy.value = false;
  }
}

async function checkUpdate() {
  if (updateState.value === "checking") return;
  if (updateTimer) {
    window.clearTimeout(updateTimer);
    updateTimer = null;
  }
  updateState.value = "checking";
  updateError.value = "";
  updateMessage.value = "检查中…";
  try {
    const info = await invoke<UpdateInfo>("check_update");
    updateUrl.value = info.url;
    updateState.value = info.has_update ? "available" : "latest";
    updateMessage.value = info.has_update
      ? `发现新版本 ${info.latest}`
      : "已是最新版本";
  } catch (err) {
    updateError.value = String(err);
    updateState.value = "error";
    updateMessage.value = updateError.value;
  }
  clearUpdateMessageSoon();
}

async function goDownload() {
  if (!updateUrl.value) return;
  try {
    await invoke("open_url", { url: updateUrl.value });
  } catch (err) {
    updateError.value = String(err);
    updateState.value = "error";
  }
}

onMounted(async () => {
  alwaysOnTop.value = await invoke<boolean>("get_always_on_top");
  addShortcut.value = await invoke<ShortcutSetting>("get_add_shortcut");
  const reminder = await invoke<{ enabled: boolean; time: string }>(
    "get_reminder_settings"
  );
  reminderEnabled.value = reminder.enabled;
  reminderTime.value = reminder.time;
  const ai = await invoke<AiSettings>("get_ai_settings");
  aiProvider.value = ai.provider;
  aiApiKey.value = ai.api_key;
  version.value = await getVersion();
  window.addEventListener("keydown", onKeyDown, true);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeyDown, true);
  if (aiSavedTimer) window.clearTimeout(aiSavedTimer);
  if (updateTimer) window.clearTimeout(updateTimer);
});
</script>

<template>
  <div class="settings" data-tauri-drag-region="deep">
    <div class="glass">
      <h2>设置</h2>

      <div class="row">
        <span class="label">窗口置顶</span>
        <button
          class="toggle"
          :class="{ on: alwaysOnTop }"
          @click="toggle"
          :disabled="busy"
        >
          <span class="knob"></span>
        </button>
      </div>

      <div class="shortcut">
        <div class="row">
          <span class="label">添加待办</span>
          <button
            class="shortcut-key"
            :class="{ recording }"
            type="button"
            :disabled="shortcutBusy"
            @click="startRecording"
          >
            {{ recording ? "按下组合键…" : shortcutLabel || "未设置" }}
          </button>
        </div>
        <p v-if="shortcutError" class="shortcut-error">{{ shortcutError }}</p>
        <p v-else class="shortcut-hint">
          {{ recording ? "按下新的组合键，Esc 取消" : "点击后按下新的组合键" }}
        </p>
      </div>

      <div class="divider"></div>

      <div class="reminder">
        <div class="row">
          <span class="label">待办提醒</span>
          <button
            class="toggle"
            :class="{ on: reminderEnabled }"
            @click="toggleReminder"
            :disabled="reminderBusy"
          >
            <span class="knob"></span>
          </button>
        </div>
        <div v-if="reminderEnabled" class="row">
          <span class="label">提醒时间</span>
          <input
            v-model="reminderTime"
            class="time-input"
            type="time"
            :disabled="reminderBusy"
            @change="saveReminder"
          />
        </div>
        <p class="reminder-hint">
          {{
            reminderEnabled
              ? "当天有待办时，会在设定时间弹出提醒"
              : "开启后每天在设定时间提醒未完成的待办"
          }}
        </p>
      </div>

      <div class="divider"></div>

      <div class="ai">
        <div class="row">
          <span class="label">AI 提供商</span>
          <select
            v-model="aiProvider"
            class="select"
            :disabled="aiBusy"
            @change="saveAi"
          >
            <option v-for="p in PROVIDERS" :key="p.value" :value="p.value">
              {{ p.label }}
            </option>
          </select>
        </div>
        <div class="row">
          <span class="label">API Key</span>
          <input
            v-model="aiApiKey"
            class="input"
            type="password"
            spellcheck="false"
            autocomplete="off"
            placeholder="填写 API Key"
            :disabled="aiBusy"
            @blur="saveAi"
            @keydown.enter="saveAi"
          />
        </div>
        <p class="ai-hint">
          {{ aiBusy ? "保存中…" : aiSaved ? "已保存" : "Key 仅保存在本地" }}
        </p>
      </div>

      <div class="divider"></div>

      <div class="update">
        <div class="row">
          <span class="label">版本</span>
          <span class="version-group">
            <span class="version">
              <span
                v-if="updateMessage"
                class="update-status"
                :class="{ error: updateState === 'error' }"
              >
                {{ updateMessage }}
              </span>
              v{{ version || "—" }}
            </span>
            <button
              v-if="updateState === 'available'"
              class="download-btn"
              @click="goDownload"
            >
              前往下载
            </button>
            <button
              v-else
              class="update-btn"
              :disabled="updateState === 'checking'"
              @click="checkUpdate"
            >
              检查更新
            </button>
          </span>
        </div>
      </div>

      <button class="close-btn" @click="close">关闭</button>
    </div>
  </div>
</template>

<style scoped>
.settings {
  width: 100vw;
  height: 90vh;
  margin: 5vh 0;
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
  max-height: 100%;
  overflow-y: auto;
  padding: 20px 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
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

h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  text-align: center;
}

.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.label {
  font-size: 14px;
}

.toggle {
  width: 44px;
  height: 24px;
  border-radius: 12px;
  border: none;
  cursor: pointer;
  background: rgba(128, 128, 128, 0.3);
  position: relative;
  transition: background 0.2s;
}

.toggle.on {
  background: #34c759;
}

.knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: #fff;
  transition: transform 0.2s;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.toggle.on .knob {
  transform: translateX(20px);
}

.shortcut {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.shortcut-key {
  min-width: 88px;
  padding: 6px 12px;
  border-radius: 8px;
  border: 1px solid rgba(128, 128, 128, 0.25);
  background: rgba(128, 128, 128, 0.12);
  color: inherit;
  font-family: inherit;
  font-size: 13px;
  letter-spacing: 1px;
  cursor: pointer;
  transition: border-color 0.2s, color 0.2s, background 0.2s;
}

.shortcut-key:hover:not(:disabled) {
  border-color: var(--gold);
  color: var(--gold);
}

.shortcut-key.recording {
  border-color: var(--gold);
  color: var(--gold);
  background: rgba(232, 196, 155, 0.18);
}

.shortcut-key:disabled {
  cursor: default;
  opacity: 0.6;
}

.shortcut-hint,
.shortcut-error {
  margin: 0;
  font-size: 11px;
  line-height: 1.4;
}

.shortcut-hint {
  opacity: 0.55;
}

.shortcut-error {
  color: #c0564f;
}

.divider {
  height: 1px;
  background: rgba(150, 145, 140, 0.22);
}

.reminder {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.time-input {
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid rgba(128, 128, 128, 0.25);
  background: rgba(128, 128, 128, 0.12);
  color: inherit;
  font-family: inherit;
  font-size: 13px;
  outline: none;
  cursor: pointer;
  transition: border-color 0.2s;
}

.time-input:focus {
  border-color: var(--gold);
}

.time-input:disabled {
  opacity: 0.6;
}

.reminder-hint {
  margin: 0;
  font-size: 11px;
  line-height: 1.4;
  opacity: 0.55;
}

.ai {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.select,
.input {
  width: 150px;
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid rgba(128, 128, 128, 0.25);
  background: rgba(128, 128, 128, 0.12);
  color: inherit;
  font-family: inherit;
  font-size: 13px;
  outline: none;
  transition: border-color 0.2s;
}

.select {
  cursor: pointer;
}

.input {
  -webkit-user-select: text;
  user-select: text;
}

.select:focus,
.input:focus {
  border-color: var(--gold);
}

.select:disabled,
.input:disabled {
  opacity: 0.6;
}

.ai-hint {
  margin: 0;
  font-size: 11px;
  line-height: 1.4;
  opacity: 0.55;
}

.update {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.version {
  display: inline-flex;
  align-items: baseline;
  gap: 8px;
  font-size: 13px;
  opacity: 0.75;
  font-variant-numeric: tabular-nums;
}

.version-group {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.version-group .update-btn,
.version-group .download-btn {
  min-width: 0;
  padding: 4px 10px;
  font-size: 12px;
}

.update-status {
  font-size: 12px;
  opacity: 0.6;
  transition: opacity 0.3s;
}

.update-status.error {
  color: #c0564f;
  opacity: 1;
}

.update-btn,
.download-btn {
  align-self: center;
  min-width: 88px;
  padding: 6px 12px;
  border-radius: 8px;
  border: 1px solid rgba(128, 128, 128, 0.25);
  background: rgba(128, 128, 128, 0.12);
  color: inherit;
  font-family: inherit;
  font-size: 13px;
  cursor: pointer;
  transition: border-color 0.2s, color 0.2s, background 0.2s;
}

.update-btn:hover:not(:disabled),
.download-btn:hover {
  border-color: var(--gold);
  color: var(--gold);
}

.download-btn {
  border-color: var(--gold);
  color: var(--gold);
  background: rgba(232, 196, 155, 0.18);
}

.update-btn:disabled {
  cursor: default;
  opacity: 0.6;
}

.close-btn {
  align-self: center;
  padding: 7px 24px;
  border-radius: 8px;
  border: none;
  background: rgba(128, 128, 128, 0.15);
  color: inherit;
  font-size: 13px;
  cursor: pointer;
  margin-top: 4px;
}
</style>
