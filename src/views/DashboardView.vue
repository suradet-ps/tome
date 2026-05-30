<script setup lang="ts">
import { ArrowRight, BookOpen, Brain, Clock3, Plus, Sparkles, TrendingUp } from 'lucide-vue-next'
import { storeToRefs } from 'pinia'
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import BaseButton from '@/components/common/BaseButton.vue'
import BaseInput from '@/components/common/BaseInput.vue'
import BaseLoader from '@/components/common/BaseLoader.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import ProgressBar from '@/components/progress/ProgressBar.vue'
import { assertSupabaseConfigured, supabase, supabaseConfigError } from '@/lib/supabase'
import { useAuthStore } from '@/stores/auth'
import { useBooksStore } from '@/stores/books'
import type { Flashcard, Progress } from '@/types'

interface ChapterRef {
  id: string
  book_id: string
}

interface ProgressSnapshot {
  completed: number
  total: number
  percent: number
}

const booksStore = useBooksStore()
const auth = useAuthStore()
const router = useRouter()
const { books, loading } = storeToRefs(booksStore)

const showAddModal = ref(false)
const newTitle = ref('')
const newAuthor = ref('')
const adding = ref(false)
const dashboardError = ref('')
const stats = ref({
  completedChapters: 0,
  timeSpentSeconds: 0,
  cardsDue: 0,
})
const bookProgress = ref<Record<string, ProgressSnapshot>>({})

const configMessage = computed(() => supabaseConfigError)
const greeting = computed(() => auth.profile?.username ?? 'Reader')

function formatHours(totalSeconds: number) {
  if (totalSeconds <= 0) return '0h'
  const hours = totalSeconds / 3600
  return hours >= 10 ? `${Math.round(hours)}h` : `${hours.toFixed(1)}h`
}

function getBookSnapshot(bookId: string, fallbackTotal: number): ProgressSnapshot {
  return (
    bookProgress.value[bookId] ?? {
      completed: 0,
      total: fallbackTotal,
      percent: 0,
    }
  )
}

async function loadDashboard() {
  dashboardError.value = ''

  try {
    await booksStore.fetchBooks()

    if (!auth.user || supabaseConfigError) {
      stats.value = { completedChapters: 0, timeSpentSeconds: 0, cardsDue: 0 }
      bookProgress.value = {}
      return
    }

    assertSupabaseConfigured()

    const [progressResponse, cardsResponse] = await Promise.all([
      supabase.from('reading_progress').select('chapter_id, status, time_spent_seconds').eq('user_id', auth.user.id),
      supabase
        .from('reading_flashcards')
        .select('id, next_review')
        .eq('user_id', auth.user.id)
        .lte('next_review', new Date().toISOString()),
    ])

    if (progressResponse.error) throw progressResponse.error
    if (cardsResponse.error) throw cardsResponse.error

    const progressRows = (progressResponse.data ?? []) as Pick<Progress, 'chapter_id' | 'status' | 'time_spent_seconds'>[]
    const dueCards = (cardsResponse.data ?? []) as Pick<Flashcard, 'id' | 'next_review'>[]

    let chapterRows: ChapterRef[] = []
    if (books.value.length > 0) {
      const { data, error } = await supabase
        .from('reading_chapters')
        .select('id, book_id')
        .in('book_id', books.value.map((book) => book.id))

      if (error) throw error
      chapterRows = (data ?? []) as ChapterRef[]
    }

    const chapterToBook = new Map(chapterRows.map((chapter) => [chapter.id, chapter.book_id]))
    const nextProgress: Record<string, ProgressSnapshot> = {}

    books.value.forEach((book) => {
      nextProgress[book.id] = {
        completed: 0,
        total: chapterRows.filter((chapter) => chapter.book_id === book.id).length || book.total_chapters,
        percent: 0,
      }
    })

    progressRows.forEach((row) => {
      const bookId = chapterToBook.get(row.chapter_id)
      if (!bookId || row.status !== 'completed') return
      nextProgress[bookId].completed += 1
    })

    Object.values(nextProgress).forEach((snapshot) => {
      snapshot.percent = snapshot.total === 0 ? 0 : Math.round((snapshot.completed / snapshot.total) * 100)
    })

    stats.value = {
      completedChapters: progressRows.filter((row) => row.status === 'completed').length,
      timeSpentSeconds: progressRows.reduce((total, row) => total + row.time_spent_seconds, 0),
      cardsDue: dueCards.length,
    }
    bookProgress.value = nextProgress
  } catch (caughtError) {
    dashboardError.value = caughtError instanceof Error ? caughtError.message : 'Unable to load dashboard.'
  }
}

