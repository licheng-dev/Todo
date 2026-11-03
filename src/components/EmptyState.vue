<script setup lang="ts">
// import TodoIllustration from "./TodoIllustration.vue";
</script>

<template>
  <div class="empty">
<!--    <div class="illus">-->
<!--      <TodoIllustration />-->
<!--    </div>-->
    <h2 class="title">还没有待办事项</h2>
    <p class="hint">按 Cmd + Shift + N 添加</p>
  </div>
</template>

<style scoped>
.empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  /* 图标已在背景图垂直居中，文字排到图标下方 */
  justify-content: flex-end;
  padding-bottom: clamp(8px, 3vh, 24px);
  text-align: center;
}

.illus {
  width: clamp(112px, 20vw, 180px);
  animation: float 5s ease-in-out infinite;
}

.title {
  margin: 0;
  font-size: clamp(19px, 3.2vw, 26px);
  font-weight: 500;
  color: var(--ink-2);
}

.hint {
  margin: 14px 0 0;
  font-size: clamp(14px, 2.6vw, 20px);
  font-weight: 400;
  color: var(--ink-3);
}

@keyframes float {
  0%,
  100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-3px);
  }
}

@media (prefers-reduced-motion: reduce) {
  .illus {
    animation: none;
  }
}
</style>