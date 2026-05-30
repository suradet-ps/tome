<script setup lang="ts">
import { ArrowLeft, Pause, Play, Plus, RotateCcw, Save, Sparkles } from 'lucide-vue-next'
import { storeToRefs } from 'pinia'
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import BaseButton from '@/components/common/BaseButton.vue'
import BaseInput from '@/components/common/BaseInput.vue'
import BaseLoader from '@/components/common/BaseLoader.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import MarkdownEditor from '@/components/editor/MarkdownEditor.vue'
import ChapterList from '@/components/progress/ChapterList.vue'
import ProgressBar from '@/components/progress/ProgressBar.vue'
import { useTimer } from '@/composables/useTimer'
import { useBooksStore } from '@/stores/books'
import { useNotesStore } from '@/stores/notes'
import { useProgressStore } from '@/stores/progress'
import type { Chapter, ReadingStatus } from '@/types'

const route = useRoute()
const booksStore = useBooksStore()
const progressStore = useProgressStore()
const notesStore = useNotesStore()
const { chapters, loading, currentBook } = storeToRefs(booksStore)
const bookId = computed(() => route.params.id as string)
const selectedChapter = ref<Chapter | null>(null)
const noteContent = ref('')
const savingNote = ref(false)
const showAddChapterModal = ref(false)
const newChapterTitle = ref('')
const newChapterSeq = ref('')
const newChapterParentId = ref('')
const viewError = ref('')

const sessionTimer = useTimer()

const statusOptions: { value: ReadingStatus; label: string; color: string }[] = [
  { value: 'not_started', label: 'Not Started', color: 'var(--color-muted)' },
  { value: 'in_progress', label: 'In Progress', color: 'var(--color-info)' },
  { value: 'completed', label: 'Completed', color: 'var(--color-success)' },
  { value: 'review_needed', label: 'Review Needed', color: 'var(--color-warning)' },
]

const flatChapters = computed(() => booksStore.flattenChapters())

const currentStatus = computed(() => {
  if (!selectedChapter.value) return undefined
  return progressStore.getProgress(selectedChapter.value.id)?.status
})

const selectedProgress = computed(() => {
  if (!selectedChapter.value) return undefined
  return progressStore.getProgress(selectedChapter.value.id)
})

const totalChapters = computed(() => flatChapters.value.length)
const completedChapters = computed(
  () => flatChapters.value.filter((chapter) => progressStore.getProgress(chapter.id)?.status === 'completed').length,
)
const remainingChapters = computed(() => Math.max(totalChapters.value - completedChapters.value, 0))
const selectedStatusLabel = computed(
  () => statusOptions.find((option) => option.value === currentStatus.value)?.label ?? 'Not Started',
)

const currentChapterTimeLabel = computed(() => {
  const seconds = selectedProgress.value?.time_spent_seconds ?? 0
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)

  if (hours === 0 && minutes === 0) return '0m logged'
  if (hours === 0) return `${minutes}m logged`
  return `${hours}h ${minutes}m logged`
})

async function loadBook() {
  if (!bookId.value) return

  viewError.value = ''

  try {
    await Promise.all([
      booksStore.fetchBooks(),
      booksStore.fetchChapters(bookId.value),
      progressStore.fetchProgressForBook(bookId.value),
    ])

    booksStore.setCurrentBook(bookId.value)

    const availableChapters = booksStore.flattenChapters()
    const nextSelection = availableChapters.find((chapter) => chapter.id === selectedChapter.value?.id) ?? availableChapters[0] ?? null

    if (nextSelection) {
      await selectChapter(nextSelection)
    } else {
      selectedChapter.value = null
      noteContent.value = ''
    }
  } catch (caughtError) {
    viewError.value = caughtError instanceof Error ? caughtError.message : 'Unable to load this book.'
  }
}

watch(
  () => bookId.value,
  () => {
    void loadBook()
  },
  { immediate: true },
)

watch(
  () => selectedChapter.value?.id,
  () => {
    sessionTimer.reset()
  },
)

async function selectChapter(chapter: Chapter) {
  selectedChapter.value = chapter
  const note = await notesStore.fetchNote(chapter.id)
  noteContent.value = note?.content ?? ''
}