onMounted(() => {
  void loadDashboard()
})

async function handleAddBook() {
  if (!newTitle.value.trim()) return

  adding.value = true

  try {
    const addedBook = await booksStore.addBook(newTitle.value.trim(), newAuthor.value.trim())
    if (addedBook) {
      await loadDashboard()
    }

    newTitle.value = ''
    newAuthor.value = ''
    showAddModal.value = false
  } finally {
    adding.value = false
  }
}

function openBook(bookId: string) {
  void router.push(`/books/${bookId}`)
}

function openReview() {
  void router.push('/review')
}
</script>

<template>
  <div class="dashboard page-shell">
    <section class="dashboard__hero page-hero">
      <div class="dashboard__hero-copy">
        <p class="eyebrow">Reading command center</p>
        <h1 class="page-title">Build compounding technical depth.</h1>
        <p class="page-subtitle">
          Welcome back, {{ greeting }}. Keep book structure, notes, and recall sessions in one deliberate workflow.
        </p>

        <div class="dashboard__hero-actions">
          <BaseButton @click="showAddModal = true">
            <Plus :size="16" />
            Add book
          </BaseButton>
          <BaseButton variant="secondary" @click="openReview">
            <Brain :size="16" />
            Review due cards
          </BaseButton>
        </div>
      </div>

      <div class="dashboard__hero-panel surface-panel surface-panel--soft">
        <div class="dashboard__hero-panel-row">
          <span class="eyebrow">Today</span>
          <span class="dashboard__hero-panel-value numeric">{{ stats.cardsDue }}</span>
        </div>
        <p class="dashboard__hero-panel-title">
          {{ stats.cardsDue > 0 ? 'Flashcards are waiting for review.' : 'No flashcards are due right now.' }}
        </p>
        <p class="dashboard__hero-panel-copy">
          Keep sessions short, capture dense chapter notes, and review concepts before they fade.
        </p>
      </div>
    </section>

    <p v-if="configMessage" class="notice">{{ configMessage }}</p>
    <p v-else-if="dashboardError" class="notice">{{ dashboardError }}</p>

    <div class="stat-grid">
      <div class="stat-tile">
        <BookOpen :size="18" class="dashboard__stat-icon" />
        <p class="stat-tile__label">Books</p>
        <p class="stat-tile__value">{{ books.length }}</p>
        <p class="stat-tile__meta">Active technical titles in your library.</p>
      </div>
      <div class="stat-tile">
        <TrendingUp :size="18" class="dashboard__stat-icon dashboard__stat-icon--success" />
        <p class="stat-tile__label">Completed chapters</p>
        <p class="stat-tile__value">{{ stats.completedChapters }}</p>
        <p class="stat-tile__meta">Across every tracked book.</p>
      </div>
      <div class="stat-tile">
        <Clock3 :size="18" class="dashboard__stat-icon dashboard__stat-icon--info" />
        <p class="stat-tile__label">Focused time</p>
        <p class="stat-tile__value">{{ formatHours(stats.timeSpentSeconds) }}</p>
        <p class="stat-tile__meta">Logged deliberate reading time.</p>
      </div>
      <div class="stat-tile">
        <Brain :size="18" class="dashboard__stat-icon dashboard__stat-icon--warning" />
        <p class="stat-tile__label">Due now</p>
        <p class="stat-tile__value">{{ stats.cardsDue }}</p>
        <p class="stat-tile__meta">Flashcards ready for active recall.</p>
      </div>
    </div>

    <section class="dashboard__board">
      <div class="dashboard__library surface-panel">
        <div class="dashboard__section-head">
          <div>
            <p class="eyebrow">Library</p>
            <h2 class="dashboard__section-title">Production-ready reading board</h2>
          </div>
          <span class="dashboard__section-meta">{{ books.length }} active {{ books.length === 1 ? 'book' : 'books' }}</span>
        </div>

        <div v-if="loading" class="dashboard__loading">
          <BaseLoader :size="32" />
        </div>

        <div v-else-if="books.length === 0" class="dashboard__empty">
          <BookOpen :size="48" class="dashboard__empty-icon" />
          <h3 class="dashboard__empty-title">Start with your first technical book.</h3>
          <p class="dashboard__empty-copy">Add a title, define chapters, and turn dense material into a durable reading system.</p>
          <BaseButton @click="showAddModal = true">
            <Plus :size="16" />
            Add book
          </BaseButton>
        </div>

        <div v-else class="dashboard__rows">
          <button
            v-for="book in books"
            :key="book.id"
            type="button"
            class="dashboard__row"
            @click="openBook(book.id)"
          >
            <div class="dashboard__row-main">
              <div class="dashboard__row-icon">
                <BookOpen :size="18" />
              </div>
              <div class="dashboard__row-copy">
                <div class="dashboard__row-title-line">
                  <h3 class="dashboard__row-title">{{ book.title }}</h3>
                  <span class="dashboard__row-author">{{ book.author ?? 'Independent track' }}</span>
                </div>
                <div class="dashboard__row-meta">
                  <span>{{ getBookSnapshot(book.id, book.total_chapters).completed }}/{{ getBookSnapshot(book.id, book.total_chapters).total }} completed</span>
                  <span>{{ book.total_chapters }} planned chapters</span>
                </div>
              </div>
            </div>

            <div class="dashboard__row-progress">
              <ProgressBar
                :completed="getBookSnapshot(book.id, book.total_chapters).completed"
                :total="getBookSnapshot(book.id, book.total_chapters).total"
              />
              <span class="dashboard__row-percent numeric">{{ getBookSnapshot(book.id, book.total_chapters).percent }}%</span>
              <ArrowRight :size="16" class="dashboard__row-chevron" />
            </div>
          </button>
        </div>
      </div>

      <aside class="dashboard__aside">
        <div class="dashboard__cta surface-panel">
          <p class="eyebrow">Next move</p>
          <h3 class="dashboard__cta-title">
            {{ stats.cardsDue > 0 ? 'Clear your review queue before adding more input.' : 'Capture another book while recall is quiet.' }}
          </h3>
          <p class="dashboard__cta-copy">
            Production reading systems work best when tracking, note-taking, and spaced repetition stay tightly connected.
          </p>
          <BaseButton variant="secondary" @click="stats.cardsDue > 0 ? openReview() : (showAddModal = true)">
            <Sparkles :size="16" />
            {{ stats.cardsDue > 0 ? 'Open review board' : 'Add another title' }}
          </BaseButton>
        </div>

        <div class="dashboard__principles surface-panel">
          <p class="eyebrow">System cues</p>
          <ul class="dashboard__principles-list">
            <li>Track chapter progress like a structured operating board, not a loose checklist.</li>
            <li>Use markdown notes to compress key insights, code samples, and recall prompts.</li>
            <li>Review flashcards while concepts are fresh enough to strengthen retention.</li>
          </ul>
        </div>
      </aside>
    </section>

    <BaseModal v-model="showAddModal" title="Add New Book">
      <form class="dashboard__modal-form" @submit.prevent="handleAddBook">
        <BaseInput v-model="newTitle" label="Title *" placeholder="The Rust Programming Language" />
        <BaseInput v-model="newAuthor" label="Author" placeholder="Steve Klabnik, Carol Nichols" />
        <div class="form-actions">
          <BaseButton variant="secondary" type="button" @click="showAddModal = false">Cancel</BaseButton>
          <BaseButton type="submit" :loading="adding">Add Book</BaseButton>
        </div>
      </form>
    </BaseModal>
  </div>
