<script setup lang="ts">
import { Pause, Play, RotateCcw } from 'lucide-vue-next';
import { computed, onUnmounted, ref } from 'vue';
import BaseButton from '@/components/common/BaseButton.vue';

const FOCUS = 25 * 60;
const SHORT_BREAK = 5 * 60;
const LONG_BREAK = 15 * 60;

const mode = ref<'focus' | 'short' | 'long'>('focus');
const seconds = ref(FOCUS);
const isRunning = ref(false);
let interval: ReturnType<typeof setInterval> | null = null;

const display = computed(() => {
  const minutes = Math.floor(seconds.value / 60)
    .toString()
    .padStart(2, '0');
  const remainingSeconds = (seconds.value % 60).toString().padStart(2, '0');
  return `${minutes}:${remainingSeconds}`;
});

const progress = computed(() => {
  const total = mode.value === 'focus' ? FOCUS : mode.value === 'short' ? SHORT_BREAK : LONG_BREAK;
  return ((total - seconds.value) / total) * 100;
});

function setMode(nextMode: typeof mode.value) {
  stop();
  mode.value = nextMode;
  seconds.value = nextMode === 'focus' ? FOCUS : nextMode === 'short' ? SHORT_BREAK : LONG_BREAK;
}

function start() {
  if (isRunning.value) return;
  isRunning.value = true;
  interval = setInterval(() => {
    if (seconds.value > 0) {
      seconds.value -= 1;
      return;
    }
    stop();
  }, 1000);
}

function stop() {
  isRunning.value = false;
  if (interval) {
    clearInterval(interval);
    interval = null;
  }
}

function reset() {
  stop();
  setMode(mode.value);
}

function toggle() {
  isRunning.value ? stop() : start();
}

onUnmounted(() => {
  stop();
});
</script>

<template>
  <div class="pomodoro">
    <div class="pomodoro__modes" role="tablist">
      <button
        v-for="m in ['focus', 'short', 'long'] as const"
        :key="m"
        type="button"
        role="tab"
        class="pomodoro__mode"
        :class="{ 'pomodoro__mode--active': mode === m }"
        @click="setMode(m)"
      >
        {{ m === 'focus' ? 'Focus' : m === 'short' ? 'Short' : 'Long' }}
      </button>
    </div>

    <div class="pomodoro__clock">
      <svg class="pomodoro__ring" viewBox="0 0 120 120" aria-hidden="true">
        <circle cx="60" cy="60" r="54" fill="none" stroke="var(--color-hairline)" stroke-width="4" />
        <circle
          cx="60"
          cy="60"
          r="54"
          fill="none"
          stroke="var(--color-primary)"
          stroke-width="4"
          stroke-linecap="round"
          :stroke-dasharray="`${2 * Math.PI * 54}`"
          :stroke-dashoffset="`${2 * Math.PI * 54 * (1 - progress / 100)}`"
          transform="rotate(-90 60 60)"
          class="pomodoro__progress"
        />
      </svg>
      <span class="pomodoro__time numeric">{{ display }}</span>
    </div>

    <div class="pomodoro__controls">
      <button class="pomodoro__icon" type="button" @click="reset" title="Reset">
        <RotateCcw :size="16" />
      </button>
      <BaseButton @click="toggle">
        <Play v-if="!isRunning" :size="14" />
        <Pause v-else :size="14" />
        {{ isRunning ? 'Pause' : 'Start' }}
      </BaseButton>
    </div>
  </div>
</template>

<style scoped>
.pomodoro {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-lg);
  width: 100%;
}

.pomodoro__modes {
  display: inline-flex;
  background: var(--color-canvas);
  border: 1px solid var(--color-hairline);
  border-radius: var(--radius-pill);
  padding: 3px;
  gap: 2px;
}

.pomodoro__mode {
  padding: 6px var(--space-md);
  border-radius: var(--radius-pill);
  font-size: var(--text-xs);
  font-weight: var(--weight-medium);
  color: var(--color-muted);
  transition: all var(--transition-fast);
}

.pomodoro__mode:hover {
  color: var(--color-on-dark);
}

.pomodoro__mode--active {
  background: var(--color-surface-elevated);
  color: var(--color-on-dark);
}

.pomodoro__clock {
  position: relative;
  width: 200px;
  height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.pomodoro__ring {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
}

.pomodoro__progress {
  transition: stroke-dashoffset 1s linear;
}

.pomodoro__time {
  font-size: 44px;
  font-weight: var(--weight-bold);
  color: var(--color-on-dark);
  letter-spacing: -0.02em;
}

.pomodoro__controls {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.pomodoro__icon {
  width: 36px;
  height: 36px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-full);
  color: var(--color-muted);
  transition: all var(--transition-fast);
}

.pomodoro__icon:hover {
  color: var(--color-on-dark);
  background: var(--color-surface-elevated);
}
</style>
