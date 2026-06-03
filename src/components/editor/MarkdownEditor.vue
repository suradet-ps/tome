<script setup lang="ts">
import { Eye, EyeOff, Save } from 'lucide-vue-next'
import { ref, watch } from 'vue'
import { useMarkdown } from '@/composables/useMarkdown'
import BaseButton from '@/components/common/BaseButton.vue'

interface Props {
  modelValue: string
  saving?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  saving: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  save: []
}>()

const { isPreview, renderMarkdown, togglePreview } = useMarkdown()
const localValue = ref(props.modelValue)

watch(
  () => props.modelValue,
  (value) => {
    localValue.value = value
  },
)

watch(localValue, (value) => {
  emit('update:modelValue', value)
})

function setPreview(nextPreview: boolean) {
  if (isPreview.value !== nextPreview) {
    togglePreview()
  }
}

const placeholder = 'Write your notes in Markdown...'
</script>

<template>
  <div class="editor">
    <div class="editor__toolbar">
      <div class="editor__switch" role="tablist" aria-label="Editor mode">
        <button
          type="button"
          role="tab"
          class="editor__toggle"
          :class="{ 'editor__toggle--active': !isPreview }"
          @click="setPreview(false)"
        >
          <EyeOff :size="13" />
          Write
        </button>
        <button
          type="button"
          role="tab"
          class="editor__toggle"
          :class="{ 'editor__toggle--active': isPreview }"
          @click="setPreview(true)"
        >
          <Eye :size="13" />
          Preview
        </button>
      </div>
      <BaseButton size="sm" @click="emit('save')" :loading="props.saving">
        <Save :size="13" />
        Save
      </BaseButton>
    </div>

    <div class="editor__body">
      <textarea
        v-if="!isPreview"
        v-model="localValue"
        class="editor__textarea"
        :placeholder="placeholder"
        spellcheck="false"
      ></textarea>

      <div v-else class="editor__preview markdown-body" v-html="renderMarkdown(localValue)"></div>
    </div>
  </div>
</template>

<style scoped>
.editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  border: 1px solid var(--color-hairline);
  border-radius: var(--radius-xl);
  overflow: hidden;
  background: var(--color-surface-card);
}

.editor__toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-sm);
  padding: var(--space-xs) var(--space-sm);
  background: var(--color-surface-elevated);
  border-bottom: 1px solid var(--color-hairline);
}

.editor__switch {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  padding: 2px;
  background: var(--color-canvas);
  border: 1px solid var(--color-hairline);
  border-radius: var(--radius-pill);
}

.editor__toggle {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 26px;
  padding: 0 var(--space-sm);
  border-radius: var(--radius-pill);
  color: var(--color-muted);
  font-size: var(--text-xs);
  font-weight: var(--weight-medium);
  transition: all var(--transition-fast);
}

.editor__toggle:hover {
  color: var(--color-on-dark);
}

.editor__toggle--active {
  background: var(--color-surface-elevated);
  color: var(--color-on-dark);
}

.editor__body {
  flex: 1;
  overflow: hidden;
}

.editor__textarea {
  width: 100%;
  height: 100%;
  min-height: 420px;
  background: var(--color-canvas);
  border: none;
  padding: var(--space-lg);
  color: var(--color-body);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
  resize: none;
  outline: none;
}

.editor__preview {
  padding: var(--space-lg);
  min-height: 420px;
  overflow-y: auto;
  color: var(--color-body);
  font-size: var(--text-base);
  line-height: var(--leading-relaxed);
}

.editor__preview :deep(h1),
.editor__preview :deep(h2),
.editor__preview :deep(h3) {
  color: var(--color-on-dark);
  margin-top: var(--space-lg);
  margin-bottom: var(--space-sm);
  font-family: var(--font-display);
}

.editor__preview :deep(h1) {
  font-size: var(--text-xl);
}

.editor__preview :deep(h2) {
  font-size: var(--text-lg);
}

.editor__preview :deep(h3) {
  font-size: var(--text-md);
}

.editor__preview :deep(p) {
  margin-bottom: var(--space-md);
}

.editor__preview :deep(ul),
.editor__preview :deep(ol) {
  padding-left: var(--space-lg);
  margin-bottom: var(--space-md);
}

.editor__preview :deep(li) {
  margin-bottom: var(--space-xs);
  list-style: disc;
}

.editor__preview :deep(code) {
  background: var(--color-surface-elevated);
  border-radius: var(--radius-sm);
  padding: 2px 6px;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--color-primary);
}

.editor__preview :deep(pre) {
  background: var(--color-canvas) !important;
  border-radius: var(--radius-md);
  padding: var(--space-md);
  overflow-x: auto;
  margin-bottom: var(--space-md);
  border: 1px solid var(--color-hairline);
}

.editor__preview :deep(pre code) {
  background: none;
  padding: 0;
  color: inherit;
}

.editor__preview :deep(blockquote) {
  border-left: 3px solid var(--color-primary);
  padding-left: var(--space-md);
  color: var(--color-muted-strong);
  margin-bottom: var(--space-md);
}
</style>
