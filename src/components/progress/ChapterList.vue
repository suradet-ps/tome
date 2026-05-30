<script setup lang="ts">
import { AlertCircle, CheckCircle2, ChevronDown, ChevronRight, Circle, Clock } from 'lucide-vue-next'
import { ref, watch } from 'vue'
import { useProgressStore } from '@/stores/progress'
import type { Chapter, ReadingStatus } from '@/types'

interface Props {
  chapters: Chapter[]
  depth?: number
  selectedId?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  depth: 0,
  selectedId: null,
})

const emit = defineEmits<{ select: [chapter: Chapter] }>()
const progressStore = useProgressStore()
const expanded = ref<Record<string, boolean>>({})

watch(
  () => props.chapters,
  (chapters) => {
    if (props.depth > 0) return
    chapters.forEach((chapter) => {
      if (chapter.children?.length && expanded.value[chapter.id] === undefined) {
        expanded.value[chapter.id] = true
      }
    })
  },
  { immediate: true, deep: true },
)

function toggleExpand(id: string) {
  expanded.value[id] = !expanded.value[id]
}

function getStatusIcon(status?: ReadingStatus) {
  switch (status) {
    case 'completed':
      return CheckCircle2
    case 'in_progress':
      return Clock
    case 'review_needed':
      return AlertCircle
    default:
      return Circle
  }
}

function getStatusColor(status?: ReadingStatus) {
  switch (status) {
    case 'completed':
      return 'var(--color-success)'
    case 'in_progress':
      return 'var(--color-info)'
    case 'review_needed':
      return 'var(--color-warning)'
    default:
      return 'var(--color-muted)'
  }
}
</script>

<template>
  <ul class="chapter-list" :class="{ 'chapter-list--nested': props.depth > 0 }">
    <li v-for="chapter in props.chapters" :key="chapter.id" class="chapter-item">
      <div
        class="chapter-row"
        :class="{ 'chapter-row--active': props.selectedId === chapter.id }"
        :style="{ paddingLeft: `${props.depth * 16 + 12}px` }"
        @click="emit('select', chapter)"
      >
        <button v-if="chapter.children?.length" type="button" class="chapter-expand" @click.stop="toggleExpand(chapter.id)">
          <ChevronDown v-if="expanded[chapter.id]" :size="14" />
          <ChevronRight v-else :size="14" />
        </button>
        <span v-else class="chapter-expand chapter-expand--spacer"></span>

        <component
          :is="getStatusIcon(progressStore.getProgress(chapter.id)?.status)"
          :size="16"
          :style="{ color: getStatusColor(progressStore.getProgress(chapter.id)?.status) }"
          class="chapter-status-icon"
        />

        <span class="chapter-sequence">{{ chapter.sequence_number }}</span>
        <span class="chapter-title">{{ chapter.title }}</span>
      </div>

      <ChapterList
        v-if="chapter.children?.length && expanded[chapter.id]"
        :chapters="chapter.children"
        :depth="props.depth + 1"
        :selected-id="props.selectedId"
        @select="emit('select', $event)"
      />
    </li>
  </ul>
</template>

<style scoped>
.chapter-list {
  display: flex;
  flex-direction: column;
}

.chapter-list--nested {
  border-left: 1px solid rgba(43, 49, 57, 0.88);
  margin-left: 20px;
  padding-top: var(--space-xs);
}

.chapter-row {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  min-height: 44px;
  padding: var(--space-xs) var(--space-md);
  border-radius: var(--radius-md);
  cursor: pointer;
  border: 1px solid transparent;
  transition:
    background var(--transition-fast),
    border-color var(--transition-fast),
    transform var(--transition-fast);
}

.chapter-row:hover {
  background: rgba(43, 49, 57, 0.72);
  border-color: rgba(43, 49, 57, 0.95);
}

.chapter-row--active {
  background: rgba(252, 213, 53, 0.08);
  border-color: rgba(252, 213, 53, 0.2);
}

.chapter-row--active .chapter-title {
  color: var(--color-on-dark);
}

.chapter-expand {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  color: var(--color-muted);
  border-radius: var(--radius-sm);
  transition: background var(--transition-fast);
}

.chapter-expand:hover {
  background: var(--color-surface-card);
}

.chapter-expand--spacer {
  cursor: default;
}

.chapter-status-icon {
  flex-shrink: 0;
}

.chapter-sequence {
  font-size: var(--text-xs);
  color: var(--color-muted);
  min-width: 42px;
  font-family: var(--font-number);
  font-variant-numeric: tabular-nums;
}

.chapter-title {
  font-size: var(--text-sm);
  color: var(--color-body);
  flex: 1;
  line-height: var(--leading-normal);
}
</style>
