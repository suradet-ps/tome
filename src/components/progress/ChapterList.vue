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

function statusIcon(status?: ReadingStatus) {
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

function statusColor(status?: ReadingStatus) {
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
        :style="{ paddingLeft: `${props.depth * 14 + 8}px` }"
        @click="emit('select', chapter)"
      >
        <button v-if="chapter.children?.length" type="button" class="chapter-expand" @click.stop="toggleExpand(chapter.id)">
          <ChevronDown v-if="expanded[chapter.id]" :size="12" />
          <ChevronRight v-else :size="12" />
        </button>
        <span v-else class="chapter-expand chapter-expand--spacer"></span>

        <component
          :is="statusIcon(progressStore.getProgress(chapter.id)?.status)"
          :size="14"
          :style="{ color: statusColor(progressStore.getProgress(chapter.id)?.status) }"
          class="chapter-icon"
        />

        <span class="chapter-seq numeric">{{ chapter.sequence_number }}</span>
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
  gap: 2px;
}

.chapter-list--nested {
  margin-left: 18px;
  padding-top: 2px;
  border-left: 1px solid var(--color-hairline);
}

.chapter-row {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  min-height: 36px;
  padding: 6px var(--space-sm);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.chapter-row:hover {
  background: var(--color-surface-elevated);
}

.chapter-row--active {
  background: rgba(252, 213, 53, 0.08);
}

.chapter-row--active .chapter-title {
  color: var(--color-on-dark);
}

.chapter-expand {
  width: 16px;
  height: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--color-muted);
  border-radius: var(--radius-sm);
}

.chapter-expand:hover {
  color: var(--color-on-dark);
}

.chapter-expand--spacer {
  cursor: default;
}

.chapter-icon {
  flex-shrink: 0;
}

.chapter-seq {
  font-size: var(--text-xs);
  color: var(--color-muted);
  min-width: 28px;
  flex-shrink: 0;
}

.chapter-title {
  font-size: var(--text-sm);
  color: var(--color-body);
  flex: 1;
  line-height: 1.3;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