async function saveNote() {
  if (!selectedChapter.value) return

  savingNote.value = true
  try {
    await notesStore.saveNote(selectedChapter.value.id, noteContent.value)
  } finally {
    savingNote.value = false
  }
}

async function updateStatus(status: ReadingStatus) {
  if (!selectedChapter.value) return
  await progressStore.updateStatus(selectedChapter.value.id, status)
}

async function handleAddChapter() {
  if (!newChapterTitle.value.trim() || !newChapterSeq.value) return

  await booksStore.addChapter(
    bookId.value,
    newChapterTitle.value.trim(),
    Number.parseFloat(newChapterSeq.value),
    newChapterParentId.value || undefined,
  )

  newChapterTitle.value = ''
  newChapterSeq.value = ''
  newChapterParentId.value = ''
  showAddChapterModal.value = false
}

async function logStudySession() {
  if (!selectedChapter.value || sessionTimer.seconds.value === 0) return

  await progressStore.logTimeSpent(selectedChapter.value.id, sessionTimer.seconds.value)
  sessionTimer.reset()
}
</script>

<template>
  <div class="book-view page-shell">
    <section class="book-view__hero page-hero">
      <div class="book-view__hero-copy">
        <RouterLink to="/" class="book-view__back">
          <ArrowLeft :size="16" />
          Dashboard
        </RouterLink>

        <div v-if="currentBook">
          <p class="eyebrow">Book workspace</p>
          <h1 class="page-title">{{ currentBook.title }}</h1>
          <p class="page-subtitle">
            {{ currentBook.author ?? 'Independent technical reading track' }}
          </p>
        </div>

        <div class="book-view__hero-progress">
          <ProgressBar :completed="completedChapters" :total="totalChapters" />
          <span class="book-view__hero-progress-label numeric">{{ completedChapters }}/{{ totalChapters }} chapters complete</span>
        </div>

        <div class="book-view__hero-actions">
          <BaseButton @click="showAddChapterModal = true">
            <Plus :size="16" />
            Add chapter
          </BaseButton>
        </div>
      </div>

      <div class="book-view__hero-stats">
        <div class="stat-tile">
          <p class="stat-tile__label">Completed</p>
          <p class="stat-tile__value">{{ completedChapters }}</p>
          <p class="stat-tile__meta">Structured progress through the current book.</p>
        </div>
        <div class="stat-tile">
          <p class="stat-tile__label">Remaining</p>
          <p class="stat-tile__value">{{ remainingChapters }}</p>
          <p class="stat-tile__meta">Sections still open for first-pass reading.</p>
        </div>
        <div class="stat-tile">
          <p class="stat-tile__label">Current chapter</p>
          <p class="stat-tile__value">{{ currentChapterTimeLabel }}</p>
          <p class="stat-tile__meta">{{ selectedChapter ? selectedStatusLabel : 'Select a chapter to begin.' }}</p>
        </div>
      </div>
    </section>

    <p v-if="viewError" class="notice">{{ viewError }}</p>

    <section class="book-view__grid">
      <aside class="book-view__chapters surface-panel">
        <div class="book-view__section-head">
          <div>
            <p class="eyebrow">Structure</p>
            <h2 class="book-view__section-title">Chapter map</h2>
          </div>
          <span class="book-view__section-meta">{{ totalChapters }} total</span>
        </div>

        <div v-if="loading" class="book-view__panel-loading">
          <BaseLoader />
        </div>

        <div v-else-if="chapters.length === 0" class="book-view__panel-empty">
          <p>No chapters yet.</p>
          <BaseButton size="sm" variant="secondary" @click="showAddChapterModal = true">Add first chapter</BaseButton>
        </div>

        <ChapterList
          v-else
          :chapters="chapters"
          :selected-id="selectedChapter?.id ?? null"
          @select="selectChapter"
        />
      </aside>

      <div class="book-view__workspace">
        <template v-if="selectedChapter">
          <div class="book-view__workspace-header surface-panel">
            <div class="book-view__workspace-copy">
              <p class="eyebrow">Current chapter</p>
              <h2 class="book-view__workspace-title">
                {{ selectedChapter.sequence_number }} · {{ selectedChapter.title }}
              </h2>
              <p class="book-view__workspace-meta">
                {{ selectedStatusLabel }} · {{ currentChapterTimeLabel }}
              </p>
            </div>

            <div class="book-view__status-pills">
              <button
                v-for="opt in statusOptions"
                :key="opt.value"
                type="button"
                class="book-view__status-pill"
                :class="{ 'book-view__status-pill--active': currentStatus === opt.value }"
                :style="currentStatus === opt.value ? `color: ${opt.color}; border-color: ${opt.color};` : ''"
                @click="updateStatus(opt.value)"
              >
                {{ opt.label }}
              </button>
            </div>
          </div>

          <div class="book-view__workspace-grid">
            <div class="book-view__study surface-panel">
              <div>
                <p class="eyebrow">Focus session</p>
                <p class="book-view__study-time numeric">{{ sessionTimer.formatTime(sessionTimer.seconds.value) }}</p>
                <p class="book-view__study-copy">Track deliberate reading blocks and push them back into chapter progress.</p>
              </div>
              <div class="book-view__study-actions">
                <button class="book-view__study-icon" type="button" @click="sessionTimer.reset" title="Reset session">
                  <RotateCcw :size="16" />
                </button>
                <BaseButton variant="secondary" size="sm" @click="sessionTimer.isRunning.value ? sessionTimer.pause() : sessionTimer.start()">
                  <Play v-if="!sessionTimer.isRunning.value" :size="14" />
                  <Pause v-else :size="14" />
                  {{ sessionTimer.isRunning.value ? 'Pause' : 'Start' }}
                </BaseButton>
                <BaseButton size="sm" :disabled="sessionTimer.seconds.value === 0" @click="logStudySession">
                  <Save :size="14" />
                  Log session
                </BaseButton>
              </div>
            </div>

            <div class="book-view__checklist surface-panel">
              <p class="eyebrow">Recall checklist</p>
              <ul class="book-view__checklist-list">
                <li>Summarize the section in your own words before writing polished notes.</li>
                <li>Capture one code example or API pattern worth revisiting.</li>
                <li>Mark the chapter for review if recall still feels fragile.</li>
              </ul>
              <div class="book-view__checklist-note">
                <Sparkles :size="16" />
                Use the markdown workspace below as the durable version of your understanding.
              </div>
            </div>
          </div>

          <MarkdownEditor v-model="noteContent" :saving="savingNote" @save="saveNote" class="book-view__editor" />
        </template>

        <div v-else class="book-view__workspace-empty surface-panel">
          <h3>Select a chapter to start working.</h3>
          <p>Open a chapter from the left rail to update status, log study time, and write technical notes.</p>
        </div>
      </div>
    </section>

    <BaseModal v-model="showAddChapterModal" title="Add Chapter">
      <form class="book-view__modal-form" @submit.prevent="handleAddChapter">
        <BaseInput v-model="newChapterTitle" label="Chapter Title *" placeholder="Getting Started" />
        <BaseInput v-model="newChapterSeq" label="Sequence Number *" type="number" placeholder="1" />

        <div class="book-view__select-group">
          <label class="book-view__select-label" for="parent-chapter">Parent Chapter</label>
          <select id="parent-chapter" v-model="newChapterParentId" class="book-view__select">
            <option value="">None</option>
            <option v-for="chapter in flatChapters" :key="chapter.id" :value="chapter.id">
              {{ chapter.sequence_number }} · {{ chapter.title }}
            </option>
          </select>
        </div>

        <div class="form-actions">
          <BaseButton variant="secondary" type="button" @click="showAddChapterModal = false">Cancel</BaseButton>
          <BaseButton type="submit">Add Chapter</BaseButton>
        </div>
      </form>
    </BaseModal>
  </div>
