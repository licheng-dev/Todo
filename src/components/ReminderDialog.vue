<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { computed, onMounted, onUnmounted, ref } from "vue";
import type { Todo } from "../types";
import { dateKey } from "../utils/date";
import { useToday } from "../composables/useToday";

const count = ref(0);
const todos = ref<Todo[]>([]);
const busy = ref(false);
let unlisten: UnlistenFn[] = [];

const today = useToday();

const todayDone = computed(() =>
  todos.value
    .filter((t) => t.completed_at && dateKey(t.completed_at) === today.value)
    .sort((a, b) => (a.completed_at ?? 0) - (b.completed_at ?? 0))
);

async function refresh() {
  const [pending, list] = await Promise.all([
    invoke<number>("get_reminder_pending_count"),
    invoke<Todo[]>("get_todos"),
  ]);
  count.value = pending;
  todos.value = list;
  if (pending === 0) {
    await getCurrentWindow().hide();
  }
}

async function dismiss() {
  if (busy.value) return;
  busy.value = true;
  try {
    await invoke("dismiss_reminder_today");
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

      <div class="done">
        <p class="done-title">今日已完成 {{ todayDone.length }} 条</p>
        <ul v-if="todayDone.length" class="done-list">
          <li v-for="item in todayDone" :key="item.id" class="done-item">
            <svg class="tick" viewBox="0 0 24 24" aria-hidden="true">
              <polyline points="4 12.5 9.5 18 20 6.5" />
            </svg>
            <span class="txt">{{ item.text }}</span>
          </li>
        </ul>
        <p v-else class="done-empty">今日还没有已完成事项</p>
      </div>

      <div class="actions">
        <button class="btn muted" :disabled="busy" @click="dismiss">
          今日不再提醒
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
  max-height: 100%;
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 20px 22px;
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

.done {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-height: 0;
}

.done-title {
  margin: 0;
  font-size: 12px;
  color: var(--ink-3);
}

.done-list {
  list-style: none;
  margin: 0;
  padding: 10px 12px;
  max-height: 200px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 7px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.28);
  border: 1px solid rgba(255, 255, 255, 0.45);
}

.done-item {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 12.5px;
  line-height: 1.4;
  color: var(--ink-4);
}

.tick {
  flex-shrink: 0;
  width: 13px;
  height: 13px;
  fill: none;
  stroke: var(--gold);
  stroke-width: 2.2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.txt {
  word-break: break-word;
}

.done-empty {
  margin: 0;
  font-size: 12.5px;
  color: var(--ink-3);
  opacity: 0.8;
}

.actions {
  display: flex;
  justify-content: center;
}

.btn {
  padding: 9px 20px;
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
