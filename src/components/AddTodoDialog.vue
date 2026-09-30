<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";

const draft = ref("");
const busy = ref(false);

async function submit() {
  const text = draft.value.trim();
  if (!text || busy.value) return;
  busy.value = true;
  try {
    await invoke("add_todo", { text });
    draft.value = "";
    await getCurrentWindow().hide();
  } finally {
    busy.value = false;
  }
}

async function cancel() {
  await getCurrentWindow().hide();
}
</script>

<template>
  <div class="dialog" data-tauri-drag-region="deep">
    <div class="glass">
      <h2>添加待办</h2>
      <form @submit.prevent="submit">
        <textarea
          v-model="draft"
          placeholder="输入待办内容…"
          rows="4"
          autofocus
          :disabled="busy"
          @keydown.meta.enter.prevent="submit"
          @keydown.ctrl.enter.prevent="submit"
          @keydown.esc.prevent="cancel"
        ></textarea>
        <div class="actions">
          <button type="button" class="btn-cancel" @click="cancel">取消</button>
          <button type="submit" class="btn-submit" :disabled="busy || !draft.trim()">
            确认
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.dialog {
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
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 20px 24px;
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

form {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

textarea {
  padding: 10px 14px;
  border-radius: 10px;
  border: 1px solid rgba(128, 128, 128, 0.3);
  background: rgba(128, 128, 128, 0.08);
  color: inherit;
  font-family: inherit;
  font-size: 14px;
  line-height: 1.5;
  resize: none;
  outline: none;
}

textarea::-webkit-scrollbar {
  width: 4px;
}

textarea::-webkit-scrollbar-track {
  background: transparent;
}

textarea::-webkit-scrollbar-thumb {
  background: rgba(128, 128, 128, 0.35);
  border-radius: 2px;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

button {
  padding: 8px 18px;
  border-radius: 8px;
  font-size: 13px;
  cursor: pointer;
  border: none;
}

.btn-cancel {
  background: rgba(128, 128, 128, 0.15);
  color: inherit;
}

.btn-submit {
  background: linear-gradient(135deg, #edcfa6, var(--gold));
  color: #6b5433;
  font-weight: 500;
  box-shadow: 0 10px 20px -14px rgba(201, 160, 110, 0.9);
  transition: filter 180ms ease, transform 180ms ease;
}

.btn-submit:hover:not(:disabled) {
  filter: brightness(1.03);
}

.btn-submit:active:not(:disabled) {
  transform: scale(0.97);
}

.btn-submit:disabled {
  opacity: 0.45;
  cursor: default;
}

@media (prefers-color-scheme: dark) {
  textarea {
    background: rgba(128, 128, 128, 0.12);
  }
  .btn-cancel {
    background: rgba(128, 128, 128, 0.2);
  }
}
</style>