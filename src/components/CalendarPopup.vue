<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { keyOf, parseKey } from "../utils/date";

interface Cell {
  key: string;
  blank: boolean;
  day: number;
  disabled: boolean;
  isToday: boolean;
  selected: boolean;
  hasDone: boolean;
}

const props = defineProps<{
  doneDays: Set<string>;
  value: string | null;
  today: string;
}>();

const emit = defineEmits<{
  (e: "select", key: string): void;
  (e: "close"): void;
}>();

const WEEK = ["日", "一", "二", "三", "四", "五", "六"];

const base = parseKey(props.value ?? props.today);
const viewYear = ref(base.year);
const viewMonth = ref(base.month);

const monthTitle = computed(() => `${viewYear.value}年${viewMonth.value + 1}月`);

const canNext = computed(() => {
  const current = parseKey(props.today);
  return (
    viewYear.value < current.year ||
    (viewYear.value === current.year && viewMonth.value < current.month)
  );
});

const cells = computed<Cell[]>(() => {
  const firstDow = new Date(viewYear.value, viewMonth.value, 1).getDay();
  const daysInMonth = new Date(viewYear.value, viewMonth.value + 1, 0).getDate();
  const list: Cell[] = [];

  for (let i = 0; i < firstDow; i++) {
    list.push({
      key: `blank-${i}`,
      blank: true,
      day: 0,
      disabled: false,
      isToday: false,
      selected: false,
      hasDone: false,
    });
  }

  for (let day = 1; day <= daysInMonth; day++) {
    const key = keyOf(viewYear.value, viewMonth.value, day);
    list.push({
      blank: false,
      key,
      day,
      disabled: key > props.today,
      isToday: key === props.today,
      selected: key === props.value,
      hasDone: props.doneDays.has(key),
    });
  }

  return list;
});

function prevMonth() {
  if (viewMonth.value === 0) {
    viewMonth.value = 11;
    viewYear.value -= 1;
  } else {
    viewMonth.value -= 1;
  }
}

function nextMonth() {
  if (!canNext.value) return;
  if (viewMonth.value === 11) {
    viewMonth.value = 0;
    viewYear.value += 1;
  } else {
    viewMonth.value += 1;
  }
}

function goToday() {
  const current = parseKey(props.today);
  viewYear.value = current.year;
  viewMonth.value = current.month;
}

function pick(cell: Cell) {
  if (cell.blank || cell.disabled) return;
  emit("select", cell.key);
}

function onKeyDown(event: KeyboardEvent) {
  if (event.key === "Escape") emit("close");
}

onMounted(() => {
  document.addEventListener("keydown", onKeyDown);
});

onUnmounted(() => {
  document.removeEventListener("keydown", onKeyDown);
});
</script>

<template>
  <div class="calendar" @mousedown.stop>
    <div class="head">
      <button class="nav" type="button" aria-label="上个月" @click="prevMonth">
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M14.5 6.5 9 12l5.5 5.5" />
        </svg>
      </button>
      <button class="title" type="button" @click="goToday">{{ monthTitle }}</button>
      <button
        class="nav"
        type="button"
        aria-label="下个月"
        :disabled="!canNext"
        @click="nextMonth"
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M9.5 6.5 15 12l-5.5 5.5" />
        </svg>
      </button>
    </div>

    <div class="week">
      <span v-for="w in WEEK" :key="w">{{ w }}</span>
    </div>

    <div class="grid">
      <template v-for="cell in cells" :key="cell.key">
        <span v-if="cell.blank" class="cell blank"></span>
        <button
          v-else
          class="cell day"
          type="button"
          :class="{
            today: cell.isToday,
            selected: cell.selected,
            disabled: cell.disabled,
          }"
          :disabled="cell.disabled"
          @click="pick(cell)"
        >
          <span class="num">{{ cell.day }}</span>
          <span v-if="cell.hasDone" class="dot"></span>
        </button>
      </template>
    </div>
  </div>
</template>

<style scoped>
.calendar {
  position: absolute;
  left: calc(12px + clamp(16px, 3vw, 24px));
  bottom: calc(
    clamp(12px, 3vh, 24px) + clamp(52px, 9vh, 74px) + 12px
  );
  width: 264px;
  padding: 14px;
  border-radius: 18px;
  z-index: 40;
  background: linear-gradient(
    135deg,
    rgba(255, 255, 255, 0.55),
    rgba(255, 255, 255, 0.28)
  );
  -webkit-backdrop-filter: blur(18px) saturate(140%);
  backdrop-filter: blur(18px) saturate(140%);
  border: 1px solid var(--glass-border);
  box-shadow:
    0 22px 50px -24px rgba(201, 190, 178, 0.95),
    inset 0 1px 0 rgba(255, 255, 255, 0.65);
  color: var(--ink);
  user-select: none;
}

.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.title {
  flex: 1;
  border: none;
  background: transparent;
  font-family: inherit;
  font-size: 13px;
  font-weight: 600;
  color: var(--ink);
  cursor: pointer;
}

.title:hover {
  color: var(--gold);
}

.nav {
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 8px;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: var(--ink-4);
  transition: background 150ms ease, color 150ms ease;
}

.nav:hover:not(:disabled) {
  background: rgba(232, 196, 155, 0.28);
  color: var(--gold);
}

.nav:disabled {
  opacity: 0.3;
  cursor: default;
}

.nav svg {
  width: 15px;
  height: 15px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.week,
.grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 2px;
}

.week {
  margin-bottom: 4px;
}

.week span {
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  color: var(--ink-3);
}

.cell {
  height: 32px;
  border: none;
  background: transparent;
  border-radius: 9px;
  font-family: inherit;
  font-size: 12.5px;
  color: var(--ink);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  padding: 0;
}

.cell.day {
  cursor: pointer;
  transition: background 150ms ease, color 150ms ease;
}

.cell.day:hover:not(.disabled) {
  background: rgba(232, 196, 155, 0.3);
}

.cell.today {
  box-shadow: inset 0 0 0 1.4px var(--gold);
}

.cell.selected {
  background: var(--gold);
  color: #6b5433;
  font-weight: 600;
}

.cell.disabled {
  color: var(--ink-3);
  opacity: 0.4;
  cursor: default;
}

.dot {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--gold);
}

.cell.selected .dot {
  background: #6b5433;
}
</style>
