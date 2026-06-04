<script setup lang="ts">
import { ArrowLeft, Pause, Play, Plus, RotateCcw, Save } from 'lucide-vue-next';
import { storeToRefs } from 'pinia';
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { useRoute } from 'vue-router';
import BaseButton from '@/components/common/BaseButton.vue';
import BaseInput from '@/components/common/BaseInput.vue';
import BaseLoader from '@/components/common/BaseLoader.vue';
import BaseModal from '@/components/common/BaseModal.vue';
import MarkdownEditor from '@/components/editor/MarkdownEditor.vue';
import ChapterList from '@/components/progress/ChapterList.vue';
import ProgressBar from '@/components/progress/ProgressBar.vue';
import { useTimer } from '@/composables/useTimer';
import { useBooksStore } from '@/stores/books';
import { useNotesStore } from '@/stores/notes';
import { useProgressStore } from '@/stores/progress';
import type { Chapter, ReadingStatus } from '@/types';

const route = useRoute();
const booksStore = useBooksStore();
const progressStore = useProgressStore();
const notesStore = useNotesStore();
const { chapters, loading, currentBook } = storeToRefs(booksStore);
const bookId = computed(() => {
  const id = route.params.id;
  return Array.isArray(id) ? id[0] : id;
});
const selectedChapter = ref<Chapter | null>(null);
const noteContent = ref('');
const loadedNoteContent = ref('');
const noteDirty = ref(false);
const savingNote = ref(false);
const showAddChapterModal = ref(false);
const newChapterTitle = ref('');
const newChapterSeq = ref('');
const newChapterParentId = ref('');
const addingChapter = ref(false);
const addChapterError = ref('');
const viewError = ref('');

const {
  seconds: timerSeconds,
  isRunning: timerIsRunning,
  start: timerStart,
  pause: timerPause,
  reset: timerReset,
  formatTime: formatTimer,
} = useTimer();

const statusOptions: { value: ReadingStatus; label: string }[] = [
  { value: 'not_started', label: 'Not started' },
  { value: 'in_progress', label: 'Reading' },
  { value: 'completed', label: 'Done' },
  { value: 'review_needed', label: 'Review' },
];

function nextStatus(current: ReadingStatus): ReadingStatus {
  const idx = statusOptions.findIndex((opt) => opt.value === current);
  return statusOptions[(idx + 1) % statusOptions.length].value;
}

function prevStatus(current: ReadingStatus): ReadingStatus {
  const idx = statusOptions.findIndex((opt) => opt.value === current);
  return statusOptions[(idx - 1 + statusOptions.length) % statusOptions.length].value;
}

const flatChapters = computed(() => booksStore.flattenChapters());

const currentStatus = computed(() => {
  if (!selectedChapter.value) return undefined;
  return progressStore.getProgress(selectedChapter.value.id)?.status;
});

const selectedProgress = computed(() => {
  if (!selectedChapter.value) return undefined;
  return progressStore.getProgress(selectedChapter.value.id);
});

const totalChapters = computed(() => flatChapters.value.length);
const completedChapters = computed(
  () =>
    flatChapters.value.filter(
      (chapter) => progressStore.getProgress(chapter.id)?.status === 'completed',
    ).length,
);

const chapterTimeLabel = computed(() => {
  const seconds = selectedProgress.value?.time_spent_seconds ?? 0;
  if (seconds === 0) return '0m';
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (hours === 0) return `${minutes}m`;
  return `${hours}h ${minutes}m`;
});

watch(noteContent, (current) => {
  if (current !== loadedNoteContent.value) {
    noteDirty.value = true;
  }
});

async function flushTimerForChapter(chapterId: string) {
  if (timerSeconds.value === 0) return;
  await progressStore.logTimeSpent(chapterId, timerSeconds.value);
  timerReset();
}

async function loadBook() {
  if (!bookId.value) return;
  viewError.value = '';

  if (selectedChapter.value && noteDirty.value) {
    const ok = window.confirm('You have unsaved notes. Discard them?');
    if (!ok) return;
  }
  if (selectedChapter.value) {
    await flushTimerForChapter(selectedChapter.value.id);
  }

  try {
    const [book] = await Promise.all([
      booksStore.fetchBook(bookId.value),
      booksStore.fetchChapters(bookId.value),
      progressStore.fetchProgressForBook(bookId.value),
    ]);

    if (book) {
      booksStore.books = [book, ...booksStore.books.filter((b) => b.id !== book.id)];
    }
    booksStore.setCurrentBook(bookId.value);

    const available = booksStore.flattenChapters();
    const next =
      available.find((chapter) => chapter.id === selectedChapter.value?.id) ?? available[0] ?? null;

    if (next) {
      await selectChapter(next);
    } else {
      selectedChapter.value = null;
      loadedNoteContent.value = '';
      noteContent.value = '';
      noteDirty.value = false;
    }
  } catch (caughtError) {
    viewError.value =
      caughtError instanceof Error ? caughtError.message : 'Unable to load this book.';
  }
}

