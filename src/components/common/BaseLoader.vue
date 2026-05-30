<script setup lang="ts">
interface Props {
  size?: number
}

withDefaults(defineProps<Props>(), {
  size: 24,
})
</script>

<template>
  <div class="loader" :style="{ width: `${size}px`, height: `${size}px` }" aria-hidden="true">
    <div class="loader__ring"></div>
  </div>
</template>

<style scoped>
.loader {
  display: inline-flex;
  align-items: center;
  justify-content: center;
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
