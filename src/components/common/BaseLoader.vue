<script setup lang="ts">
interface Props {
  size?: number;
}

withDefaults(defineProps<Props>(), {
  size: 24,
});
</script>

<template>
  <div
    class="loader"
    :style="{ '--loader-size': `${size}px` }"
    role="status"
    aria-live="polite"
    aria-label="Loading"
  >
    <div class="loader__ring"></div>
  </div>
</template>

<style scoped>
.loader {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: var(--loader-size);
  height: var(--loader-size);
}

.loader__ring {
  width: 100%;
  height: 100%;
  border: 2px solid rgba(43, 49, 57, 0.8);
  border-top-color: var(--color-primary);
  border-radius: var(--radius-full);
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
