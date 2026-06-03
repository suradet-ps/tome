<script setup lang="ts">
import { useId } from 'vue'

interface Props {
  modelValue: string
  placeholder?: string
  type?: string
  label?: string
  error?: string
  disabled?: boolean
  inputmode?: 'none' | 'text' | 'decimal' | 'numeric' | 'tel' | 'search' | 'email' | 'url'
}

const props = withDefaults(defineProps<Props>(), {
  type: 'text',
  disabled: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const inputId = useId()
</script>

<template>
  <div class="input-group">
    <label v-if="props.label" class="input-label" :for="inputId">{{ props.label }}</label>
    <input
      :id="inputId"
      :type="props.type"
      :value="props.modelValue"
      :placeholder="props.placeholder"
      :disabled="props.disabled"
      :inputmode="props.inputmode"
      class="input-field"
      :class="{ 'input-field--error': props.error }"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    />
    <p v-if="props.error" class="input-error">{{ props.error }}</p>
  </div>
</template>

<style scoped>
.input-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.input-label {
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-muted);
}

.input-field {
  background: var(--color-surface-card);
  border: 1px solid var(--color-hairline);
  border-radius: var(--radius-md);
  padding: 10px var(--space-md);
  min-height: 40px;
  color: var(--color-on-dark);
  font-size: var(--text-base);
  transition:
    background var(--transition-fast),
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
  width: 100%;
}

.input-field::placeholder {
  color: var(--color-muted);
}

.input-field:focus {
  outline: none;
  border-color: rgba(59, 130, 246, 0.55);
  box-shadow: var(--shadow-focus);
}

.input-field:disabled {
  opacity: 0.65;
  cursor: not-allowed;
}

.input-field--error {
  border-color: var(--color-danger);
}

.input-error {
  font-size: var(--text-xs);
  color: var(--color-danger);
}
</style>
