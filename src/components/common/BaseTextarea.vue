<script setup lang="ts">
import { useId } from 'vue'

interface Props {
  modelValue: string
  placeholder?: string
  label?: string
  error?: string
  disabled?: boolean
  rows?: number
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  rows: 5,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const inputId = useId()
</script>

<template>
  <div class="textarea-group">
    <label v-if="props.label" class="textarea-label" :for="inputId">{{ props.label }}</label>
    <textarea
      :id="inputId"
      :value="props.modelValue"
      :rows="props.rows"
      :placeholder="props.placeholder"
      :disabled="props.disabled"
      class="textarea-field"
      :class="{ 'textarea-field--error': props.error }"
      @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    ></textarea>
    <p v-if="props.error" class="textarea-error">{{ props.error }}</p>
  </div>
</template>

<style scoped>
.textarea-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.textarea-label {
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-muted);
}

.textarea-field {
  width: 100%;
  min-height: 120px;
  resize: vertical;
  background: var(--color-surface-card);
  border: 1px solid var(--color-hairline);
  border-radius: var(--radius-md);
  padding: 10px var(--space-md);
  color: var(--color-on-dark);
  line-height: var(--leading-relaxed);
  transition:
    background var(--transition-fast),
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
}

.textarea-field:focus {
  outline: none;
  border-color: rgba(59, 130, 246, 0.55);
  box-shadow: var(--shadow-focus);
}

.textarea-field::placeholder {
  color: var(--color-muted);
}

.textarea-field--error {
  border-color: var(--color-danger);
}

.textarea-error {
  font-size: var(--text-xs);
  color: var(--color-danger);
}
</style>
