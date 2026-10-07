<script setup lang="ts">
import { ref } from "vue";
import type { Todo } from "../types";

withDefaults(defineProps<{ items: Todo[]; readonly?: boolean; revertable?: boolean }>(), {
  readonly: false,
  revertable: false,
});
defineEmits<{
  (e: "toggle", item: Todo): void;
  (e: "remove", id: number): void;
  (e: "revert", item: Todo): void;
  (e: "pin", item: Todo): void;
}>();

const listEl = ref<HTMLElement | null>(null);
defineExpose({ listEl });
</script>

<template>
  <ul ref="listEl" class="list">
    <li
      v-for="todo in items"
      :key="todo.id"
      :data-id="todo.id"
      class="card"
      :class="{ readonly }"
    >
      <input
        class="check"
        type="checkbox"
        :checked="todo.done"
        :disabled="readonly || todo.pinned"
        aria-label="标记完成"
        @change="$emit('toggle', todo)"
      />
      <span class="text">{{ todo.text }}</span>
      <button
        v-if="!readonly"
        class="pin"
        :class="{ pinned: todo.pinned }"
        type="button"
        :aria-label="todo.pinned ? '取消固定' : '固定'"
        @click="$emit('pin', todo)"
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M12 17v5" />
          <path
            d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z"
          />
        </svg>
      </button>
      <button
        v-if="!readonly"
        class="remove"
        type="button"
        aria-label="删除"
        @click="$emit('remove', todo.id)"
      >
        ×
      </button>
      <button
        v-if="revertable"
        class="revert"
        type="button"
        aria-label="回退"
        @click="$emit('revert', todo)"
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <polyline points="1 4 1 10 7 10" />
          <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
        </svg>
      </button>
    </li>
  </ul>
</template>

<style scoped>
.list {
  list-style: none;
  margin: 0;
  padding: 0;
  width: 100%;
  max-width: 560px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.card {
  display: flex;
  align-items: center;
  gap: clamp(10px, 1.4vw, 16px);
  padding: clamp(10px, 1.6vh, 16px) clamp(12px, 1.6vw, 20px);
  border-radius: 14px;
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

.check {
  flex-shrink: 0;
  appearance: none;
  -webkit-appearance: none;
  width: 16px;
  height: 16px;
  margin: 0;
  border-radius: 50%;
  border: 1.4px solid #c9beb2;
  background: transparent;
  cursor: pointer;
  transition: border-color 180ms ease, background 180ms ease;
}

.check:hover {
  border-color: var(--gold);
}

.check:checked {
  background: var(--gold);
  border-color: var(--gold);
}

.card.readonly .text {
  color: var(--ink-3);
  text-decoration: line-through;
}

.card.readonly .check {
  cursor: default;
}

.text {
  flex: 1;
  font-size: clamp(13px, 1.5vw, 18px);
  line-height: 1.4;
  color: var(--ink);
  word-break: break-word;
  -webkit-user-select: text;
  user-select: text;
}

.check:disabled {
  cursor: default;
  opacity: 0.45;
}

.check:disabled:hover {
  border-color: #c9beb2;
}

.pin {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: var(--ink-3);
  padding: 0 4px;
  cursor: pointer;
  opacity: 0;
  transition: opacity 180ms ease, color 180ms ease;
}

.pin svg {
  width: 16px;
  height: 16px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.card:hover .pin {
  opacity: 0.7;
}

.pin:hover {
  color: var(--gold);
  opacity: 1;
}

.pin.pinned {
  color: var(--gold);
  opacity: 0.9;
}

.pin.pinned svg {
  fill: currentColor;
  fill-opacity: 0.18;
}

.remove {
  flex-shrink: 0;
  border: none;
  background: transparent;
  color: var(--ink-3);
  font-size: 18px;
  line-height: 1;
  padding: 0 4px;
  cursor: pointer;
  opacity: 0;
  transition: opacity 180ms ease, color 180ms ease;
}

.card:hover .remove {
  opacity: 0.7;
}

.remove:hover {
  color: var(--ink-4);
  opacity: 1;
}

.revert {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: var(--ink-3);
  padding: 0 4px;
  cursor: pointer;
  opacity: 0;
  transition: opacity 180ms ease, color 180ms ease;
}

.revert svg {
  width: 16px;
  height: 16px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.card:hover .revert {
  opacity: 0.7;
}

.revert:hover {
  color: var(--gold);
  opacity: 1;
}

@media (prefers-reduced-motion: reduce) {
  .check,
  .remove,
  .revert,
  .pin {
    transition: none;
  }
}
</style>