<script setup lang="ts">
import { RouterView } from 'vue-router';
import AppTopbar from '@/components/layout/AppSidebar.vue';
import { useAuthStore } from '@/stores/auth';

const auth = useAuthStore();
</script>

<template>
  <div class="app">
    <div v-if="auth.user" class="app-shell">
      <AppTopbar />
      <main class="app-main">
        <div class="app-main__inner">
          <RouterView />
        </div>
      </main>
    </div>
    <RouterView v-else />
  </div>
</template>

<style scoped>
.app {
  min-height: 100vh;
}
</style>