</template>

<style scoped>
.book-view {
  gap: var(--space-xl);
}

.book-view__hero {
  grid-template-columns: minmax(0, 1.35fr) minmax(320px, 0.95fr);
}

.book-view__hero-copy {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.book-view__back {
  display: inline-flex;
  align-items: center;
  gap: var(--space-xs);
  color: var(--color-muted);
  font-size: var(--text-sm);
}

.book-view__back:hover {
  color: var(--color-on-dark);
}

.book-view__hero-progress {
  display: flex;
  align-items: center;
  gap: var(--space-md);
  max-width: 420px;
}

.book-view__hero-progress-label {
  font-size: var(--text-sm);
  color: var(--color-muted-strong);
  white-space: nowrap;
}

.book-view__hero-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-sm);
}

.book-view__hero-stats {
  display: grid;
  gap: var(--space-md);
}

.book-view__grid {
  display: grid;
  grid-template-columns: minmax(300px, 0.8fr) minmax(0, 1.7fr);
  gap: var(--space-lg);
}

.book-view__chapters,
.book-view__workspace-header,
.book-view__study,
.book-view__checklist {
  padding: var(--space-xl);
}

.book-view__section-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-md);
  margin-bottom: var(--space-lg);
}

.book-view__section-title,
.book-view__workspace-title {
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
  letter-spacing: -0.02em;
}

