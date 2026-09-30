<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Todo } from "../types";
import TopNav from "./TopNav.vue";
import EmptyState from "./EmptyState.vue";
import StatsBar from "./StatsBar.vue";
import TodoList from "./TodoList.vue";

const todos = ref<Todo[]>([]);
const listComp = ref<InstanceType<typeof TodoList> | null>(null);
const statsComp = ref<InstanceType<typeof StatsBar> | null>(null);
let unlisten: UnlistenFn | null = null;

const reduceMotion = window.matchMedia(
  "(prefers-reduced-motion: reduce)"
).matches;

const pending = computed(() => todos.value.filter((t) => !t.done));
const hasPending = computed(() => pending.value.length > 0);

const dates = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
const now = new Date();
const dateLabel = `${now.getMonth() + 1}月${now.getDate()}日 ${dates[now.getDay()]}`;

function isToday(ms?: number | null): boolean {
  if (!ms) return false;
  const d = new Date(ms);
  return (
    d.getFullYear() === now.getFullYear() &&
    d.getMonth() === now.getMonth() &&
    d.getDate() === now.getDate()
  );
}

const todayPending = computed(
  () => todos.value.filter((t) => !t.done && isToday(t.created_at)).length
);
const todayDone = computed(
  () => todos.value.filter((t) => isToday(t.completed_at)).length
);

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

onMounted(async () => {
  await load();
  unlisten = await listen<Todo[]>("todos-changed", (event) => {
    todos.value = event.payload;
  });
});

onUnmounted(() => {
  unlisten?.();
});
</script>

<template>
  <div class="page" data-tauri-drag-region="deep">
    <TopNav @more="openSettings" />

    <main class="stage">
      <div class="stage-inner">
        <Transition name="swap" mode="out-in">
          <TodoList
            v-if="hasPending"
            key="list"
            ref="listComp"
            :items="pending"
            @toggle="toggle"
            @remove="remove"
          />
          <EmptyState v-else key="empty" />
        </Transition>
      </div>
    </main>

    <StatsBar
      ref="statsComp"
      :date="dateLabel"
      :pending="todayPending"
      :done="todayDone"
    />
  </div>
</template>

<style scoped>
.page {
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
  flex: 1;
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: clamp(12px, 3vh, 28px) var(--pad-x) clamp(20px, 6vh, 48px);
}

.swap-enter-active,
.swap-leave-active {
  transition: opacity 200ms ease;
}

.swap-enter-from,
.swap-leave-to {
  opacity: 0;
}

@media (prefers-reduced-motion: reduce) {
  .swap-enter-active,
  .swap-leave-active {
    transition: none;
  }
}
</style>