<script setup lang="ts">
import { BookOpen, Brain, LayoutDashboard, LogOut, Menu, X } from 'lucide-vue-next'
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const router = useRouter()
const route = useRoute()
const mobileOpen = ref(false)

watch(
  () => route.fullPath,
  () => {
    mobileOpen.value = false
  },
)

async function handleSignOut() {
  await auth.signOut()
  await router.push('/login')
}

function isActive(path: string) {
  if (path === '/') return route.path === '/' || route.path.startsWith('/books')
  return route.path.startsWith(path)
}
</script>

<template>
  <header class="topbar">
    <div class="topbar__inner">
      <RouterLink to="/" class="topbar__brand" aria-label="Tome">
        <BookOpen :size="20" />
        <span>Tome</span>
      </RouterLink>

      <nav class="topbar__nav" aria-label="Primary">
        <RouterLink to="/" class="topbar__link" :class="{ 'topbar__link--active': isActive('/') }">
          <LayoutDashboard :size="15" />
          <span>Library</span>
        </RouterLink>
        <RouterLink to="/review" class="topbar__link" :class="{ 'topbar__link--active': isActive('/review') }">
          <Brain :size="15" />
          <span>Review</span>
        </RouterLink>
      </nav>

      <div class="topbar__actions">
        <div class="topbar__user" :title="auth.profile?.username ?? auth.user?.email ?? ''">
          <div class="topbar__avatar">{{ auth.profile?.username?.charAt(0).toUpperCase() ?? '?' }}</div>
          <span class="topbar__user-name">{{ auth.profile?.username ?? auth.user?.email }}</span>
        </div>
        <button class="topbar__icon-btn" type="button" @click="handleSignOut" aria-label="Sign out" title="Sign out">
          <LogOut :size="16" />
        </button>
        <button
          class="topbar__icon-btn topbar__menu-toggle"
          type="button"
          :aria-expanded="mobileOpen"
          aria-label="Menu"
          @click="mobileOpen = !mobileOpen"
        >
          <Menu v-if="!mobileOpen" :size="18" />
          <X v-else :size="18" />
        </button>
      </div>
    </div>

    <Transition name="sheet">
      <div v-if="mobileOpen" class="topbar__sheet">
        <RouterLink to="/" class="topbar__sheet-link" :class="{ 'topbar__sheet-link--active': isActive('/') }">
          <LayoutDashboard :size="16" />
          Library
        </RouterLink>
        <RouterLink to="/review" class="topbar__sheet-link" :class="{ 'topbar__sheet-link--active': isActive('/review') }">
          <Brain :size="16" />
          Review
        </RouterLink>
        <button type="button" class="topbar__sheet-link topbar__sheet-link--danger" @click="handleSignOut">
          <LogOut :size="16" />
          Sign out
        </button>
      </div>
    </Transition>
  </header>
</template>

<style scoped>
.topbar {
  position: sticky;
  top: 0;
  z-index: var(--z-header);
  background: rgba(11, 14, 17, 0.92);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid var(--color-hairline);
}

.topbar__inner {
  display: flex;
  align-items: center;
  gap: var(--space-lg);
  min-height: 56px;
  width: min(100%, var(--container-default));
  margin: 0 auto;
  padding: 0 var(--space-lg);
}

.topbar__brand {
  display: inline-flex;
  align-items: center;
  gap: var(--space-xs);
  font-size: var(--text-md);
  font-weight: var(--weight-bold);
  color: var(--color-on-dark);
  letter-spacing: -0.01em;
}

.topbar__brand svg {
  color: var(--color-primary);
}

.topbar__nav {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: var(--space-md);
}

.topbar__link {
  display: inline-flex;
  align-items: center;
  gap: var(--space-xs);
  height: 32px;
  padding: 0 var(--space-sm);
  border-radius: var(--radius-md);
  color: var(--color-muted);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  transition: background var(--transition-fast), color var(--transition-fast);
}

.topbar__link:hover {
  background: var(--color-surface-card);
  color: var(--color-on-dark);
}

.topbar__link--active {
  background: var(--color-surface-card);
  color: var(--color-on-dark);
}

.topbar__actions {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: var(--space-xs);
}

.topbar__user {
  display: inline-flex;
  align-items: center;
  gap: var(--space-xs);
  padding: 0 var(--space-xs);
  max-width: 200px;
}

.topbar__avatar {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-full);
  background: var(--color-primary);
  color: var(--color-on-primary);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: var(--text-xs);
  font-weight: var(--weight-bold);
  flex-shrink: 0;
}

.topbar__user-name {
  font-size: var(--text-sm);
  color: var(--color-body);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.topbar__icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  color: var(--color-muted);
  transition: background var(--transition-fast), color var(--transition-fast);
}

.topbar__icon-btn:hover {
  background: var(--color-surface-card);
  color: var(--color-on-dark);
}

.topbar__menu-toggle {
  display: none;
}

.topbar__sheet {
  display: none;
  flex-direction: column;
  gap: 4px;
  padding: var(--space-sm) var(--space-md) var(--space-md);
  border-top: 1px solid var(--color-hairline);
  background: var(--color-canvas);
}

.topbar__sheet-link {
  display: inline-flex;
  align-items: center;
  gap: var(--space-sm);
  min-height: 40px;
  padding: 0 var(--space-md);
  border-radius: var(--radius-md);
  color: var(--color-body);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
}

.topbar__sheet-link--active {
  background: var(--color-surface-card);
  color: var(--color-on-dark);
}

.topbar__sheet-link--danger {
  color: var(--color-danger);
  margin-top: var(--space-xs);
}

.sheet-enter-active,
.sheet-leave-active {
  transition: opacity var(--transition-base);
}

.sheet-enter-from,
.sheet-leave-to {
  opacity: 0;
}

@media (max-width: 768px) {
  .topbar__inner {
    padding: 0 var(--space-md);
    gap: var(--space-sm);
  }

  .topbar__nav,
  .topbar__user-name {
    display: none;
  }

  .topbar__menu-toggle {
    display: inline-flex;
  }

  .topbar__sheet {
    display: flex;
  }
}
</style>
