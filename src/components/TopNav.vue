<script setup lang="ts">
withDefaults(defineProps<{ hideActions?: boolean }>(), { hideActions: false });
defineEmits<{ (e: "more"): void; (e: "ai"): void }>();
</script>

<template>
  <header class="topnav">
    <div class="brand">
      <span class="brand-text">朝暮</span>
      <span class="brand-line"></span>
    </div>

    <div v-if="!hideActions" class="actions">
      <button class="more" type="button" aria-label="AI 日报" @click="$emit('ai')">
        <svg class="more-icon" viewBox="0 0 24 24" aria-hidden="true">
          <path
            class="spark"
            d="M12 3.4l1.7 4.6 4.6 1.7-4.6 1.7L12 16l-1.7-4.6L5.7 9.7l4.6-1.7L12 3.4z"
          />
          <path class="spark small" d="M18.4 14.2l.8 2.1 2.1.8-2.1.8-.8 2.1-.8-2.1-2.1-.8 2.1-.8.8-2.1z" />
        </svg>
      </button>

      <button class="more" type="button" aria-label="更多" @click="$emit('more')">
        <svg class="more-icon" viewBox="0 0 24 24" aria-hidden="true">
          <circle class="dot" cx="6" cy="12" r="1.9" />
          <circle class="dot" cx="12" cy="12" r="1.9" />
          <circle class="dot" cx="18" cy="12" r="1.9" />
        </svg>
      </button>
    </div>
  </header>
</template>

<style scoped>
.topnav {
  flex-shrink: 0;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: clamp(20px, 2vh, 30px) var(--pad-x) 0 max(var(--pad-x), 84px);
}

.brand {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-left: 20px;
}

.brand-text {
  font-size: clamp(18px, 2.6vw, 26px);
  font-weight: 400;
  letter-spacing: 3px;
  color: #454545;
  line-height: 1;
}

.brand-line {
  width: 40px;
  height: 3px;
  border-radius: 999px;
  background: var(--gold);
}

.actions {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.more {
  flex-shrink: 0;
  width: clamp(27px, 4.2vw, 31px);
  height: clamp(27px, 4.2vw, 31px);
  padding: 0;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.6);
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
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--ink-4);
  cursor: pointer;
  transition:
    background 180ms ease,
    box-shadow 180ms ease,
    transform 180ms ease,
    color 180ms ease;
}

.more:hover {
  background: linear-gradient(
    135deg,
    rgba(255, 255, 255, 0.55),
    rgba(255, 255, 255, 0.24)
  );
  box-shadow:
    0 16px 30px -20px rgba(201, 190, 178, 1),
    inset 0 1px 0 rgba(255, 255, 255, 0.7);
  transform: translateY(-1px);
  color: var(--gold);
}

.more:active {
  transform: scale(0.96);
}

.more:focus-visible {
  outline: 2px solid var(--gold);
  outline-offset: 2px;
}

.more-icon {
  width: clamp(15px, 2.4vw, 18px);
  height: clamp(15px, 2.4vw, 18px);
  display: block;
}

.dot {
  fill: currentColor;
  transform-box: fill-box;
  transform-origin: center;
  transition:
    transform 180ms ease,
    fill 180ms ease;
}

.spark {
  fill: currentColor;
  transform-box: fill-box;
  transform-origin: center;
  transition: transform 180ms ease;
}

.spark.small {
  opacity: 0.75;
}

.more:hover .spark {
  transform: scale(1.15) rotate(8deg);
}

.more:hover .dot {
  transform: scale(1.25);
}

.more:hover .dot:nth-child(2) {
  transition-delay: 60ms;
}

.more:hover .dot:nth-child(3) {
  transition-delay: 120ms;
}

@media (prefers-reduced-motion: reduce) {
  .more,
  .dot {
    transition: none;
  }
  .more:hover {
    transform: none;
  }
  .more:hover .dot {
    transform: none;
  }
}
</style>