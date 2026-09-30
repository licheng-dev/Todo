<script setup lang="ts">
import { ref } from "vue";

defineProps<{
  date: string;
  pending: number;
  done: number;
}>();

const doneEl = ref<HTMLElement | null>(null);
defineExpose({ doneEl });
</script>

<template>
  <footer class="statsbar">
    <div class="item">
      <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
        <rect x="3.5" y="5" width="17" height="15.5" rx="3.5" />
        <path d="M3.5 9.5h17" />
        <path d="M8 3v4M16 3v4" />
      </svg>
      <span>{{ date }}</span>
    </div>

    <span class="divider"></span>

    <div class="group">
      <div class="item">
        <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="12" cy="12" r="8.5" />
          <path d="M8.5 12.2l2.4 2.4 4.6-4.8" />
        </svg>
        <span>今日待办 {{ pending }}</span>
      </div>
      <div ref="doneEl" class="item">
        <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="12" cy="12" r="8.5" />
          <path d="M8.5 12.2l2.4 2.4 4.6-4.8" />
        </svg>
        <span>今日已办 {{ done }}</span>
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

@media (max-width: 520px) {
  .statsbar {
    font-size: 12px;
  }
  .divider {
    display: none;
  }
}
</style>