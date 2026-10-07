<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Todo } from "../types";
import { dateKey, formatDateLabel } from "../utils/date";
import { useToday } from "../composables/useToday";
import TopNav from "./TopNav.vue";
import EmptyState from "./EmptyState.vue";
import StatsBar from "./StatsBar.vue";
import TodoList from "./TodoList.vue";
import CalendarPopup from "./CalendarPopup.vue";

const todos = ref<Todo[]>([]);
const listComp = ref<InstanceType<typeof TodoList> | null>(null);
const statsComp = ref<InstanceType<typeof StatsBar> | null>(null);
let unlisten: UnlistenFn | null = null;

const reduceMotion = window.matchMedia(
  "(prefers-reduced-motion: reduce)"
).matches;

const today = useToday();
const filterMode = ref<"all" | "today" | "leftover">("all");
const pending = computed(() =>
  todos.value.filter((t) => {
    if (t.done || isDeferred(t)) return false;
    if (filterMode.value === "today") return isToday(t.created_at);
    if (filterMode.value === "leftover") return !isToday(t.created_at);
    return true;
  })
);
const hasPending = computed(() => pending.value.length > 0);

const selectedDate = ref<string | null>(null);
const calendarOpen = ref(false);
const showingHistory = computed(() => selectedDate.value !== null);

function isToday(ms?: number | null): boolean {
  return !!ms && dateKey(ms) === today.value;
}

function isDeferred(t: Todo): boolean {
  return !!t.defer_until && dateKey(t.defer_until) > today.value;
}

const dateLabel = computed(() =>
  formatDateLabel(selectedDate.value ?? today.value)
);

watch(today, (_next, prev) => {
  if (selectedDate.value === prev) selectedDate.value = null;
});

const todayPending = computed(
  () => todos.value.filter((t) => !t.done && !isDeferred(t) && isToday(t.created_at)).length
);
const leftoverPending = computed(
  () => todos.value.filter((t) => !t.done && !isDeferred(t) && !isToday(t.created_at)).length
);
const todayDone = computed(
  () => todos.value.filter((t) => isToday(t.completed_at)).length
);

const doneDays = computed(() => {
  const days = new Set<string>();
  todos.value.forEach((t) => {
    if (t.completed_at) days.add(dateKey(t.completed_at));
  });
  return days;
});

const selectedDone = computed(() => {
  if (!selectedDate.value) return [];
  return todos.value
    .filter(
      (t) => t.completed_at && dateKey(t.completed_at) === selectedDate.value
    )
    .sort((a, b) => (a.completed_at ?? 0) - (b.completed_at ?? 0));
});

const doneCount = computed(() =>
  showingHistory.value ? selectedDone.value.length : todayDone.value
);

function toggleCalendar() {
  calendarOpen.value = !calendarOpen.value;
}

function toggleTodayFilter() {
  filterMode.value = filterMode.value === "today" ? "all" : "today";
}

function toggleLeftoverFilter() {
  filterMode.value = filterMode.value === "leftover" ? "all" : "leftover";
}

function selectDate(key: string) {
  calendarOpen.value = false;
  filterMode.value = "all";
  selectedDate.value = key === today.value ? null : key;
}

function showTodayDone() {
  calendarOpen.value = false;
  filterMode.value = "all";
  selectedDate.value = today.value;
}

function backToToday() {
  selectedDate.value = null;
  calendarOpen.value = false;
  filterMode.value = "all";
}

function getListEl(): HTMLElement | null {
  return listComp.value?.listEl ?? null;
}

function snapshotRects(excludeId: number): Map<number, DOMRect> {
  const map = new Map<number, DOMRect>();
  getListEl()
    ?.querySelectorAll<HTMLElement>(".card")
    .forEach((el) => {
      const id = Number(el.dataset.id);
      if (id !== excludeId) map.set(id, el.getBoundingClientRect());
    });
  return map;
}