watch(
  () => bookId.value,
  () => void loadBook(),
  { immediate: true },
);

async function selectChapter(chapter: Chapter) {
  if (selectedChapter.value && noteDirty.value && selectedChapter.value.id !== chapter.id) {
    const ok = window.confirm('You have unsaved notes. Discard them?');
    if (!ok) return;
  }
  if (selectedChapter.value && selectedChapter.value.id !== chapter.id) {
    await flushTimerForChapter(selectedChapter.value.id);
  }
  selectedChapter.value = chapter;
  noteDirty.value = false;
  const note = await notesStore.fetchNote(chapter.id);
  loadedNoteContent.value = note?.content ?? '';
  noteContent.value = loadedNoteContent.value;
  timerReset();
}

watch(
  () => showAddChapterModal.value,
  (open) => {
    addChapterError.value = '';
    if (open) {
      addingChapter.value = false;
      newChapterTitle.value = '';
      newChapterSeq.value = '';
      newChapterParentId.value = '';
    }
  },
);

async function saveNote() {
  if (!selectedChapter.value) return;
  savingNote.value = true;
  try {
    await notesStore.saveNote(selectedChapter.value.id, noteContent.value);
    loadedNoteContent.value = noteContent.value;
    noteDirty.value = false;
  } finally {
    savingNote.value = false;
  }
}

async function updateStatus(status: ReadingStatus) {
  if (!selectedChapter.value) return;
  await progressStore.updateStatus(selectedChapter.value.id, status);
}

async function handleAddChapter() {
  if (addingChapter.value) return;

  addChapterError.value = '';

  const title = newChapterTitle.value.trim();
  const seq = Number.parseFloat(newChapterSeq.value);

  if (!title) {
    addChapterError.value = 'Title is required.';
    return;
  }

  if (!newChapterSeq.value || Number.isNaN(seq)) {
    addChapterError.value = 'Sequence number is required (e.g. 1, 1.1, 2).';
    return;
  }

  if (!bookId.value) {
    addChapterError.value = 'No book selected.';
    return;
  }

  addingChapter.value = true;

  try {
    await booksStore.addChapter(bookId.value, title, seq, newChapterParentId.value || undefined);

    newChapterTitle.value = '';
    newChapterSeq.value = '';
    newChapterParentId.value = '';
    showAddChapterModal.value = false;

    await booksStore.fetchChapters(bookId.value);
  } catch (caughtError) {
    addChapterError.value =
      caughtError instanceof Error ? caughtError.message : 'Unable to add chapter.';
  } finally {
    addingChapter.value = false;
  }
}

async function logSession() {
  if (!selectedChapter.value || timerSeconds.value === 0) return;
  await flushTimerForChapter(selectedChapter.value.id);
}

onBeforeUnmount(() => {
  if (selectedChapter.value && timerSeconds.value > 0) {
    void progressStore.logTimeSpent(selectedChapter.value.id, timerSeconds.value);
  }
});
</script>

