<script setup lang="ts">
import { computed } from 'vue'
import type { ReadingStatus } from '@/types'

interface Props {
  completed: number
  total: number
  status?: ReadingStatus
}

const props = defineProps<Props>()

const percentage = computed(() => (props.total === 0 ? 0 : Math.round((props.completed / props.total) * 100)))
</script>

<template>
  <div class="progress">
    <div class="progress__bar">
      <div class="progress__fill" :style="{ width: `${percentage}%` }" :data-status="props.status"></div>
    </div>
    <span class="progress__label">{{ percentage }}%</span>
  </div>
</template>

<style scoped>
.progress {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.progress__bar {
  flex: 1;
  height: 8px;
  background: rgba(43, 49, 57, 0.88);
  border-radius: var(--radius-pill);
  overflow: hidden;
  border: 1px solid rgba(43, 49, 57, 0.8);
}

.progress__fill {
  height: 100%;
  background: var(--color-primary);
  border-radius: var(--radius-pill);
  transition: width var(--transition-slow);
}

.progress__fill[data-status='completed'] {
  background: var(--color-success);
}

.progress__fill[data-status='review_needed'] {
  background: var(--color-warning);
}

.progress__label {
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  color: var(--color-muted);
  min-width: 32px;
  font-family: var(--font-number);
  font-variant-numeric: tabular-nums;
}
</style>