function flyToDone(card: HTMLElement) {
  const target = statsComp.value?.doneEl;
  if (!target) return;
  const from = card.getBoundingClientRect();
  const to = target.getBoundingClientRect();

  const p0x = from.left + from.width / 2;
  const p0y = from.top + from.height / 2;
  const p1x = to.left + to.width / 2;
  const p1y = to.top + to.height / 2;

  const cpx = p0x + (p1x - p0x) * 0.85 + 70;
  const cpy = p0y + (p1y - p0y) * 0.2;

  const clone = card.cloneNode(true) as HTMLElement;
  Object.assign(clone.style, {
    position: "fixed",
    left: `${from.left}px`,
    top: `${from.top}px`,
    width: `${from.width}px`,
    height: `${from.height}px`,
    margin: "0",
    zIndex: "60",
    pointerEvents: "none",
    transformOrigin: "center center",
    willChange: "transform, opacity, border-radius",
  });
  document.body.appendChild(clone);

  const STEPS = 32;
  const keyframes: Keyframe[] = [];
  for (let i = 0; i <= STEPS; i++) {
    const t = i / STEPS;
    const e = t * t;
    const inv = 1 - e;
    const x = inv * inv * p0x + 2 * inv * e * cpx + e * e * p1x;
    const y = inv * inv * p0y + 2 * inv * e * cpy + e * e * p1y;
    const scale = 1 - 0.97 * e;
    const radius = Math.min(50, e * 65);
    const opacity = i === STEPS ? 0 : Math.max(0, 1 - e * e * 0.85);
    keyframes.push({
      transform: `translate(${x - p0x}px, ${y - p0y}px) scale(${scale})`,
      borderRadius: `${radius}%`,
      opacity,
    });
  }

  const anim = clone.animate(keyframes, {
    duration: 560,
    easing: "linear",
    fill: "forwards",
  });
  anim.onfinish = () => {
    clone.remove();
    target.animate(
      [
        { transform: "scale(1)" },
        { transform: "scale(1.4)" },
        { transform: "scale(1)" },
      ],
      { duration: 240, easing: "ease-out" }
    );
  };
  window.setTimeout(() => clone.remove(), 900);
}

function playFlip(prev: Map<number, DOMRect>) {
  getListEl()
    ?.querySelectorAll<HTMLElement>(".card")
    .forEach((el) => {
      const before = prev.get(Number(el.dataset.id));
      if (!before) return;
      const after = el.getBoundingClientRect();
      const dy = before.top - after.top;
      if (Math.abs(dy) < 0.5) return;
      el.animate(
        [{ transform: `translateY(${dy}px)` }, { transform: "translateY(0)" }],
        { duration: 300, easing: "cubic-bezier(.4,0,.2,1)" }
      );
    });
}

async function load() {
  todos.value = await invoke<Todo[]>("get_todos");
}

async function remove(id: number) {
  await invoke("delete_todo", { id });
}

async function toggle(item: Todo) {
  if (item.done) {
    await invoke("toggle_todo", { id: item.id, done: false });
    return;
  }

  const card = getListEl()?.querySelector<HTMLElement>(
    `.card[data-id="${item.id}"]`
  );
  const prevRects = card ? snapshotRects(item.id) : new Map<number, DOMRect>();
  if (card && !reduceMotion) flyToDone(card);

  todos.value = todos.value.map((t) =>
    t.id === item.id ? { ...t, done: true, completed_at: Date.now() } : t
  );

  await nextTick();
  if (!reduceMotion) playFlip(prevRects);
  await invoke("toggle_todo", { id: item.id, done: true });
}

async function openSettings() {
  await invoke("open_settings");
}

async function openAi() {
  await invoke("open_ai");
}

function onKeyDown(event: KeyboardEvent) {
  if (!event.metaKey || event.shiftKey || event.ctrlKey || event.altKey) return;
  if (event.key === "w") {
    event.preventDefault();
    void invoke("hide_window");
  } else if (event.key === "m") {
    event.preventDefault();
    void invoke("minimize_window");
  }
}

onMounted(async () => {
  await load();
  window.addEventListener("keydown", onKeyDown);
  unlisten = await listen<Todo[]>("todos-changed", (event) => {
    todos.value = event.payload;
  });
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeyDown);
  unlisten?.();
});
</script>

