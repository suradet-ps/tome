<script setup lang="ts">
import { Brain, CheckCheck, Clock3, Plus } from 'lucide-vue-next';
import { computed, onMounted, ref } from 'vue';
import BaseButton from '@/components/common/BaseButton.vue';
import BaseInput from '@/components/common/BaseInput.vue';
import BaseLoader from '@/components/common/BaseLoader.vue';
import BaseModal from '@/components/common/BaseModal.vue';
import BaseTextarea from '@/components/common/BaseTextarea.vue';
import FlashcardContainer from '@/components/review/FlashcardContainer.vue';
import PomodoroTimer from '@/components/review/PomodoroTimer.vue';
import { assertSupabaseConfigured, supabase, supabaseConfigError } from '@/lib/supabase';
import { useAuthStore } from '@/stores/auth';
import type { Flashcard } from '@/types';

const auth = useAuthStore();
const cards = ref<Flashcard[]>([]);
const loading = ref(false);
const showAddModal = ref(false);
const newFront = ref('');
const newBack = ref('');
const adding = ref(false);
const error = ref('');
const activeTab = ref<'cards' | 'timer'>('cards');
const configMessage = computed(() => supabaseConfigError);

async function loadCards() {
  if (!auth.user || supabaseConfigError) {
    cards.value = [];
    return;
  }

  loading.value = true;
  error.value = '';

  try {
    assertSupabaseConfigured();

    const { data, error: loadError } = await supabase
      .from('reading_flashcards')
      .select('*')
      .eq('user_id', auth.user.id)
      .lte('next_review', new Date().toISOString())
      .order('next_review', { ascending: true })
      .range(0, 999);

    if (loadError) throw loadError;
    cards.value = (data ?? []) as Flashcard[];
  } catch (caughtError) {
    error.value = caughtError instanceof Error ? caughtError.message : 'Unable to load flashcards.';
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void loadCards();
});

async function handleRated(cardId: string, quality: number) {
  const card = cards.value.find((item) => item.id === cardId);
  if (!card) return;
  error.value = '';

  try {
    let easeFactor = card.ease_factor + (0.1 - (5 - quality) * (0.08 + (5 - quality) * 0.02));
    easeFactor = Math.max(1.3, easeFactor);

    const interval =
      quality < 3
        ? 1
        : card.interval_days === 0
          ? 1
          : card.interval_days === 1
            ? 6
            : Math.round(card.interval_days * easeFactor);

    const nextReview = new Date();
    nextReview.setDate(nextReview.getDate() + interval);

    const { error: updateError } = await supabase
      .from('reading_flashcards')
      .update({
        ease_factor: easeFactor,
        interval_days: interval,
        next_review: nextReview.toISOString(),
      })
      .eq('id', cardId);

    if (updateError) throw updateError;
    cards.value = cards.value.filter((item) => item.id !== cardId);
  } catch (caughtError) {
    error.value =
      caughtError instanceof Error ? caughtError.message : 'Unable to update flashcard.';
  }
}

async function handleAddCard() {
  if (!newFront.value.trim() || !newBack.value.trim() || !auth.user) return;

  adding.value = true;
  error.value = '';

  try {
    assertSupabaseConfigured();

    const { data, error: insertError } = await supabase
      .from('reading_flashcards')
      .insert({
        user_id: auth.user.id,
        chapter_id: null,
        front: newFront.value.trim(),
        back: newBack.value.trim(),
      })
      .select('*')
      .single();

    if (insertError) throw insertError;

    if (data && new Date(data.next_review) <= new Date()) {
      cards.value = [...cards.value, data as Flashcard];
    }

    newFront.value = '';
    newBack.value = '';
    showAddModal.value = false;
  } catch (caughtError) {
    error.value = caughtError instanceof Error ? caughtError.message : 'Unable to add flashcard.';
  } finally {
    adding.value = false;
  }
}
</script>

