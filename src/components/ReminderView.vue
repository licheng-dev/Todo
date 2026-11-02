<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import type { Todo } from "../types";
import { dateKey } from "../utils/date";
import { useToday } from "../composables/useToday";

const props = defineProps<{ active: boolean; todos: Todo[] }>();
const emit = defineEmits<{ (e: "close"): void }>();

const count = ref(0);
const busy = ref(false);
const today = useToday();

const todayDone = computed(() =>
  props.todos
    .filter((t) => t.completed_at && dateKey(t.completed_at) === today.value)
    .sort((a, b) => (a.completed_at ?? 0) - (b.completed_at ?? 0))
);

async function loadCount() {
  count.value = await invoke<number>("get_reminder_pending_count");
}

async function dismiss() {
  if (busy.value) return;
  busy.value = true;
  try {
    await invoke("dismiss_reminder_today");
    emit("close");
  } finally {
    busy.value = false;
  }
}

function close() {
  emit("close");
}

function onKeyDown(event: KeyboardEvent) {
  if (!props.active) return;
  if (event.key === "Escape") close();
}

onMounted(() => {
  void loadCount();
  window.addEventListener("keydown", onKeyDown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeyDown);
});

watch(
  () => props.active,
  (active) => {
    if (active) void loadCount();
  }
);
</script>

<template>
  <div class="reminder">
    <div class="glass">
      <header class="head">
        <button class="head-back" type="button" aria-label="返回" @click="close">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M14.5 6.5 9 12l5.5 5.5" />
          </svg>
          <span>返回</span>
        </button>
        <h2>待办提醒</h2>
        <span class="head-spacer"></span>
      </header>

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

      <footer class="foot">
        <button class="btn primary" type="button" :disabled="busy" @click="dismiss">
          今日不再提醒
        </button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.reminder {
  width: 100%;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.glass {
  width: 100%;
  flex: 1;
  min-height: 0;
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

.head {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
}

.head-spacer {
  display: block;
}

.head-back {
  justify-self: start;
  display: inline-flex;
  align-items: center;
  gap: 2px;
  padding: 4px 12px 4px 8px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.6);
  background: linear-gradient(
    135deg,
    rgba(255, 255, 255, 0.42),
    rgba(255, 255, 255, 0.16)
  );
  -webkit-backdrop-filter: blur(10px) saturate(130%);
  backdrop-filter: blur(10px) saturate(130%);
  box-shadow:
    0 12px 28px -22px rgba(201, 190, 178, 0.95),
    inset 0 1px 0 rgba(255, 255, 255, 0.6);
  color: var(--ink-4);
  font-family: inherit;
  font-size: 12px;
  line-height: 1;
  cursor: pointer;
  transition: color 180ms ease, transform 180ms ease, box-shadow 180ms ease;
}

.head-back:hover {
  color: var(--gold);
  box-shadow:
    0 16px 30px -20px rgba(201, 190, 178, 1),
    inset 0 1px 0 rgba(255, 255, 255, 0.7);
}

.head-back:active {
  transform: scale(0.97);
}

.head-back svg {
  width: 14px;
  height: 14px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
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
  -webkit-user-select: text;
  user-select: text;
}

.done-empty {
  margin: 0;
  font-size: 12.5px;
  color: var(--ink-3);
  opacity: 0.8;
}

.foot {
  margin-top: auto;
  display: flex;
  justify-content: center;
}

.btn {
  padding: 9px 20px;
  border-radius: 10px;
  font-family: inherit;
  font-size: 13px;
  line-height: 1.3;
  cursor: pointer;
  transition: filter 0.2s, transform 0.15s;
}

.btn.primary {
  border: none;
  background: linear-gradient(135deg, #edcfa6, var(--gold));
  color: #6b5433;
  font-weight: 500;
  box-shadow: 0 10px 20px -14px rgba(201, 160, 110, 0.9);
}

.btn.primary:hover:not(:disabled) {
  filter: brightness(1.03);
}

.btn:active:not(:disabled) {
  transform: scale(0.97);
}

.btn:disabled {
  opacity: 0.5;
  cursor: default;
}
</style>
