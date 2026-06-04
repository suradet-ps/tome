<script setup lang="ts">
import { ArrowRight, BookOpen, Plus } from 'lucide-vue-next';
import { storeToRefs } from 'pinia';
import { computed, onMounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import BaseButton from '@/components/common/BaseButton.vue';
import BaseInput from '@/components/common/BaseInput.vue';
import BaseLoader from '@/components/common/BaseLoader.vue';
import BaseModal from '@/components/common/BaseModal.vue';
import ProgressBar from '@/components/progress/ProgressBar.vue';
import { assertSupabaseConfigured, supabase, supabaseConfigError } from '@/lib/supabase';
import { useAuthStore } from '@/stores/auth';
import { useBooksStore } from '@/stores/books';

interface ProgressSnapshot {
  completed: number;
  total: number;
  percent: number;
}

interface DashboardSummaryRow {
  book_id: string;
  total: number;
  completed: number;
  cards_due: number;
}

const booksStore = useBooksStore();
const auth = useAuthStore();
const router = useRouter();
const { books, loading } = storeToRefs(booksStore);

const showAddModal = ref(false);
const newTitle = ref('');
const newAuthor = ref('');
const adding = ref(false);
const addError = ref('');
const dashboardError = ref('');

watch(
  () => showAddModal.value,
  (open) => {
    if (!open) {
      addError.value = '';
    }
  },
);
const stats = ref({
  completedChapters: 0,
  cardsDue: 0,
});
const bookProgress = ref<Record<string, ProgressSnapshot>>({});

const configMessage = computed(() => supabaseConfigError);
const greeting = computed(() => auth.profile?.username ?? 'there');

function snapshotFor(bookId: string, fallbackTotal: number): ProgressSnapshot {
  return (
    bookProgress.value[bookId] ?? {
      completed: 0,
      total: fallbackTotal,
      percent: 0,
    }
  );
}

async function loadDashboard() {
  dashboardError.value = '';

  try {
    await booksStore.fetchBooks();

    if (!auth.user || supabaseConfigError) {
      stats.value = { completedChapters: 0, cardsDue: 0 };
      bookProgress.value = {};
      return;
    }

    assertSupabaseConfigured();

    const { data: summary, error: summaryError } = await supabase.rpc('get_dashboard_summary');
    if (summaryError) throw summaryError;

    const rows = (summary ?? []) as DashboardSummaryRow[];
    const nextProgress: Record<string, ProgressSnapshot> = {};
    let totalCompleted = 0;

    for (const book of books.value) {
      const row = rows.find((r) => r.book_id === book.id);
      const total = row?.total ?? book.total_chapters;
      const completed = row?.completed ?? 0;
      nextProgress[book.id] = {
        completed,
        total,
        percent: total === 0 ? 0 : Math.round((completed / total) * 100),
      };
      totalCompleted += completed;
    }

    const { data: cards, error: cardsError } = await supabase
      .from('reading_flashcards')
      .select('id')
      .eq('user_id', auth.user.id)
      .lte('next_review', new Date().toISOString())
      .range(0, 999);
    if (cardsError) throw cardsError;
    const cardsDue = (cards ?? []).length;

    stats.value = { completedChapters: totalCompleted, cardsDue };
    bookProgress.value = nextProgress;
  } catch (caughtError) {
    dashboardError.value =
      caughtError instanceof Error ? caughtError.message : 'Unable to load dashboard.';
  }
}

onMounted(() => {
  void loadDashboard();
});

async function handleAddBook() {
  addError.value = '';

  if (!newTitle.value.trim()) {
    addError.value = 'Title is required.';
    return;
  }

  adding.value = true;

  try {
    const addedBook = await booksStore.addBook(newTitle.value.trim(), newAuthor.value.trim());
    if (addedBook) {
      await loadDashboard();
    }

    newTitle.value = '';
    newAuthor.value = '';
    showAddModal.value = false;
  } catch (caughtError) {
    addError.value = caughtError instanceof Error ? caughtError.message : 'Unable to add book.';
  } finally {
    adding.value = false;
  }
}

function openBook(bookId: string) {
  void router.push(`/books/${bookId}`);
}
</script>

<template>
  <div class="page dashboard">
    <header class="page-header">
      <div>
        <h1 class="page-header__title">Hi, {{ greeting }}</h1>
        <p class="page-header__sub">Track what you read, one chapter at a time.</p>
      </div>
      <div class="page-header__actions">
        <BaseButton @click="showAddModal = true">
          <Plus :size="16" />
          Add book
        </BaseButton>
      </div>
    </header>

    <p v-if="configMessage" class="notice">{{ configMessage }}</p>
    <p v-else-if="dashboardError" class="notice">{{ dashboardError }}</p>

    <section class="stats">
      <div class="stats__item">
        <span class="stats__label">Books</span>
        <span class="stats__value numeric">{{ books.length }}</span>
      </div>
      <div class="stats__divider" aria-hidden="true"></div>
      <div class="stats__item">
        <span class="stats__label">Chapters done</span>
        <span class="stats__value numeric">{{ stats.completedChapters }}</span>
      </div>
      <div class="stats__divider" aria-hidden="true"></div>
      <div class="stats__item">
        <span class="stats__label">Cards due</span>
        <span class="stats__value numeric">{{ stats.cardsDue }}</span>
      </div>
    </section>

    <section v-if="loading" class="dashboard__loading">
      <BaseLoader :size="28" />
    </section>

    <section v-else-if="books.length === 0" class="empty">
      <BookOpen :size="32" class="empty__icon" />
      <h3 class="empty__title">No books yet</h3>
      <p class="empty__copy">Add your first book to start tracking chapters and notes.</p>
      <BaseButton @click="showAddModal = true">
        <Plus :size="16" />
        Add book
      </BaseButton>
    </section>

    <section v-else class="book-grid">
      <button
        v-for="book in books"
        :key="book.id"
        type="button"
        class="book-card"
        @click="openBook(book.id)"
      >
        <div class="book-card__head">
          <h3 class="book-card__title">{{ book.title }}</h3>
          <ArrowRight :size="16" class="book-card__arrow" />
        </div>
        <p class="book-card__author">{{ book.author || 'Unknown author' }}</p>

        <div class="book-card__progress">
          <ProgressBar
            :completed="snapshotFor(book.id, book.total_chapters).completed"
            :total="snapshotFor(book.id, book.total_chapters).total"
          />
        </div>

        <div class="book-card__meta">
          <span class="numeric">{{ snapshotFor(book.id, book.total_chapters).completed }} / {{ snapshotFor(book.id, book.total_chapters).total }}</span>
          <span>chapters</span>
        </div>
      </button>
    </section>

    <BaseModal v-model="showAddModal" title="Add book">
      <form class="dashboard__form" aria-label="Add book" @submit.prevent="handleAddBook">
        <BaseInput v-model="newTitle" label="Title *" placeholder="e.g. Atomic Habits" />
        <BaseInput v-model="newAuthor" label="Author" placeholder="e.g. James Clear" />
        <p v-if="addError" class="dashboard__form-error">{{ addError }}</p>
        <div class="form-actions">
          <BaseButton variant="secondary" type="button" @click="showAddModal = false">Cancel</BaseButton>
          <BaseButton type="submit" :loading="adding">Add</BaseButton>
        </div>
      </form>
    </BaseModal>
  </div>
</template>

<style scoped>
.stats {
  display: flex;
  align-items: stretch;
  gap: var(--space-md);
  padding: var(--space-md) var(--space-lg);
  background: var(--color-surface-card);
  border: 1px solid var(--color-hairline);
  border-radius: var(--radius-xl);
}

.stats__item {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.stats__label {
  font-size: var(--text-xs);
  color: var(--color-muted);
  letter-spacing: 0.04em;
}

.stats__value {
  font-size: var(--text-xl);
  font-weight: var(--weight-bold);
  color: var(--color-on-dark);
  line-height: 1.1;
}

.stats__divider {
  width: 1px;
  background: var(--color-hairline);
  flex-shrink: 0;
}

.dashboard__loading {
  display: flex;
  justify-content: center;
  padding: var(--space-xxl) 0;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-xxl) var(--space-lg);
  text-align: center;
  background: var(--color-surface-card);
  border: 1px dashed var(--color-hairline);
  border-radius: var(--radius-xl);
}

.empty__icon {
  color: var(--color-muted);
  margin-bottom: var(--space-xs);
}

.empty__title {
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
}

.empty__copy {
  max-width: 360px;
  color: var(--color-muted);
  font-size: var(--text-sm);
  margin-bottom: var(--space-xs);
}

.book-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--space-md);
}

