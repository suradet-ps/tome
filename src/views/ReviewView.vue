<script setup lang="ts">
import { Brain, CheckCheck, Clock3, Plus, Sparkles } from 'lucide-vue-next'
import { computed, onMounted, ref } from 'vue'
import BaseButton from '@/components/common/BaseButton.vue'
import BaseInput from '@/components/common/BaseInput.vue'
import BaseLoader from '@/components/common/BaseLoader.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import BaseTextarea from '@/components/common/BaseTextarea.vue'
import FlashcardContainer from '@/components/review/FlashcardContainer.vue'
import PomodoroTimer from '@/components/review/PomodoroTimer.vue'
import { assertSupabaseConfigured, supabase, supabaseConfigError } from '@/lib/supabase'
import { useAuthStore } from '@/stores/auth'
import type { Flashcard } from '@/types'

const auth = useAuthStore()
const cards = ref<Flashcard[]>([])
const loading = ref(false)
const showAddModal = ref(false)
const newFront = ref('')
const newBack = ref('')
const adding = ref(false)
const error = ref('')
const activeTab = ref<'review' | 'timer'>('review')
const configMessage = computed(() => supabaseConfigError)

async function loadCards() {
  if (!auth.user || supabaseConfigError) {
    cards.value = []
    return
  }

  loading.value = true
  error.value = ''

  try {
    assertSupabaseConfigured()

    const { data, error: loadError } = await supabase
      .from('reading_flashcards')
      .select('*')
      .eq('user_id', auth.user.id)
      .lte('next_review', new Date().toISOString())
      .order('next_review', { ascending: true })

    if (loadError) throw loadError

    cards.value = (data ?? []) as Flashcard[]
  } catch (caughtError) {
    error.value = caughtError instanceof Error ? caughtError.message : 'Unable to load flashcards.'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  void loadCards()
})

async function handleRated(cardId: string, quality: number) {
  const card = cards.value.find((item) => item.id === cardId)
  if (!card) return

  error.value = ''

  try {
    let easeFactor = card.ease_factor + (0.1 - (5 - quality) * (0.08 + (5 - quality) * 0.02))
    easeFactor = Math.max(1.3, easeFactor)

    const interval =
      quality < 3
        ? 1
        : card.interval_days === 0
          ? 1
          : card.interval_days === 1
            ? 6
            : Math.round(card.interval_days * easeFactor)

    const nextReview = new Date()
    nextReview.setDate(nextReview.getDate() + interval)

    const { error: updateError } = await supabase
      .from('reading_flashcards')
      .update({
        ease_factor: easeFactor,
        interval_days: interval,
        next_review: nextReview.toISOString(),
      })
      .eq('id', cardId)

    if (updateError) throw updateError

    cards.value = cards.value.filter((item) => item.id !== cardId)
  } catch (caughtError) {
    error.value = caughtError instanceof Error ? caughtError.message : 'Unable to update flashcard.'
  }
}

async function handleAddCard() {
  if (!newFront.value.trim() || !newBack.value.trim() || !auth.user) return

  adding.value = true
  error.value = ''

  try {
    assertSupabaseConfigured()

    const { data, error: insertError } = await supabase
      .from('reading_flashcards')
      .insert({
        user_id: auth.user.id,
        chapter_id: null,
        front: newFront.value.trim(),
        back: newBack.value.trim(),
      })
      .select('*')
      .single()

    if (insertError) throw insertError

    if (data && new Date(data.next_review) <= new Date()) {
      cards.value = [...cards.value, data as Flashcard]
    }

    newFront.value = ''
    newBack.value = ''
    showAddModal.value = false
  } catch (caughtError) {
    error.value = caughtError instanceof Error ? caughtError.message : 'Unable to add flashcard.'
  } finally {
    adding.value = false
  }
}
</script>

