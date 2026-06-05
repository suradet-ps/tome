<script setup lang="ts">
import { X } from '@lucide/vue';
import { nextTick, ref, watch } from 'vue';

interface Props {
  title?: string;
  modelValue: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>();

const containerRef = ref<HTMLElement | null>(null);
let previouslyFocused: HTMLElement | null = null;

function close() {
  emit('update:modelValue', false);
}

function getFocusable(root: HTMLElement): HTMLElement[] {
  const selector =
    'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';
  return Array.from(root.querySelectorAll<HTMLElement>(selector)).filter(
    (el) => !el.hasAttribute('aria-hidden') && el.offsetParent !== null,
  );
}

function onKeydown(event: KeyboardEvent) {
  if (!props.modelValue) return;
  if (event.key === 'Escape') {
    event.stopPropagation();
    close();
    return;
  }
  if (event.key !== 'Tab' || !containerRef.value) return;
  const focusable = getFocusable(containerRef.value);
  if (focusable.length === 0) {
    event.preventDefault();
    return;
  }
  const first = focusable[0];
  const last = focusable[focusable.length - 1];
  const active = document.activeElement as HTMLElement | null;
  if (event.shiftKey && (active === first || !containerRef.value.contains(active))) {
    event.preventDefault();
    last.focus();
  } else if (!event.shiftKey && active === last) {
    event.preventDefault();
    first.focus();
  }
}

watch(
  () => props.modelValue,
  async (open) => {
    if (open) {
      previouslyFocused = document.activeElement as HTMLElement | null;
      document.addEventListener('keydown', onKeydown);
      await nextTick();
      const focusable = containerRef.value ? getFocusable(containerRef.value) : [];
      if (focusable.length > 0) {
        focusable[0].focus();
      } else {
        containerRef.value?.focus();
      }
    } else {
      document.removeEventListener('keydown', onKeydown);
      previouslyFocused?.focus();
    }
  },
);
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="props.modelValue" class="modal-overlay" @click.self="close">
        <div
          ref="containerRef"
          class="modal-container"
          role="dialog"
          aria-modal="true"
          tabindex="-1"
          :aria-label="props.title || 'Dialog'"
        >
          <div class="modal-header">
            <h3 class="modal-title">{{ props.title }}</h3>
            <button class="modal-close" type="button" @click="close" aria-label="Close">
              <X :size="18" />
            </button>
          </div>
          <div class="modal-body">
            <slot />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(11, 14, 17, 0.82);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  padding: var(--space-lg);
}

.modal-container {
  background: var(--color-surface-card);
  border: 1px solid var(--color-hairline);
  border-radius: var(--radius-xl);
  width: 100%;
  max-width: 480px;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: var(--shadow-panel);
  outline: none;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-lg);
  border-bottom: 1px solid var(--color-hairline);
}

.modal-title {
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
}

.modal-close {
  color: var(--color-muted);
  padding: var(--space-xs);
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.modal-close:hover {
  color: var(--color-on-dark);
  background: var(--color-surface-elevated);
}

.modal-body {
  padding: var(--space-lg);
}

.modal-enter-active,
.modal-leave-active {
  transition: opacity var(--transition-base);
}

.modal-enter-active .modal-container,
.modal-leave-active .modal-container {
  transition: transform var(--transition-base);
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-container,
.modal-leave-to .modal-container {
  transform: scale(0.95) translateY(-8px);
}
</style>
