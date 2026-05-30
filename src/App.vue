<script setup lang="ts">
import { RouterView } from 'vue-router'
import AppSidebar from '@/components/layout/AppSidebar.vue'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
</script>

<template>
  <div class="app" :class="{ 'app--authed': auth.user }">
    <div v-if="auth.user" class="app-shell">
      <AppSidebar />
      <main class="app-main">
        <div class="app-main__inner">
          <RouterView />
        </div>
      </main>
      <footer class="app-footer">
        <div class="app-footer__inner container--wide">
          <div class="app-footer__intro">
            <p class="app-footer__eyebrow">Tome Reading System</p>
            <h2 class="app-footer__title">Keep technical reading compounding.</h2>
            <p class="app-footer__copy">
              Track chapters, capture durable notes, and review concepts before they decay.
            </p>
          </div>

          <div class="app-footer__grid">
            <div class="app-footer__column">
              <h3>Track</h3>
              <p>Structured progress across books, chapters, and deep sub-sections.</p>
            </div>
            <div class="app-footer__column">
              <h3>Reflect</h3>
              <p>Markdown notes with code highlighting for dense technical material.</p>
            </div>
            <div class="app-footer__column">
              <h3>Review</h3>
              <p>Flashcards and focus sessions tuned for long-term retention.</p>
            </div>
          </div>
        </div>
      </footer>
    </div>
    <RouterView v-else />
  </div>
</template>

<style scoped>
.app {
  min-height: 100vh;
}

.app-footer {
  background: var(--color-surface-soft-light);
  border-top: 1px solid var(--color-hairline-light);
  color: var(--color-body-on-light);
}

.app-footer__inner {
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) minmax(0, 1.8fr);
  gap: var(--space-xxl);
  padding-top: 64px;
  padding-bottom: 64px;
}

.app-footer__eyebrow {
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--color-muted);
  margin-bottom: var(--space-sm);
}

.app-footer__title {
  font-family: var(--font-display);
  font-size: clamp(var(--text-xl), 3vw, var(--text-3xl));
  font-weight: var(--weight-bold);
  color: var(--color-ink);
  margin-bottom: var(--space-sm);
}

.app-footer__copy {
  max-width: 420px;
  color: rgba(24, 26, 32, 0.72);
}

.app-footer__grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-lg);
}

.app-footer__column {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.app-footer__column h3 {
  font-size: var(--text-base);
  font-weight: var(--weight-semibold);
  color: var(--color-ink);
}

.app-footer__column p {
  color: rgba(24, 26, 32, 0.72);
  font-size: var(--text-sm);
}

@media (max-width: 900px) {
  .app-footer__inner {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 640px) {
  .app-footer__inner {
    gap: var(--space-xl);
    padding-top: var(--space-xxl);
    padding-bottom: var(--space-xxl);
  }

  .app-footer__grid {
    grid-template-columns: 1fr;
  }
}
</style>
