<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { BookOpen, Brain, ChevronRight, LayoutDashboard, LogOut, Menu, X } from 'lucide-vue-next'
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useBooksStore } from '@/stores/books'

const auth = useAuthStore()
const booksStore = useBooksStore()
const { books } = storeToRefs(booksStore)
const router = useRouter()
const route = useRoute()
const mobileMenuOpen = ref(false)

onMounted(() => {
  if (auth.user) {
    void booksStore.fetchBooks()
  }
})

watch(
  () => route.fullPath,
  () => {
    mobileMenuOpen.value = false
  },
)

async function handleSignOut() {
  await auth.signOut()
  await router.push('/login')
}

function isActive(path: string) {
  return route.path === path
}

const currentBookId = computed(() => (typeof route.params.id === 'string' ? route.params.id : ''))
const currentBook = computed(() => books.value.find((book) => book.id === currentBookId.value))
</script>

<template>
  <header class="topbar">
    <div class="topbar__main container--wide">
      <RouterLink to="/" class="topbar__brand" aria-label="Tome dashboard">
        <BookOpen :size="22" class="topbar__brand-icon" />
        <div>
          <span class="topbar__brand-name">Tome</span>
          <span class="topbar__brand-sub">Technical Reading Tracker</span>
        </div>
      </RouterLink>

      <nav class="topbar__nav" aria-label="Primary">
        <RouterLink to="/" class="topbar__nav-item" :class="{ 'topbar__nav-item--active': isActive('/') }">
          <LayoutDashboard :size="16" />
          Dashboard
        </RouterLink>
        <RouterLink
          to="/review"
          class="topbar__nav-item"
          :class="{ 'topbar__nav-item--active': isActive('/review') }"
        >
          <Brain :size="16" />
          Review
        </RouterLink>
      </nav>

      <div class="topbar__actions">
        <div class="topbar__user">
          <div class="topbar__avatar">{{ auth.profile?.username?.charAt(0).toUpperCase() ?? '?' }}</div>
          <div class="topbar__user-copy">
            <span class="topbar__user-name">{{ auth.profile?.username ?? auth.user?.email }}</span>
            <span class="topbar__user-role">Deep work mode</span>
          </div>
        </div>
        <button class="topbar__signout" type="button" @click="handleSignOut">
          <LogOut :size="16" />
          Sign out
        </button>
        <button
          class="topbar__menu-toggle"
          type="button"
          :aria-expanded="mobileMenuOpen"
          aria-label="Toggle navigation"
          @click="mobileMenuOpen = !mobileMenuOpen"
        >
          <Menu v-if="!mobileMenuOpen" :size="18" />
          <X v-else :size="18" />
        </button>
      </div>
    </div>

    <div class="topbar__rail">
      <div class="topbar__rail-inner container--wide">
        <div class="topbar__rail-copy">
          <span class="topbar__rail-label">Library</span>
          <span class="topbar__rail-current">{{ currentBook?.title ?? 'Select a book and keep momentum.' }}</span>
        </div>
        <div class="topbar__books" aria-label="Books">
          <RouterLink
            v-for="book in books"
            :key="book.id"
            :to="`/books/${book.id}`"
            class="topbar__book-item"
            :class="{ 'topbar__book-item--active': currentBookId === book.id }"
          >
            <BookOpen :size="14" />
            <span class="topbar__book-name">{{ book.title }}</span>
            <ChevronRight :size="12" class="topbar__book-chevron" />
          </RouterLink>
        </div>
      </div>
    </div>

    <Transition name="sheet">
      <div v-if="mobileMenuOpen" class="topbar__sheet">
        <div class="topbar__sheet-card">
          <nav class="topbar__sheet-nav" aria-label="Mobile">
            <RouterLink to="/" class="topbar__sheet-link" :class="{ 'topbar__sheet-link--active': isActive('/') }">
              <LayoutDashboard :size="16" />
              Dashboard
            </RouterLink>
            <RouterLink
              to="/review"
              class="topbar__sheet-link"
              :class="{ 'topbar__sheet-link--active': isActive('/review') }"
            >
              <Brain :size="16" />
              Review
            </RouterLink>
          </nav>

          <div class="topbar__sheet-books">
            <p class="topbar__sheet-label">Books</p>
            <RouterLink
              v-for="book in books"
              :key="book.id"
              :to="`/books/${book.id}`"
              class="topbar__sheet-book"
              :class="{ 'topbar__sheet-book--active': currentBookId === book.id }"
            >
              <span>{{ book.title }}</span>
              <ChevronRight :size="14" />
            </RouterLink>
          </div>

          <button class="topbar__sheet-signout" type="button" @click="handleSignOut">
            <LogOut :size="16" />
            Sign out
          </button>
        </div>
      </div>
    </Transition>
  </header>
</template>

<style scoped>
.topbar {
  position: sticky;
  top: 0;
  z-index: var(--z-header);
  background: rgba(11, 14, 17, 0.96);
  backdrop-filter: blur(14px);
  border-bottom: 1px solid var(--color-hairline);
}