.book-view__section-meta,
.book-view__workspace-meta {
  font-size: var(--text-sm);
  color: var(--color-muted);
}

.book-view__panel-loading,
.book-view__panel-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-md);
  padding: var(--space-section) 0;
  color: var(--color-muted);
}

.book-view__workspace {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  min-width: 0;
}

.book-view__workspace-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-lg);
}

.book-view__workspace-copy {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.book-view__status-pills {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: var(--space-xs);
}

.book-view__status-pill {
  min-height: 32px;
  padding: 0 var(--space-sm);
  border-radius: var(--radius-pill);
  border: 1px solid var(--color-hairline);
  color: var(--color-muted);
  font-size: var(--text-xs);
  font-weight: var(--weight-medium);
  transition: background var(--transition-fast), border-color var(--transition-fast), color var(--transition-fast);
}

.book-view__status-pill:hover {
  background: rgba(43, 49, 57, 0.7);
  color: var(--color-on-dark);
}

.book-view__status-pill--active {
  background: rgba(255, 255, 255, 0.02);
}

.book-view__workspace-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.1fr) minmax(280px, 0.9fr);
  gap: var(--space-lg);
}

.book-view__study,
.book-view__checklist {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: var(--space-lg);
}

.book-view__study-time {
  font-size: clamp(var(--text-2xl), 5vw, var(--text-4xl));
  font-weight: var(--weight-bold);
  color: var(--color-on-dark);
  margin: var(--space-xs) 0 var(--space-sm);
}

.book-view__study-copy {
  color: var(--color-muted);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
}

.book-view__study-actions {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  flex-wrap: wrap;
}

.book-view__study-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-hairline);
  color: var(--color-muted);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: background var(--transition-fast), color var(--transition-fast);
}

.book-view__study-icon:hover {
  background: var(--color-surface-elevated);
  color: var(--color-on-dark);
}

.book-view__checklist-list {
  display: grid;
  gap: var(--space-sm);
  list-style: disc;
  padding-left: var(--space-lg);
  color: var(--color-muted-strong);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
}

.book-view__checklist-note {
  display: inline-flex;
  align-items: center;
  gap: var(--space-xs);
  color: var(--color-primary);
  font-size: var(--text-sm);
}

.book-view__editor {
  min-height: 520px;
}

.book-view__workspace-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-sm);
  min-height: 420px;
  padding: var(--space-section);
  text-align: center;
}

.book-view__workspace-empty h3 {
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
}

.book-view__workspace-empty p {
  max-width: 420px;
  color: var(--color-muted);
}

.book-view__modal-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.book-view__select-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.book-view__select-label {
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-muted);
}

.book-view__select {
  width: 100%;
  min-height: 40px;
  background: var(--color-surface-card);
  border: 1px solid var(--color-hairline);
  border-radius: var(--radius-md);
  padding: 10px var(--space-md);
  color: var(--color-on-dark);
}

.book-view__select:focus {
  outline: none;
  border-color: rgba(59, 130, 246, 0.55);
  box-shadow: var(--shadow-focus);
}

@media (max-width: 1100px) {
  .book-view__hero,
  .book-view__grid,
  .book-view__workspace-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 720px) {
  .book-view__hero-progress,
  .book-view__workspace-header,
  .book-view__study-actions {
    flex-direction: column;
    align-items: flex-start;
  }

  .book-view__status-pills {
    justify-content: flex-start;
  }
}
</style>
