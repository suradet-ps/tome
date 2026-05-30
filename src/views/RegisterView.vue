<script setup lang="ts">
import { BookOpen, Brain, FolderKanban, NotebookPen } from 'lucide-vue-next'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import BaseButton from '@/components/common/BaseButton.vue'
import BaseInput from '@/components/common/BaseInput.vue'
import { supabaseConfigError } from '@/lib/supabase'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const router = useRouter()

const email = ref('')
const password = ref('')
const username = ref('')
const error = ref('')
const configurationMessage = computed(() => supabaseConfigError)

async function handleRegister() {
  error.value = ''

  if (username.value.trim().length < 3) {
    error.value = 'Username must be at least 3 characters'
    return
  }

  try {
    await auth.signUp(email.value, password.value, username.value.trim())
    await router.push('/')
  } catch (caughtError) {
    error.value = caughtError instanceof Error ? caughtError.message : 'Registration failed'
  }
}
</script>

<template>
  <div class="auth-page">
    <div class="auth-shell">
      <section class="auth-hero surface-panel surface-panel--soft">
        <div class="auth-brand">
          <BookOpen :size="22" class="auth-brand__icon" />
          <div>
            <span class="auth-brand__name">Tome</span>
            <span class="auth-brand__sub">Technical Reading Tracker</span>
          </div>
        </div>

        <div class="auth-hero__copy">
          <p class="eyebrow">Setup your system</p>
          <h1 class="auth-hero__title">Create a reading workspace that feels built, not improvised.</h1>
          <p class="auth-hero__subtitle">
            Bring chapter structure, note discipline, and review loops together from the first session.
          </p>
        </div>

        <div class="auth-hero__stats">
          <div class="auth-hero__stat">
            <FolderKanban :size="18" />
            <div>
              <strong>Library board</strong>
              <span>Organize books and nested chapters with clear progress.</span>
            </div>
          </div>
          <div class="auth-hero__stat">
            <NotebookPen :size="18" />
            <div>
              <strong>Note workspace</strong>
              <span>Write markdown notes with code-friendly formatting.</span>
            </div>
          </div>
          <div class="auth-hero__stat">
            <Brain :size="18" />
            <div>
              <strong>Recall system</strong>
              <span>Turn fragile understanding into stable memory.</span>
            </div>
          </div>
        </div>
      </section>

      <section class="auth-panel">
        <div class="auth-card">
          <div class="auth-card__header">
            <p class="eyebrow">Create account</p>
            <h2 class="auth-title">Start your reading system</h2>
            <p class="auth-subtitle">Create a profile and begin tracking technical books with structure.</p>
          </div>

          <p v-if="configurationMessage" class="notice auth-notice">{{ configurationMessage }}</p>

          <form class="auth-form" @submit.prevent="handleRegister">
            <BaseInput v-model="username" label="Username" placeholder="rustacean42" />
            <BaseInput v-model="email" label="Email" type="email" placeholder="you@example.com" />
            <BaseInput v-model="password" label="Password" type="password" placeholder="••••••••" />
            <p v-if="error" class="auth-error">{{ error }}</p>
            <BaseButton type="submit" :loading="auth.loading" block>Create Account</BaseButton>
          </form>

          <p class="auth-switch">
            Already have an account?
            <RouterLink to="/login" class="auth-link">Sign in</RouterLink>
          </p>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.auth-page {
  min-height: 100vh;
  background: var(--color-canvas);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-xl);
}

.auth-shell {
  width: min(100%, 1180px);
  display: grid;
  grid-template-columns: minmax(0, 1.15fr) minmax(360px, 0.85fr);
  border: 1px solid var(--color-hairline);
  border-radius: 28px;
  overflow: hidden;
  background: var(--color-canvas);
  box-shadow: var(--shadow-panel);
}

.auth-hero,
.auth-panel {
  min-height: 720px;
}

.auth-hero {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: var(--space-xxl);
  padding: 48px;
  border-right: 1px solid var(--color-hairline);
}

.auth-brand {
  display: inline-flex;
  align-items: center;
  gap: var(--space-sm);
}

.auth-brand__icon {
  color: var(--color-primary);
}

.auth-brand__name {
  display: block;
  font-size: var(--text-lg);
  font-weight: var(--weight-bold);
  color: var(--color-on-dark);
}

.auth-brand__sub {
  display: block;
  font-size: var(--text-xs);
  color: var(--color-muted);
}

.auth-hero__copy {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.auth-hero__title {
  font-family: var(--font-display);
  font-size: clamp(var(--text-3xl), 6vw, var(--text-hero));
  font-weight: var(--weight-bold);
  line-height: 0.98;
  letter-spacing: -0.05em;
  color: var(--color-on-dark);
  max-width: 600px;
}

.auth-hero__subtitle {
  max-width: 520px;
  color: var(--color-muted-strong);
  font-size: var(--text-md);
  line-height: var(--leading-relaxed);
}

.auth-hero__stats {
  display: grid;
  gap: var(--space-md);
}

.auth-hero__stat {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: var(--space-sm);
  padding: var(--space-lg);
  border: 1px solid rgba(43, 49, 57, 0.95);
  border-radius: var(--radius-xl);
  background: rgba(11, 14, 17, 0.5);
}

.auth-hero__stat svg {
  color: var(--color-primary);
}

.auth-hero__stat strong {
  display: block;
  color: var(--color-on-dark);
  font-size: var(--text-base);
  margin-bottom: 4px;
}

.auth-hero__stat span {
  color: var(--color-muted);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
}

.auth-panel {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-xl);
}

.auth-card {
  width: 100%;
  max-width: 420px;
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.auth-card__header {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.auth-title {
  font-size: var(--text-3xl);
  font-weight: var(--weight-bold);
  color: var(--color-on-dark);
  letter-spacing: -0.03em;
}

.auth-subtitle {
  color: var(--color-muted);
}

.auth-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.auth-notice {
  margin: 0;
}

.auth-error {
  font-size: var(--text-sm);
  color: var(--color-danger);
  background: rgba(246, 70, 93, 0.08);
  border: 1px solid rgba(246, 70, 93, 0.24);
  border-radius: var(--radius-md);
  padding: var(--space-sm) var(--space-md);
}

.auth-switch {
  color: var(--color-muted);
  font-size: var(--text-sm);
}

.auth-link {
  color: var(--color-primary);
  font-weight: var(--weight-semibold);
}

.auth-link:hover {
  text-decoration: underline;
}

@media (max-width: 960px) {
  .auth-shell {
    grid-template-columns: 1fr;
  }

  .auth-hero,
  .auth-panel {
    min-height: auto;
  }

  .auth-hero {
    border-right: 0;
    border-bottom: 1px solid var(--color-hairline);
  }
}

@media (max-width: 640px) {
  .auth-page {
    padding: var(--space-md);
  }

  .auth-hero,
  .auth-panel {
    padding: var(--space-xl);
  }

  .auth-hero__title {
    font-size: clamp(var(--text-2xl), 12vw, 52px);
  }
}
</style>
