<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { onMounted, ref } from "vue";

const alwaysOnTop = ref(false);
const busy = ref(false);

async function sync() {
  if (busy.value) return;
  busy.value = true;
  try {
    await invoke("set_always_on_top", { enabled: alwaysOnTop.value });
  } finally {
    busy.value = false;
  }
}

async function toggle() {
  alwaysOnTop.value = !alwaysOnTop.value;
  await sync();
}

async function close() {
  await getCurrentWindow().hide();
}

onMounted(async () => {
  alwaysOnTop.value = await invoke<boolean>("get_always_on_top");
});
</script>

<template>
  <div class="settings" data-tauri-drag-region="deep">
    <div class="glass">
      <h2>设置</h2>

      <div class="row">
        <span class="label">窗口置顶</span>
        <button
          class="toggle"
          :class="{ on: alwaysOnTop }"
          @click="toggle"
          :disabled="busy"
        >
          <span class="knob"></span>
        </button>
      </div>

      <div class="shortcuts">
        <p class="hint">快捷键</p>
        <kbd>Cmd+Shift+N</kbd> 添加待办
        <kbd>Cmd+,</kbd> 设置
      </div>

      <button class="close-btn" @click="close">关闭</button>
    </div>
  </div>
</template>

<style scoped>
.settings {
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

h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  text-align: center;
}

.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.label {
  font-size: 14px;
}

.toggle {
  width: 44px;
  height: 24px;
  border-radius: 12px;
  border: none;
  cursor: pointer;
  background: rgba(128, 128, 128, 0.3);
  position: relative;
  transition: background 0.2s;
}

.toggle.on {
  background: #34c759;
}

.knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: #fff;
  transition: transform 0.2s;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.toggle.on .knob {
  transform: translateX(20px);
}

.shortcuts {
  font-size: 12px;
  opacity: 0.6;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
}

.hint {
  width: 100%;
  margin: 0 0 2px;
  font-weight: 600;
}

kbd {
  background: rgba(128, 128, 128, 0.15);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: inherit;
}

.close-btn {
  align-self: center;
  padding: 7px 24px;
  border-radius: 8px;
  border: none;
  background: rgba(128, 128, 128, 0.15);
  color: inherit;
  font-size: 13px;
  cursor: pointer;
  margin-top: 4px;
}
</style>