</template>

<style scoped>
.dashboard {
  gap: var(--space-xl);
}

.dashboard__hero {
  grid-template-columns: minmax(0, 1.6fr) minmax(300px, 0.8fr);
  align-items: stretch;
}

.dashboard__hero-copy {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.dashboard__hero-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-sm);
}

.dashboard__hero-panel {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: var(--space-md);
  padding: var(--space-xl);
}

.dashboard__hero-panel-row {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: var(--space-md);
}

.dashboard__hero-panel-value {
  font-size: clamp(var(--text-2xl), 6vw, var(--text-4xl));
  font-weight: var(--weight-bold);
  color: var(--color-primary);
}

.dashboard__hero-panel-title {
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
}

.dashboard__hero-panel-copy {
  color: var(--color-muted);
  font-size: var(--text-sm);
}

.dashboard__stat-icon {
  color: var(--color-primary);
}

.dashboard__stat-icon--success {
  color: var(--color-success);
}

.dashboard__stat-icon--info {
  color: var(--color-info);
}

.dashboard__stat-icon--warning {
  color: var(--color-warning);
}

.dashboard__board {
  display: grid;
  grid-template-columns: minmax(0, 1.6fr) minmax(320px, 0.9fr);
  gap: var(--space-lg);
}

.dashboard__library,
.dashboard__cta,
.dashboard__principles {
  padding: var(--space-xl);
}