<template>
  <div class="page book">
    <header class="book__header">
      <RouterLink to="/" class="book__back">
        <ArrowLeft :size="14" />
        Library
      </RouterLink>

      <div class="book__title-row">
        <div class="book__title-block">
          <h1 v-if="currentBook" class="book__title">{{ currentBook.title }}</h1>
          <h1 v-else class="book__title">Book</h1>
          <p v-if="currentBook?.author" class="book__author">{{ currentBook.author }}</p>
        </div>
        <BaseButton variant="secondary" size="sm" @click="showAddChapterModal = true">
          <Plus :size="14" />
          Add chapter
        </BaseButton>
      </div>

      <div class="book__progress">
        <ProgressBar :completed="completedChapters" :total="totalChapters" />
        <span class="book__progress-label numeric">{{ completedChapters }} / {{ totalChapters }}</span>
      </div>
    </header>

    <p v-if="viewError" class="notice">{{ viewError }}</p>

    <div class="book__layout">
      <aside class="book__sidebar surface">
        <div v-if="loading" class="book__loading">
          <BaseLoader />
        </div>
        <div v-else-if="chapters.length === 0" class="book__empty">
          <p>No chapters yet.</p>
          <BaseButton size="sm" variant="secondary" @click="showAddChapterModal = true">
            <Plus :size="14" />
            Add chapter
          </BaseButton>
        </div>
        <ChapterList
          v-else
          :chapters="chapters"
          :selected-id="selectedChapter?.id ?? null"
          @select="selectChapter"
        />
      </aside>

      <div class="book__workspace">
        <template v-if="selectedChapter">
          <div class="chapter-bar surface">
            <div class="chapter-bar__head">
              <h2 class="chapter-bar__title">
                <span class="chapter-bar__seq numeric">{{ selectedChapter.sequence_number }}</span>
                {{ selectedChapter.title }}
              </h2>
              <span class="chapter-bar__time numeric">{{ chapterTimeLabel }} logged</span>
            </div>

            <div class="chapter-bar__row">
              <div class="chapter-bar__pills" role="radiogroup" aria-label="Status">
                <button
                  v-for="opt in statusOptions"
                  :key="opt.value"
                  type="button"
                  role="radio"
                  class="status-pill"
                  :class="[
                    `status-pill--${opt.value}`,
                    { 'status-pill--active': currentStatus === opt.value },
                  ]"
                  :aria-checked="currentStatus === opt.value"
                  :tabindex="currentStatus === opt.value || (!currentStatus && opt.value === 'not_started') ? 0 : -1"
                  @click="updateStatus(opt.value)"
                  @keydown.left.prevent="updateStatus(prevStatus(opt.value))"
                  @keydown.right.prevent="updateStatus(nextStatus(opt.value))"
                >
                  {{ opt.label }}
                </button>
              </div>

              <div class="chapter-bar__timer" role="group" aria-label="Session timer">
                <span
                  class="chapter-bar__clock numeric"
                  role="timer"
                  :aria-label="`Elapsed: ${formatTimer(timerSeconds)}`"
                >{{ formatTimer(timerSeconds) }}</span>
                <button
                  class="timer-btn"
                  type="button"
                  @click="timerReset"
                  title="Reset"
                  aria-label="Reset timer"
                >
                  <RotateCcw :size="14" />
                </button>
                <button
                  class="timer-btn timer-btn--primary"
                  type="button"
                  :title="timerIsRunning ? 'Pause' : 'Start'"
                  :aria-label="timerIsRunning ? 'Pause timer' : 'Start timer'"
                  @click="timerIsRunning ? timerPause() : timerStart()"
                >
                  <Play v-if="!timerIsRunning" :size="14" />
                  <Pause v-else :size="14" />
                </button>
                <button
                  class="timer-btn"
                  type="button"
                  :disabled="timerSeconds === 0"
                  title="Log session"
                  aria-label="Log session"
                  @click="logSession"
                >
                  <Save :size="14" />
                </button>
              </div>
            </div>
          </div>

          <MarkdownEditor v-model="noteContent" :saving="savingNote" @save="saveNote" class="book__editor" />
        </template>

        <div v-else class="book__no-chapter surface">
          <h3>Select a chapter</h3>
          <p>Pick a chapter from the sidebar to start writing notes.</p>
        </div>
      </div>
    </div>

    <BaseModal v-model="showAddChapterModal" title="Add chapter">
      <form class="book__form" @submit.prevent="handleAddChapter">
        <BaseInput v-model="newChapterTitle" label="Title *" placeholder="Getting started" />
        <BaseInput
          v-model="newChapterSeq"
          label="Sequence number *"
          type="text"
          inputmode="decimal"
          placeholder="e.g. 1 or 1.1"
        />

        <div class="book__select-group">
          <label class="book__select-label" for="parent-chapter">Parent chapter</label>
          <select id="parent-chapter" v-model="newChapterParentId" class="book__select">
            <option value="">None (top level)</option>
            <option v-for="chapter in flatChapters" :key="chapter.id" :value="chapter.id">
              {{ chapter.sequence_number }} · {{ chapter.title }}
            </option>
          </select>
        </div>

        <p v-if="addChapterError" class="book__form-error">{{ addChapterError }}</p>

        <div class="form-actions">
          <BaseButton variant="secondary" type="button" @click="showAddChapterModal = false">Cancel</BaseButton>
          <BaseButton type="submit" :loading="addingChapter">Add chapter</BaseButton>
        </div>
      </form>
    </BaseModal>
  </div>
</template>

<style scoped>
.book__header {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.book__back {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: var(--text-xs);
  color: var(--color-muted);
  width: fit-content;
}

.book__back:hover {
  color: var(--color-on-dark);
}

.book__back:focus-visible {
  outline: 2px solid var(--color-info);
  outline-offset: 2px;
  border-radius: var(--radius-sm);
}

.book__title-row {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: var(--space-md);
  flex-wrap: wrap;
}

.book__title {
  font-family: var(--font-display);
  font-size: var(--text-2xl);
  font-weight: var(--weight-bold);
  color: var(--color-on-dark);
  letter-spacing: -0.02em;
  line-height: 1.2;
}

.book__author {
  margin-top: 2px;
  font-size: var(--text-sm);
  color: var(--color-muted);
}

.book__progress {
  display: flex;
  align-items: center;
  gap: var(--space-md);
  max-width: 480px;
}

.book__progress-label {
  font-size: var(--text-xs);
  color: var(--color-muted);
  white-space: nowrap;
}

.book__layout {
  display: grid;
  grid-template-columns: minmax(240px, 280px) minmax(0, 1fr);
  gap: var(--space-md);
  align-items: start;
}

.book__sidebar {
  position: sticky;
  top: 72px;
  padding: var(--space-sm);
  max-height: calc(100vh - 96px);
  overflow-y: auto;
}

.book__loading,
.book__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-xl) var(--space-md);
  color: var(--color-muted);
  font-size: var(--text-sm);
}