<template>
  <div class="review page-shell">
    <section class="review__hero page-hero">
      <div class="review__hero-copy">
        <p class="eyebrow">Retention workflow</p>
        <h1 class="page-title">Review what matters before it fades.</h1>
        <p class="page-subtitle">
          Use spaced repetition for durable recall and focus blocks for deeper reading sessions.
        </p>

        <div class="review__hero-actions">
          <BaseButton @click="showAddModal = true">
            <Plus :size="16" />
            Add card
          </BaseButton>
          <BaseButton variant="secondary" @click="activeTab = 'timer'">
            <Clock3 :size="16" />
            Open focus timer
          </BaseButton>
        </div>
      </div>

      <div class="review__hero-panel surface-panel surface-panel--soft">
        <div class="review__hero-panel-row">
          <span class="eyebrow">Due now</span>
          <span class="review__hero-panel-value numeric">{{ cards.length }}</span>
        </div>
        <p class="review__hero-panel-title">
          {{ cards.length > 0 ? 'Work the recall queue first.' : 'Your recall queue is clear.' }}
        </p>
        <p class="review__hero-panel-copy">
          Rate honestly, let intervals expand naturally, and use the timer when you want a deeper review block.
        </p>
      </div>
    </section>

    <p v-if="configMessage" class="notice">{{ configMessage }}</p>
    <p v-else-if="error" class="notice">{{ error }}</p>

    <div class="review__tabs">
      <button
        type="button"
        class="review__tab"
        :class="{ 'review__tab--active': activeTab === 'review' }"
        @click="activeTab = 'review'"
      >
        <Brain :size="16" />
        Flashcards
        <span v-if="cards.length" class="review__badge">{{ cards.length }}</span>
      </button>
      <button
        type="button"
        class="review__tab"
        :class="{ 'review__tab--active': activeTab === 'timer' }"
        @click="activeTab = 'timer'"
      >
        <Clock3 :size="16" />
        Focus timer
      </button>
    </div>

    <div v-if="activeTab === 'review'" class="review__grid">
      <div class="review__board surface-panel">
        <div class="review__board-head">
          <div>
            <p class="eyebrow">Review board</p>
            <h2 class="review__section-title">Active recall queue</h2>
          </div>
          <span class="review__section-meta">{{ cards.length }} due</span>
        </div>

        <div v-if="loading" class="review__loading">
          <BaseLoader :size="32" />
        </div>
        <div v-else-if="cards.length === 0" class="review__done">
          <div class="review__done-icon">
            <CheckCheck :size="26" />
          </div>
          <h2 class="review__done-title">All caught up.</h2>
          <p class="review__done-sub">No cards are due for review. Capture more prompts or switch into a focus session.</p>
          <BaseButton variant="secondary" @click="showAddModal = true">Add new cards</BaseButton>
        </div>
        <div v-else class="review__flashcard-area">
          <p class="review__count">{{ cards.length }} card{{ cards.length !== 1 ? 's' : '' }} remaining</p>
          <FlashcardContainer :card="cards[0]" @rated="handleRated" />
        </div>
      </div>

      <aside class="review__side">
        <div class="review__side-card surface-panel">
          <p class="eyebrow">Protocol</p>
          <ul class="review__protocol-list">
            <li>Attempt recall before flipping the card.</li>
            <li>Use “Hard” only when the answer is partial but recoverable.</li>
            <li>Create another card when one prompt carries too many ideas.</li>
          </ul>
        </div>

        <div class="review__side-card surface-panel">
          <p class="eyebrow">Capture standard</p>
          <h3 class="review__side-title">Turn dense notes into smaller prompts.</h3>
          <p class="review__side-copy">
            Strong cards test one concept, one code pattern, or one mental model at a time.
          </p>
          <BaseButton variant="secondary" @click="showAddModal = true">
            <Sparkles :size="16" />
            Create a prompt
          </BaseButton>
        </div>
      </aside>
    </div>

    <div v-else class="review__grid">
      <div class="review__timer surface-panel">
        <PomodoroTimer />
      </div>

      <aside class="review__side">
        <div class="review__side-card surface-panel">
          <p class="eyebrow">Cadence</p>
          <ul class="review__protocol-list">
            <li>Focus: 25 minutes for deep reading or note consolidation.</li>
            <li>Short break: 5 minutes to reset before the next block.</li>
            <li>Long break: 15 minutes after several hard rounds.</li>
          </ul>
        </div>

        <div class="review__side-card surface-panel">
          <p class="eyebrow">Why it works</p>
          <h3 class="review__side-title">Pair recall with deliberate timeboxing.</h3>
          <p class="review__side-copy">
            The timer keeps review sessions honest while preventing marathon note-taking that turns passive.
          </p>
        </div>
      </aside>
    </div>

    <BaseModal v-model="showAddModal" title="Add Flashcard">
      <form class="review__modal-form" @submit.prevent="handleAddCard">
        <BaseInput v-model="newFront" label="Question / Front *" placeholder="What is ownership in Rust?" />
        <BaseTextarea
          v-model="newBack"
          label="Answer / Back *"
          placeholder="Ownership is Rust's central feature..."
          :rows="6"
        />
        <div class="form-actions">
          <BaseButton variant="secondary" type="button" @click="showAddModal = false">Cancel</BaseButton>
          <BaseButton type="submit" :loading="adding">Add Card</BaseButton>
        </div>
      </form>
    </BaseModal>
  </div>
