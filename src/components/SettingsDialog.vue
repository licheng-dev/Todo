<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, onUnmounted, ref } from "vue";

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
  if (!recording.value) return;
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

onMounted(async () => {
  alwaysOnTop.value = await invoke<boolean>("get_always_on_top");
  addShortcut.value = await invoke<ShortcutSetting>("get_add_shortcut");
  window.addEventListener("keydown", onKeyDown, true);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeyDown, true);
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

      <button class="close-btn" @click="close">关闭</button>
    </div>
  </div>
</template>

<style scoped>
.settings {
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
