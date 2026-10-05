import { ref, type Ref } from "vue";
import { todayKey } from "../utils/date";

const today = ref(todayKey());
let started = false;

function sync() {
  const next = todayKey();
  if (next !== today.value) today.value = next;
}

function start() {
  if (started) return;
  started = true;
  window.setInterval(sync, 30 * 60_000);
  document.addEventListener("visibilitychange", sync);
  window.addEventListener("focus", sync);
}

export function useToday(): Ref<string> {
  start();
  sync();
  return today;
}