.book-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  padding: var(--space-lg);
  background: var(--color-surface-card);
  border: 1px solid var(--color-hairline);
  border-radius: var(--radius-xl);
  text-align: left;
  cursor: pointer;
  transition: border-color var(--transition-fast), background var(--transition-fast), transform var(--transition-fast);
}

.book-card:hover {
  border-color: rgba(252, 213, 53, 0.32);
  background: var(--color-surface-elevated);
}

.book-card:hover .book-card__arrow {
  color: var(--color-primary);
  transform: translateX(2px);
}

.book-card__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-sm);
}

.book-card__title {
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
  line-height: 1.3;
  letter-spacing: -0.01em;
}

.book-card__arrow {
  color: var(--color-muted);
  flex-shrink: 0;
  transition: color var(--transition-fast), transform var(--transition-fast);
}

.book-card__author {
  font-size: var(--text-sm);
  color: var(--color-muted);
}

.book-card__progress {
  margin-top: var(--space-sm);
}

.book-card__meta {
  display: flex;
  align-items: baseline;
  gap: 4px;
  font-size: var(--text-xs);
  color: var(--color-muted);
}

.book-card__meta .numeric {
  color: var(--color-body);
  font-weight: var(--weight-semibold);
}

.dashboard__form {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.dashboard__form-error {
  font-size: var(--text-sm);
  color: var(--color-danger);
  background: rgba(246, 70, 93, 0.08);
  border: 1px solid rgba(246, 70, 93, 0.24);
  border-radius: var(--radius-md);
  padding: var(--space-xs) var(--space-sm);
}

@media (max-width: 640px) {
  .stats {
    padding: var(--space-md);
    flex-wrap: wrap;
  }

  .stats__divider {
    display: none;
  }

  .stats__item {
    flex: 1 1 30%;
  }

  .book-grid {
    grid-template-columns: 1fr;
  }
}
</style>