<template>
  <div class="page" data-tauri-drag-region="deep">
    <TopNav @more="openSettings" @ai="openAi" />

    <main class="stage">
      <div class="stage-inner">
        <Transition name="swap">
          <div v-if="showingHistory" key="history" class="view">
            <div class="history-head">
              <button class="back" type="button" @click="backToToday">
                <svg viewBox="0 0 24 24" aria-hidden="true">
                  <path d="M14.5 6.5 9 12l5.5 5.5" />
                </svg>
                <span>返回今天待办</span>
              </button>
            </div>
            <TodoList v-if="selectedDone.length" :items="selectedDone" readonly />
            <p v-else class="history-empty">该日期没有已办</p>
          </div>

          <div v-else key="today" class="view">
            <TodoList
              v-if="hasPending"
              ref="listComp"
              :items="pending"
              @toggle="toggle"
              @remove="remove"
            />
            <EmptyState v-else-if="filterMode === 'all'" />
            <p v-else class="filter-empty">
              {{ filterMode === "leftover" ? "太棒了，没有遗留待办" : "太棒了，今天待办全部完成" }}
            </p>
          </div>
        </Transition>
      </div>
    </main>

    <StatsBar
      ref="statsComp"
      :date="dateLabel"
      :mode="showingHistory ? 'history' : 'today'"
      :leftover="leftoverPending"
      :pending="todayPending"
      :done="doneCount"
      :filter="filterMode"
      @date-click="toggleCalendar"
      @done-click="showTodayDone"
      @pending-click="toggleTodayFilter"
      @leftover-click="toggleLeftoverFilter"
    />

    <button
      v-if="calendarOpen"
      class="cal-backdrop"
      type="button"
      aria-label="关闭日历"
      @click="calendarOpen = false"
    ></button>

    <Transition name="cal">
      <CalendarPopup
        v-if="calendarOpen"
        :done-days="doneDays"
        :value="selectedDate"
        :today="today"
        @select="selectDate"
        @close="calendarOpen = false"
      />
    </Transition>
  </div>
</template>

<style scoped>
.page {
  position: relative;
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  border-radius: var(--page-radius);
  overflow: hidden;
  user-select: none;
  background-color: var(--bg-a);
  background-image: url("../assets/bg.png");
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
  color: var(--ink);
}

.stage {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.stage-inner {
  position: relative;
  flex: 1;
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.view {
  width: 100%;
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: clamp(12px, 3vh, 28px) var(--pad-x) clamp(20px, 6vh, 48px);
}

.cal-backdrop {
  position: absolute;
  inset: 0;
  z-index: 30;
  margin: 0;
  padding: 0;
  border: none;
  background: transparent;
  cursor: default;
}

.history-head {
  width: 100%;
  max-width: 560px;
  margin-bottom: 12px;
}

.back {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 14px 6px 10px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.55);
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
  font-size: 12.5px;
  cursor: pointer;
  transition: color 180ms ease, transform 180ms ease;
}

.back:hover {
  color: var(--gold);
  transform: translateX(-1px);
}

.back:active {
  transform: scale(0.97);
}

.back svg {
  width: 15px;
  height: 15px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.history-empty {
  margin: 0;
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--ink-3);
  font-size: clamp(14px, 2.6vw, 18px);
}

.filter-empty {
  margin: 40px 0 0 0;
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--ink-3);
  font-size: clamp(14px, 2.6vw, 18px);
}

.swap-enter-active,
.swap-leave-active {
  transition: opacity 200ms ease;
}

.swap-leave-active {
  position: absolute;
  inset: 0;
}

.swap-enter-from,
.swap-leave-to {
  opacity: 0;
}

.cal-enter-active,
.cal-leave-active {
  transition: opacity 160ms ease, transform 160ms ease;
  transform-origin: bottom left;
}

.cal-enter-from,
.cal-leave-to {
  opacity: 0;
  transform: translateY(6px) scale(0.97);
}

@media (prefers-reduced-motion: reduce) {
  .swap-enter-active,
  .swap-leave-active,
  .cal-enter-active,
  .cal-leave-active {
    transition: none;
  }
}
</style>