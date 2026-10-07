<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { onMounted, onUnmounted, ref } from "vue";

const count = ref(0);
const busy = ref(false);
let unlisten: UnlistenFn[] = [];

async function refresh() {
  count.value = await invoke<number>("get_reminder_pending_count");
  if (count.value === 0) {
    await getCurrentWindow().hide();
  }
}

async function run(command: string) {
  if (busy.value) return;
  busy.value = true;
  try {
    await invoke(command);
    await getCurrentWindow().hide();
  } finally {
    busy.value = false;
  }
}

onMounted(async () => {
  await refresh();
  unlisten.push(await listen("reminder-shown", () => void refresh()));
  unlisten.push(await listen("todos-changed", () => void refresh()));
});

onUnmounted(() => {
  unlisten.forEach((fn) => fn());
  unlisten = [];
});
</script>

<template>
  <div class="dialog" data-tauri-drag-region="deep">
    <div class="glass">
      <h2>待办提醒</h2>
      <p class="message">
        还有 <span class="count">{{ count }}</span> 个待办未完成
      </p>
      <div class="actions">
        <button class="btn primary" :disabled="busy" @click="run('complete_all_pending')">
          全部已完成
        </button>
        <button class="btn" :disabled="busy" @click="run('defer_all_pending_to_tomorrow')">
          转移至明天
        </button>
        <button class="btn" :disabled="busy" @click="run('snooze_reminder')">
          半小时后再提醒
        </button>
        <button class="btn muted" :disabled="busy" @click="run('dismiss_reminder_today')">
          关闭（今天不再提醒）
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog {
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
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 22px 24px;
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

.message {
  margin: 0;
  text-align: center;
  font-size: 14px;
  line-height: 1.6;
  color: var(--ink-4);
}

.count {
  color: var(--gold);
  font-size: 20px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  padding: 0 2px;
}

.actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.btn {
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid rgba(128, 128, 128, 0.25);
  background: rgba(128, 128, 128, 0.12);
  color: inherit;
  font-family: inherit;
  font-size: 13px;
  line-height: 1.3;
  cursor: pointer;
  transition: border-color 0.2s, color 0.2s, background 0.2s, transform 0.15s;
}

.btn:hover:not(:disabled) {
  border-color: var(--gold);
  color: var(--gold);
}

.btn:active:not(:disabled) {
  transform: scale(0.97);
}

.btn:disabled {
  opacity: 0.5;
  cursor: default;
}

.btn.primary {
  border: none;
  border-color: transparent;
  background: linear-gradient(135deg, #edcfa6, var(--gold));
  color: #6b5433;
  font-weight: 500;
  box-shadow: 0 10px 20px -14px rgba(201, 160, 110, 0.9);
}

.btn.primary:hover:not(:disabled) {
  color: #6b5433;
  filter: brightness(1.03);
}

.btn.muted {
  font-size: 12px;
  opacity: 0.85;
}

@media (prefers-color-scheme: dark) {
  .btn {
    background: rgba(128, 128, 128, 0.2);
  }
}
</style>
