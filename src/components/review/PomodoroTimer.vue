<script setup lang="ts">
import { Pause, Play, RotateCcw } from 'lucide-vue-next'
import { computed, onUnmounted, ref } from 'vue'
import BaseButton from '@/components/common/BaseButton.vue'

const FOCUS = 25 * 60
const SHORT_BREAK = 5 * 60
const LONG_BREAK = 15 * 60

const mode = ref<'focus' | 'short' | 'long'>('focus')
const seconds = ref(FOCUS)
const isRunning = ref(false)
let interval: ReturnType<typeof setInterval> | null = null

const display = computed(() => {
  const minutes = Math.floor(seconds.value / 60)
    .toString()
    .padStart(2, '0')
  const remainingSeconds = (seconds.value % 60).toString().padStart(2, '0')

  return `${minutes}:${remainingSeconds}`
})

const progress = computed(() => {
  const total = mode.value === 'focus' ? FOCUS : mode.value === 'short' ? SHORT_BREAK : LONG_BREAK
  return ((total - seconds.value) / total) * 100
})

function setMode(nextMode: typeof mode.value) {
  stop()
  mode.value = nextMode
  seconds.value = nextMode === 'focus' ? FOCUS : nextMode === 'short' ? SHORT_BREAK : LONG_BREAK
}

function start() {
  if (isRunning.value) return

  isRunning.value = true
  interval = setInterval(() => {
    if (seconds.value > 0) {
      seconds.value -= 1
      return
    }

    stop()
  }, 1000)
}

function stop() {
  isRunning.value = false
  if (interval) {
    clearInterval(interval)
    interval = null
  }
}

function reset() {
  stop()
  setMode(mode.value)
}

function toggle() {
  isRunning.value ? stop() : start()
}

onUnmounted(() => {
  stop()
})
</script>

<template>
  <div class="pomodoro">
    <div class="pomodoro__intro">
      <p class="pomodoro__eyebrow">Focus system</p>
      <h2 class="pomodoro__title">Pomodoro timer</h2>
      <p class="pomodoro__description">Timebox note-taking, chapter reviews, and flashcard drills in deliberate bursts.</p>
    </div>

    <div class="pomodoro__modes">
      <button
        v-for="m in ['focus', 'short', 'long'] as const"
        :key="m"
        type="button"
        class="pomodoro__mode-btn"
        :class="{ 'pomodoro__mode-btn--active': mode === m }"
        @click="setMode(m)"
      >
        {{ m === 'focus' ? 'Focus' : m === 'short' ? 'Short Break' : 'Long Break' }}
      </button>
    </div>

    <div class="pomodoro__clock">
      <svg class="pomodoro__ring" viewBox="0 0 120 120">
        <circle cx="60" cy="60" r="54" fill="none" stroke="var(--color-hairline)" stroke-width="6" />
        <circle
          cx="60"
          cy="60"
          r="54"
          fill="none"
          stroke="var(--color-primary)"
          stroke-width="6"
          stroke-linecap="round"
          :stroke-dasharray="`${2 * Math.PI * 54}`"
          :stroke-dashoffset="`${2 * Math.PI * 54 * (1 - progress / 100)}`"
          transform="rotate(-90 60 60)"
          class="pomodoro__progress"
        />
      </svg>
      <span class="pomodoro__time">{{ display }}</span>
    </div>

    <div class="pomodoro__controls">
      <button class="pomodoro__reset" type="button" @click="reset" title="Reset">
        <RotateCcw :size="18" />
      </button>
      <BaseButton size="lg" @click="toggle">
        <Play v-if="!isRunning" :size="18" />
        <Pause v-else :size="18" />
        {{ isRunning ? 'Pause' : 'Start' }}
      </BaseButton>
    </div>

    <p class="pomodoro__label">
      {{ mode === 'focus' ? '🎯 Focus Session' : mode === 'short' ? '☕ Short Break' : '🧘 Long Break' }}
    </p>
  </div>
</template>

<style scoped>
.pomodoro {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-lg);
  padding: var(--space-xl);
}

.pomodoro__intro {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-xs);
  text-align: center;
}

.pomodoro__eyebrow {
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--color-muted);
}

.pomodoro__title {
  font-size: var(--text-2xl);
  font-weight: var(--weight-bold);
  color: var(--color-on-dark);
}

.pomodoro__description {
  max-width: 420px;
  color: var(--color-muted);
  font-size: var(--text-sm);
}

.pomodoro__modes {
  display: flex;
  background: rgba(11, 14, 17, 0.72);
  border: 1px solid var(--color-hairline);
  border-radius: var(--radius-pill);
  padding: 4px;
  gap: 4px;
  flex-wrap: wrap;
  justify-content: center;
}

.pomodoro__mode-btn {
  padding: 6px var(--space-md);
  border-radius: var(--radius-pill);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  color: var(--color-muted);
  transition: all var(--transition-fast);
}

.pomodoro__mode-btn--active {
  background: var(--color-surface-card);
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
  font-size: 42px;
  font-weight: var(--weight-bold);
  font-family: var(--font-number);
  color: var(--color-on-dark);
  letter-spacing: -1px;
  font-variant-numeric: tabular-nums;
}

.pomodoro__controls {
  display: flex;
  align-items: center;
  gap: var(--space-md);
}

.pomodoro__reset {
  color: var(--color-muted);
  padding: var(--space-sm);
  border-radius: var(--radius-full);
  transition: all var(--transition-fast);
}

.pomodoro__reset:hover {
  color: var(--color-on-dark);
  background: var(--color-surface-elevated);
}

.pomodoro__label {
  font-size: var(--text-sm);
  color: var(--color-muted-strong);
}
</style>