.dashboard__section-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-lg);
  margin-bottom: var(--space-lg);
}

.dashboard__section-title {
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
  letter-spacing: -0.02em;
}

.dashboard__section-meta {
  font-size: var(--text-sm);
  color: var(--color-muted);
}

.dashboard__loading {
  display: flex;
  justify-content: center;
  padding: var(--space-section);
}

.dashboard__empty {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: var(--space-md);
  padding: var(--space-section) 0 var(--space-md);
}

.dashboard__empty-icon {
  color: var(--color-primary);
}

.dashboard__empty-title {
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
}

.dashboard__empty-copy {
  max-width: 480px;
  color: var(--color-muted);
}

.dashboard__rows {
  display: flex;
  flex-direction: column;
}

.dashboard__row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(240px, 320px);
  align-items: center;
  gap: var(--space-lg);
  width: 100%;
  padding: var(--space-lg) 0;
  border-top: 1px solid rgba(43, 49, 57, 0.7);
  text-align: left;
}

.dashboard__row:first-child {
  border-top: 0;
  padding-top: 0;
}

.dashboard__row:last-child {
  padding-bottom: 0;
}

.dashboard__row-main {
  display: flex;
  align-items: center;
  gap: var(--space-md);
  min-width: 0;
}

.dashboard__row-icon {
  width: 44px;
  height: 44px;
  border-radius: var(--radius-lg);
  background: rgba(252, 213, 53, 0.1);
  color: var(--color-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.dashboard__row-copy {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  min-width: 0;
}

.dashboard__row-title-line {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: var(--space-sm);
}

.dashboard__row-title {
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
}

.dashboard__row-author {
  font-size: var(--text-sm);
  color: var(--color-muted);
}

.dashboard__row-meta {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-sm);
  color: var(--color-muted-strong);
  font-size: var(--text-sm);
}

.dashboard__row-progress {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  align-items: center;
  gap: var(--space-sm);
}

.dashboard__row-percent {
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
  min-width: 44px;
  text-align: right;
}

.dashboard__row-chevron {
  color: var(--color-muted);
}

.dashboard__aside {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.dashboard__cta,
.dashboard__principles {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.dashboard__cta-title {
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
}

.dashboard__cta-copy {
  color: var(--color-muted);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
}

.dashboard__principles-list {
  display: grid;
  gap: var(--space-md);
  list-style: disc;
  color: var(--color-muted-strong);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
  padding-left: var(--space-lg);
}

.dashboard__modal-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

@media (max-width: 1100px) {
  .dashboard__hero,
  .dashboard__board {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 900px) {
  .dashboard__row {
    grid-template-columns: 1fr;
  }

  .dashboard__row-progress {
    grid-template-columns: minmax(0, 1fr) auto;
  }

  .dashboard__row-chevron {
    display: none;
  }
}

@media (max-width: 640px) {
  .dashboard__hero-actions {
    flex-direction: column;
  }

  .dashboard__section-head {
    flex-direction: column;
  }
}
</style>
