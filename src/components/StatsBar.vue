<script setup lang="ts">
import { ref } from "vue";

defineProps<{
  date: string;
  mode: "today" | "history";
  leftover: number;
  pending: number;
  done: number;
  filter: "all" | "today" | "leftover" | "done";
}>();

defineEmits<{
  (e: "date-click"): void;
  (e: "done-click"): void;
  (e: "pending-click"): void;
  (e: "leftover-click"): void;
}>();

const doneEl = ref<HTMLElement | null>(null);
defineExpose({ doneEl });
</script>

<template>
  <footer class="statsbar">
    <button
      class="item date"
      type="button"
      data-calendar-trigger
      aria-label="选择日期查看已办"
      @click="$emit('date-click')"
    >
      <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
        <rect x="3.5" y="5" width="17" height="15.5" rx="3.5" />
        <path d="M3.5 9.5h17" />
        <path d="M8 3v4M16 3v4" />
      </svg>
      <span>{{ date }}</span>
      <svg class="caret" viewBox="0 0 24 24" aria-hidden="true">
        <path d="M7 10l5 5 5-5" />
      </svg>
    </button>

    <span class="divider"></span>

    <div class="group">
      <template v-if="mode === 'today'">
        <button
          class="item stat-btn"
          :class="{ active: filter === 'leftover' }"
          type="button"
          aria-label="筛选遗留待办"
          @click="$emit('leftover-click')"
        >
          <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
            <circle cx="12" cy="12" r="8.5" />
            <path d="M12 7.5V12l3 2" />
          </svg>
          <span>遗留待办 {{ leftover }}</span>
        </button>
        <button
          class="item stat-btn"
          :class="{ active: filter === 'today' }"
          type="button"
          aria-label="筛选今日待办"
          @click="$emit('pending-click')"
        >
          <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
            <circle cx="12" cy="12" r="8.5" />
          </svg>
          <span>当日待办 {{ pending }}</span>
        </button>
        <button
          ref="doneEl"
          class="item stat-btn"
          :class="{ active: filter === 'done' }"
          type="button"
          aria-label="筛选当日已办"
          @click="$emit('done-click')"
        >
          <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
            <circle cx="12" cy="12" r="8.5" />
            <path d="M8.5 12.2l2.4 2.4 4.6-4.8" />
          </svg>
          <span>当日已办 {{ done }}</span>
        </button>
      </template>
      <div v-else class="item">
        <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="12" cy="12" r="8.5" />
          <path d="M8.5 12.2l2.4 2.4 4.6-4.8" />
        </svg>
        <span>当日已办 {{ done }}</span>
      </div>
    </div>
  </footer>
</template>

<style scoped>
.statsbar {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: clamp(8px, 1.6vw, 20px);
  margin: 0 12px clamp(12px, 3vh, 24px);
  height: clamp(52px, 9vh, 74px);
  padding: 0 clamp(16px, 3vw, 24px);
  border-radius: var(--bar-radius);
  background: linear-gradient(
    135deg,
    rgba(255, 255, 255, 0.34),
    rgba(255, 255, 255, 0.14)
  );
  -webkit-backdrop-filter: blur(10px) saturate(130%);
  backdrop-filter: blur(10px) saturate(130%);
  border: 1px solid var(--glass-border);
  box-shadow:
    0 18px 44px -22px rgba(201, 190, 178, 0.75),
    inset 0 1px 0 rgba(255, 255, 255, 0.6);
  color: var(--ink-4);
  font-size: clamp(12px, 2.4vw, 20px);
  white-space: nowrap;
}

.item {
  display: flex;
  align-items: center;
  gap: clamp(6px, 1vw, 12px);
}

.date {
  border: none;
  background: transparent;
  padding: 0;
  font: inherit;
  color: inherit;
  cursor: pointer;
  border-radius: 999px;
  transition: color 180ms ease;
}

.stat-btn {
  border: none;
  background: transparent;
  padding: 0;
  font: inherit;
  color: inherit;
  cursor: pointer;
  transition: color 180ms ease;
}

.date:hover,
.stat-btn:hover {
  color: var(--gold);
}

.stat-btn.active {
  color: var(--gold);
}

.stat-btn.active .icon {
  stroke: var(--gold);
}

.date:hover .caret {
  opacity: 1;
  transform: translateY(1px);
}

.group {
  display: flex;
  align-items: center;
  gap: clamp(16px, 3vw, 40px);
}

.divider {
  width: 1px;
  height: clamp(24px, 4vh, 40px);
  background: rgba(150, 145, 140, 0.25);
}

.icon {
  width: clamp(15px, 1.9vw, 22px);
  height: clamp(15px, 1.9vw, 22px);
  flex-shrink: 0;
  fill: none;
  stroke: var(--icon);
  stroke-width: 1.6;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.caret {
  width: clamp(11px, 1.4vw, 15px);
  height: clamp(11px, 1.4vw, 15px);
  flex-shrink: 0;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
  opacity: 0.5;
  transition: opacity 180ms ease, transform 180ms ease;
}

@media (max-width: 520px) {
  .statsbar {
    font-size: 12px;
  }
  .divider {
    display: none;
  }
}
</style>