.topbar__main,
.topbar__rail-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.topbar__main {
  min-height: 72px;
  gap: var(--space-lg);
}

.topbar__brand {
  display: inline-flex;
  align-items: center;
  gap: var(--space-sm);
  min-width: 0;
}

.topbar__brand-icon {
  color: var(--color-primary);
}

.topbar__brand-name {
  display: block;
  font-size: var(--text-lg);
  font-weight: var(--weight-bold);
  color: var(--color-on-dark);
  letter-spacing: -0.03em;
}

.topbar__brand-sub {
  display: block;
  font-size: var(--text-xs);
  color: var(--color-muted);
}

.topbar__nav {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  margin-left: auto;
}

.topbar__nav-item,
.topbar__book-item,
.topbar__sheet-link,
.topbar__sheet-book,
.topbar__signout,
.topbar__menu-toggle,
.topbar__sheet-signout {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-xs);
  min-height: 40px;
  padding: 0 var(--space-md);
  border-radius: var(--radius-md);
  border: 1px solid transparent;
  color: var(--color-muted-strong);
  transition: background var(--transition-fast), border-color var(--transition-fast), color var(--transition-fast);
}

.topbar__nav-item:hover,
.topbar__book-item:hover,
.topbar__sheet-link:hover,
.topbar__sheet-book:hover,
.topbar__signout:hover,
.topbar__menu-toggle:hover,
.topbar__sheet-signout:hover {
  background: var(--color-surface-card);
  border-color: var(--color-hairline);
  color: var(--color-on-dark);
}

.topbar__nav-item--active,
.topbar__sheet-link--active {
  background: rgba(252, 213, 53, 0.1);
  border-color: rgba(252, 213, 53, 0.24);
  color: var(--color-primary);
}

.topbar__actions {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.topbar__user {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding-right: var(--space-sm);
}

.topbar__avatar {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-full);
  background: var(--color-primary);
  color: var(--color-on-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: var(--text-sm);
  font-weight: var(--weight-bold);
  flex-shrink: 0;
}

.topbar__user-copy {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.topbar__user-name {
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  color: var(--color-on-dark);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.topbar__user-role {
  font-size: var(--text-xs);
  color: var(--color-muted);
}

.topbar__signout {
  background: var(--color-surface-card);
  border-color: var(--color-hairline);
  color: var(--color-on-dark);
}

.topbar__menu-toggle {
  display: none;
  width: 40px;
  padding: 0;
}

.topbar__rail {
  border-top: 1px solid rgba(43, 49, 57, 0.55);
}

.topbar__rail-inner {
  gap: var(--space-lg);
  min-height: 60px;
}

.topbar__rail-copy {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex-shrink: 0;
}

.topbar__rail-label,
.topbar__sheet-label {
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--color-muted);
}

.topbar__rail-current {
  font-size: var(--text-sm);
  color: var(--color-on-dark);
}

.topbar__books {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  min-width: 0;
  overflow-x: auto;
  padding-bottom: 2px;
}

.topbar__book-item {
  flex-shrink: 0;
  background: transparent;
}

.topbar__book-item--active,
.topbar__sheet-book--active {
  background: var(--color-surface-card);
  border-color: var(--color-hairline);
  color: var(--color-on-dark);
}

.topbar__book-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.topbar__book-chevron {
  margin-left: auto;
  flex-shrink: 0;
}

.topbar__sheet {
  display: none;
}

.topbar__sheet-card {
  background: var(--color-canvas);
  border-top: 1px solid var(--color-hairline);
  padding: var(--space-lg);
  display: grid;
  gap: var(--space-lg);
}

.topbar__sheet-nav,
.topbar__sheet-books {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.topbar__sheet-link,
.topbar__sheet-book {
  justify-content: space-between;
  background: var(--color-surface-card);
  border-color: var(--color-hairline);
}

.topbar__sheet-signout {
  background: rgba(246, 70, 93, 0.08);
  border-color: rgba(246, 70, 93, 0.24);
  color: var(--color-danger);
  justify-content: center;
}

.sheet-enter-active,
.sheet-leave-active {
  transition: opacity var(--transition-base);
}

.sheet-enter-from,
.sheet-leave-to {
  opacity: 0;
}

@media (max-width: 1024px) {
  .topbar__user-copy {
    display: none;
  }
}

@media (max-width: 768px) {
  .topbar__main {
    min-height: 64px;
  }

  .topbar__nav,
  .topbar__user,
  .topbar__signout,
  .topbar__rail {
    display: none;
  }

  .topbar__actions {
    margin-left: auto;
  }

  .topbar__menu-toggle {
    display: inline-flex;
  }

  .topbar__sheet {
    display: block;
  }
}

@media (max-width: 640px) {
  .topbar__brand-sub {
    display: none;
  }

  .topbar__brand-name {
    font-size: var(--text-md);
  }
}
</style>
