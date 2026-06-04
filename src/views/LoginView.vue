<script setup lang="ts">
import { BookOpen } from 'lucide-vue-next';
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import BaseButton from '@/components/common/BaseButton.vue';
import BaseInput from '@/components/common/BaseInput.vue';
import { supabaseConfigError } from '@/lib/supabase';
import { useAuthStore } from '@/stores/auth';

const auth = useAuthStore();
const router = useRouter();

const email = ref('');
const password = ref('');
const error = ref('');
const configurationMessage = computed(() => supabaseConfigError);

async function handleLogin() {
  error.value = '';

  try {
    await auth.signIn(email.value, password.value);
    await router.push('/');
  } catch (caughtError) {
    error.value = caughtError instanceof Error ? caughtError.message : 'Sign in failed';
  }
}
</script>

<template>
  <div class="auth">
    <div class="auth__card">
      <div class="auth__brand">
        <BookOpen :size="22" />
        <span>Tome</span>
      </div>

      <div class="auth__intro">
        <h1 class="auth__title">Welcome back</h1>
        <p class="auth__subtitle">Sign in to keep tracking your reading.</p>
      </div>

      <p v-if="configurationMessage" class="notice">{{ configurationMessage }}</p>

      <form class="auth__form" aria-label="Sign in" @submit.prevent="handleLogin">
        <BaseInput v-model="email" label="Email" type="email" placeholder="you@example.com" />
        <BaseInput v-model="password" label="Password" type="password" placeholder="••••••••" />
        <p v-if="error" class="auth__error">{{ error }}</p>
        <BaseButton type="submit" :loading="auth.loading" block>Sign in</BaseButton>
      </form>

      <p class="auth__switch">
        Don't have an account?
        <RouterLink to="/register" class="auth__link">Create one</RouterLink>
      </p>
    </div>
  </div>
</template>

<style scoped>
.auth {
  min-height: 100vh;
  background: var(--color-canvas);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-lg);
}

.auth__card {
  width: 100%;
  max-width: 400px;
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  padding: var(--space-xl);
  background: var(--color-surface-card);
  border: 1px solid var(--color-hairline);
  border-radius: var(--radius-xl);
}

.auth__brand {
  display: inline-flex;
  align-items: center;
  gap: var(--space-xs);
  font-size: var(--text-md);
  font-weight: var(--weight-bold);
  color: var(--color-on-dark);
  letter-spacing: -0.01em;
}

.auth__brand svg {
  color: var(--color-primary);
}

.auth__intro {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.auth__title {
  font-family: var(--font-display);
  font-size: var(--text-xl);
  font-weight: var(--weight-bold);
  color: var(--color-on-dark);
  letter-spacing: -0.02em;
}

.auth__subtitle {
  color: var(--color-muted);
  font-size: var(--text-sm);
}

.auth__form {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.auth__error {
  font-size: var(--text-sm);
  color: var(--color-danger);
  background: rgba(246, 70, 93, 0.08);
  border: 1px solid rgba(246, 70, 93, 0.24);
  border-radius: var(--radius-md);
  padding: var(--space-xs) var(--space-sm);
}

.auth__switch {
  color: var(--color-muted);
  font-size: var(--text-sm);
  text-align: center;
}

.auth__link {
  color: var(--color-primary);
  font-weight: var(--weight-semibold);
}

.auth__link:hover {
  text-decoration: underline;
}

@media (max-width: 480px) {
  .auth {
    padding: var(--space-md);
  }

  .auth__card {
    padding: var(--space-lg);
  }
}
</style>
