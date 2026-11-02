<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import MainPanel from "./components/MainPanel.vue";
import AddTodoDialog from "./components/AddTodoDialog.vue";

const label = getCurrentWindow().label;
</script>

<template>
  <MainPanel v-if="label === 'main'" />
  <AddTodoDialog v-else-if="label === 'add-todo'" />
  <div v-else class="fallback">
    <p>未知窗口: {{ label }}</p>
  </div>
</template>

<style>
.fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100vh;
  background: var(--bg-a);
  color: var(--ink);
}
</style>