<template>
  <div class="page review">
    <header class="page-header">
      <div>
        <h1 class="page-header__title">Review</h1>
        <p class="page-header__sub">Recall flashcards and run focus sessions.</p>
      </div>
      <div class="page-header__actions">
        <BaseButton variant="secondary" size="sm" @click="showAddModal = true">
          <Plus :size="14" />
          Add card
        </BaseButton>
      </div>
    </header>

    <p v-if="configMessage" class="notice">{{ configMessage }}</p>
    <p v-else-if="error" class="notice">{{ error }}</p>

    <div class="review__tabs" role="tablist" aria-label="Review sections">
      <button
        id="review-tab-cards"
        type="button"
        role="tab"
        class="review__tab"
        :class="{ 'review__tab--active': activeTab === 'cards' }"
        :aria-selected="activeTab === 'cards'"
        :aria-controls="'review-panel-cards'"
        :tabindex="activeTab === 'cards' ? 0 : -1"
        @click="activeTab = 'cards'"
      >
        <Brain :size="14" />
        Flashcards
        <span v-if="cards.length" class="review__badge" aria-hidden="true">{{ cards.length }}</span>
      </button>
      <button
        id="review-tab-timer"
        type="button"
        role="tab"
        class="review__tab"
        :class="{ 'review__tab--active': activeTab === 'timer' }"
        :aria-selected="activeTab === 'timer'"
        :aria-controls="'review-panel-timer'"
        :tabindex="activeTab === 'timer' ? 0 : -1"
        @click="activeTab = 'timer'"
      >
        <Clock3 :size="14" />
        Timer
      </button>
    </div>

    <section
      v-if="activeTab === 'cards'"
      id="review-panel-cards"
      role="tabpanel"
      aria-labelledby="review-tab-cards"
      class="review__content surface"
    >
      <div v-if="loading" class="review__loading">
        <BaseLoader :size="28" />
      </div>

      <div v-else-if="cards.length === 0" class="review__done">
        <div class="review__done-icon">
          <CheckCheck :size="22" />
        </div>
        <h2 class="review__done-title">All caught up</h2>
        <p class="review__done-sub">No cards are due. Add new prompts or come back later.</p>
        <BaseButton variant="secondary" size="sm" @click="showAddModal = true">
          <Plus :size="14" />
          Add card
        </BaseButton>
      </div>

      <div v-else class="review__cards">
        <p class="review__count">
          <span class="numeric">{{ cards.length }}</span>
          {{ cards.length === 1 ? 'card' : 'cards' }} left
        </p>
        <FlashcardContainer :card="cards[0]" @rated="handleRated" />
      </div>
    </section>

    <section
      v-else
      id="review-panel-timer"
      role="tabpanel"
      aria-labelledby="review-tab-timer"
      class="review__content surface"
    >
      <PomodoroTimer />
    </section>

    <BaseModal v-model="showAddModal" title="Add flashcard">
      <form class="review__form" aria-label="Add flashcard" @submit.prevent="handleAddCard">
        <BaseInput v-model="newFront" label="Front (question)" placeholder="What is ownership?" />
        <BaseTextarea
          v-model="newBack"
          label="Back (answer)"
          placeholder="Ownership is a set of rules that..."
          :rows="5"
        />
        <div class="form-actions">
          <BaseButton variant="secondary" type="button" @click="showAddModal = false">Cancel</BaseButton>
          <BaseButton type="submit" :loading="adding">Add</BaseButton>
        </div>
      </form>
    </BaseModal>
  </div>
</template>

<style scoped>
.review__tabs {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px;
  background: var(--color-surface-card);
  border: 1px solid var(--color-hairline);
  border-radius: var(--radius-pill);
  width: fit-content;
}

.review__tab {
  display: inline-flex;
  align-items: center;
  gap: var(--space-xs);
  height: 32px;
  padding: 0 var(--space-md);
  border-radius: var(--radius-pill);
  color: var(--color-muted);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  transition: all var(--transition-fast);
}

.review__tab:hover {
  color: var(--color-on-dark);
}

.review__tab--active {
  background: var(--color-surface-elevated);
  color: var(--color-on-dark);
}

.review__badge {
  background: var(--color-primary);
  color: var(--color-on-primary);
  font-size: var(--text-xs);
  font-weight: var(--weight-bold);
  padding: 2px 8px;
  border-radius: var(--radius-pill);
  line-height: 1;
}

.review__content {
  padding: var(--space-xl);
  min-height: 420px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.review__loading {
  display: flex;
  justify-content: center;
  padding: var(--space-xl) 0;
}

.review__done {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-sm);
  text-align: center;
  padding: var(--space-lg) 0;
}

.review__done-icon {
  width: 48px;
  height: 48px;
  border-radius: var(--radius-full);
  background: rgba(14, 203, 129, 0.12);
  color: var(--color-success);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-bottom: var(--space-xs);
}

.review__done-title {
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  color: var(--color-on-dark);
}

.review__done-sub {
  max-width: 320px;
  color: var(--color-muted);
  font-size: var(--text-sm);
  margin-bottom: var(--space-xs);
}

.review__cards {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-md);
  width: 100%;
}

.review__count {
  font-size: var(--text-xs);
  color: var(--color-muted);
}

.review__count .numeric {
  color: var(--color-on-dark);
  font-weight: var(--weight-semibold);
}

.review__form {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

@media (max-width: 640px) {
  .review__tabs {
    width: 100%;
  }

  .review__tab {
    flex: 1;
    justify-content: center;
  }

  .review__content {
    padding: var(--space-lg);
  }
}
</style>