</template>

<style scoped>
.review {
  gap: var(--space-xl);
}

.review__hero {
  grid-template-columns: minmax(0, 1.5fr) minmax(320px, 0.85fr);
}

.review__hero-copy {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.review__hero-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-sm);
}

.review__hero-panel {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
  justify-content: space-between;
  padding: var(--space-xl);
}

.review__hero-panel-row {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: var(--space-md);
}

.review__hero-panel-value {
  font-size: clamp(var(--text-2xl), 6vw, var(--text-4xl));
  font-weight: var(--weight-bold);
  color: var(--color-primary);
}

.review__hero-panel-title {
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
}

.review__hero-panel-copy {
  color: var(--color-muted);
  font-size: var(--text-sm);
}

.review__tabs {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px;
  background: rgba(11, 14, 17, 0.72);
  border: 1px solid var(--color-hairline);
  border-radius: var(--radius-pill);
  width: fit-content;
}

.review__tab {
  display: inline-flex;
  align-items: center;
  gap: var(--space-xs);
  min-height: 40px;
  padding: 0 var(--space-md);
  border-radius: var(--radius-pill);
  color: var(--color-muted);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  transition: background var(--transition-fast), color var(--transition-fast);
}

.review__tab:hover,
.review__tab--active {
  background: var(--color-surface-card);
  color: var(--color-on-dark);
}

.review__badge {
  background: var(--color-primary);
  color: var(--color-on-primary);
  font-size: var(--text-xs);
  font-weight: var(--weight-bold);
  padding: 2px 8px;
  border-radius: var(--radius-pill);
}

.review__grid {
  display: grid;
  grid-template-columns: minmax(0, 1.45fr) minmax(300px, 0.75fr);
  gap: var(--space-lg);
}

.review__board,
.review__side-card,
.review__timer {
  padding: var(--space-xl);
}

.review__board-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-lg);
  margin-bottom: var(--space-lg);
}

.review__section-title {
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
  letter-spacing: -0.02em;
}

.review__section-meta {
  font-size: var(--text-sm);
  color: var(--color-muted);
}

.review__loading {
  display: flex;
  justify-content: center;
  padding: var(--space-section);
}

.review__done {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-md);
  padding: var(--space-section) 0;
  text-align: center;
}

.review__done-icon {
  width: 56px;
  height: 56px;
  border-radius: var(--radius-full);
  background: rgba(14, 203, 129, 0.12);
  color: var(--color-success);
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.review__done-title {
  font-size: var(--text-2xl);
  font-weight: var(--weight-bold);
  color: var(--color-on-dark);
}

.review__done-sub {
  max-width: 420px;
  color: var(--color-muted);
}

.review__flashcard-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-lg);
}

.review__count {
  font-size: var(--text-sm);
  color: var(--color-muted);
}

.review__side {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.review__side-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.review__side-title {
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
}

.review__side-copy {
  color: var(--color-muted);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
}

.review__protocol-list {
  display: grid;
  gap: var(--space-sm);
  list-style: disc;
  padding-left: var(--space-lg);
  color: var(--color-muted-strong);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
}

.review__modal-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

@media (max-width: 1100px) {
  .review__hero,
  .review__grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 640px) {
  .review__hero-actions {
    flex-direction: column;
  }

  .review__tabs {
    width: 100%;
    justify-content: space-between;
  }

  .review__tab {
    flex: 1;
    justify-content: center;
  }
}
</style>
