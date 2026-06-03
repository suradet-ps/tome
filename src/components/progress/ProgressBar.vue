<script setup lang="ts">
import { computed } from 'vue'
import type { ReadingStatus } from '@/types'

interface Props {
  completed: number
  total: number
  status?: ReadingStatus
  showLabel?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  showLabel: false,
})

const percentage = computed(() => (props.total === 0 ? 0 : Math.round((props.completed / props.total) * 100)))
</script>

<template>
  <div class="progress">
    <div class="progress__bar">
      <div class="progress__fill" :style="{ width: `${percentage}%` }" :data-status="props.status"></div>
    </div>
    <span v-if="props.showLabel" class="progress__label numeric">{{ percentage }}%</span>
  </div>
</template>

<style scoped>
.progress {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  width: 100%;
}

.progress__bar {
  flex: 1;
  height: 6px;
  background: var(--color-canvas);
  border-radius: var(--radius-pill);
  overflow: hidden;
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
  text-align: right;
}
</style>