.book__workspace {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
  min-width: 0;
}

.chapter-bar {
  padding: var(--space-md) var(--space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.chapter-bar__head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-md);
  flex-wrap: wrap;
}

.chapter-bar__title {
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
  line-height: 1.3;
  display: flex;
  align-items: baseline;
  gap: var(--space-xs);
  flex-wrap: wrap;
}

.chapter-bar__seq {
  font-size: var(--text-sm);
  color: var(--color-muted);
  font-weight: var(--weight-medium);
}

.chapter-bar__time {
  font-size: var(--text-xs);
  color: var(--color-muted);
  white-space: nowrap;
}

.chapter-bar__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-md);
  flex-wrap: wrap;
}

.chapter-bar__pills {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.status-pill {
  height: 28px;
  padding: 0 var(--space-sm);
  border-radius: var(--radius-pill);
  border: 1px solid var(--color-hairline);
  background: transparent;
  color: var(--color-muted);
  font-size: var(--text-xs);
  font-weight: var(--weight-medium);
  transition: all var(--transition-fast);
}

.status-pill:hover {
  color: var(--color-on-dark);
  border-color: var(--color-muted);
}

.status-pill--active.status-pill--not_started {
  background: rgba(112, 122, 138, 0.15);
  color: var(--color-on-dark);
  border-color: var(--color-muted);
}

.status-pill--active.status-pill--in_progress {
  background: rgba(59, 130, 246, 0.12);
  color: var(--color-info);
  border-color: rgba(59, 130, 246, 0.4);
}

.status-pill--active.status-pill--completed {
  background: rgba(14, 203, 129, 0.12);
  color: var(--color-success);
  border-color: rgba(14, 203, 129, 0.4);
}

.status-pill--active.status-pill--review_needed {
  background: rgba(240, 185, 11, 0.12);
  color: var(--color-warning);
  border-color: rgba(240, 185, 11, 0.4);
}

.chapter-bar__timer {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  padding: 4px 4px 4px var(--space-sm);
  background: var(--color-canvas);
  border: 1px solid var(--color-hairline);
  border-radius: var(--radius-pill);
}

.chapter-bar__clock {
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
  min-width: 56px;
  text-align: center;
}

.timer-btn {
  width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-full);
  color: var(--color-muted);
  transition: all var(--transition-fast);
}

.timer-btn:hover:not(:disabled) {
  color: var(--color-on-dark);
  background: var(--color-surface-elevated);
}

.timer-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.timer-btn--primary {
  background: var(--color-primary);
  color: var(--color-on-primary);
}

.timer-btn--primary:hover:not(:disabled) {
  background: var(--color-primary-active);
  color: var(--color-on-primary);
}

.book__editor {
  min-height: 520px;
}

.book__no-chapter {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-xs);
  min-height: 360px;
  padding: var(--space-xl);
  text-align: center;
}

.book__no-chapter h3 {
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
}

.book__no-chapter p {
  color: var(--color-muted);
  font-size: var(--text-sm);
}

.book__form {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.book__select-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.book__select-label {
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  color: var(--color-muted);
}

.book__select {
  width: 100%;
  min-height: 40px;
  background: var(--color-surface-card);
  border: 1px solid var(--color-hairline);
  border-radius: var(--radius-md);
  padding: 10px var(--space-md);
  color: var(--color-on-dark);
  font-size: var(--text-base);
}

.book__select:focus {
  outline: none;
  border-color: rgba(59, 130, 246, 0.55);
  box-shadow: var(--shadow-focus);
}

.book__form-error {
  font-size: var(--text-sm);
  color: var(--color-danger);
  background: rgba(246, 70, 93, 0.08);
  border: 1px solid rgba(246, 70, 93, 0.24);
  border-radius: var(--radius-md);
  padding: var(--space-xs) var(--space-sm);
}

@media (max-width: 960px) {
  .book__layout {
    grid-template-columns: 1fr;
  }

  .book__sidebar {
    position: static;
    max-height: 320px;
  }
}

@media (max-width: 640px) {
  .chapter-bar__row {
    flex-direction: column;
    align-items: stretch;
  }

  .chapter-bar__timer {
    justify-content: space-between;
  }
}
</style>
