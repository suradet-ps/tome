<script setup lang="ts">
interface Props {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  loading?: boolean;
  disabled?: boolean;
  type?: 'button' | 'submit' | 'reset';
  block?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  loading: false,
  disabled: false,
  type: 'button',
  block: false,
});
</script>

<template>
  <button
    :type="props.type"
    class="btn"
    :class="[
      `btn--${props.variant}`,
      `btn--${props.size}`,
      {
        'btn--loading': props.loading,
        'btn--block': props.block,
      },
    ]"
    :disabled="props.disabled || props.loading"
  >
    <span v-if="props.loading" class="btn__spinner" aria-hidden="true"></span>
    <slot />
  </button>
</template>

<style scoped>
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-xs);
  font-family: var(--font-display);
  font-weight: var(--weight-semibold);
  font-size: var(--text-base);
  line-height: 1;
  border-radius: var(--radius-md);
  min-height: 40px;
  padding: 0 var(--space-lg);
  transition:
    background var(--transition-fast),
    color var(--transition-fast),
    border-color var(--transition-fast),
    transform var(--transition-fast),
    box-shadow var(--transition-fast);
  white-space: nowrap;
  cursor: pointer;
  border: 1px solid transparent;
  font-variant-numeric: tabular-nums;
}

.btn--block {
  width: 100%;
}

.btn:disabled {
  cursor: not-allowed;
  transform: none;
}

.btn:focus-visible {
  outline: none;
  box-shadow: var(--shadow-focus);
}

.btn--sm {
  min-height: 32px;
  padding: 0 var(--space-sm);
  font-size: var(--text-xs);
}

.btn--md {
  min-height: 40px;
  padding: 0 var(--space-lg);
}

.btn--lg {
  padding: 0 var(--space-xl);
  min-height: 48px;
  font-size: var(--text-md);
}

.btn--primary {
  background: var(--color-primary);
  color: var(--color-on-primary);
  box-shadow: inset 0 -1px 0 rgba(24, 26, 32, 0.24);
}

.btn--primary:hover:not(:disabled) {
  background: var(--color-primary-active);
  transform: translateY(-1px);
}

.btn--primary:disabled {
  background: var(--color-primary-disabled);
  color: rgba(234, 236, 239, 0.72);
}

.btn--secondary {
  background: var(--color-surface-card);
  color: var(--color-on-dark);
  border-color: var(--color-hairline);
}

.btn--secondary:hover:not(:disabled) {
  background: var(--color-surface-elevated);
  border-color: rgba(234, 236, 239, 0.16);
}

.btn--danger {
  background: rgba(246, 70, 93, 0.15);
  color: var(--color-danger);
  border-color: rgba(246, 70, 93, 0.3);
}

.btn--danger:hover:not(:disabled) {
  background: rgba(246, 70, 93, 0.25);
}

.btn--ghost {
  background: transparent;
  color: var(--color-muted-strong);
  border-color: transparent;
}

.btn--ghost:hover:not(:disabled) {
  background: rgba(43, 49, 57, 0.72);
  color: var(--color-on-dark);
}

.btn__spinner {
  width: 14px;
  height: 14px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: var(--radius-full);
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
