<script setup lang="ts">
import { Minus, ThumbsDown, ThumbsUp } from 'lucide-vue-next'
import { ref } from 'vue'
import BaseButton from '@/components/common/BaseButton.vue'
import type { Flashcard } from '@/types'

interface Props {
  card: Flashcard
}

const props = defineProps<Props>()
const emit = defineEmits<{ rated: [cardId: string, quality: number] }>()

const flipped = ref(false)

function flip() {
  flipped.value = !flipped.value
}

function rate(quality: number) {
  emit('rated', props.card.id, quality)
  flipped.value = false
}
</script>

<template>
  <div class="flashcard">
    <div class="flashcard__meta">
      <span class="flashcard__badge">Due now</span>
      <span class="flashcard__meta-copy">{{ props.card.chapter_id ? 'Linked to a chapter prompt' : 'General knowledge prompt' }}</span>
    </div>

    <button type="button" class="flashcard__card" :class="{ 'flashcard__card--flipped': flipped }" @click="flip">
      <div class="flashcard__face flashcard__face--front">
        <p class="flashcard__side-label">Question</p>
        <p class="flashcard__content">{{ props.card.front }}</p>
        <p class="flashcard__hint">Click to reveal answer</p>
      </div>
      <div class="flashcard__face flashcard__face--back">
        <p class="flashcard__side-label flashcard__side-label--back">Answer</p>
        <p class="flashcard__content">{{ props.card.back }}</p>
      </div>
    </button>

    <div v-if="flipped" class="flashcard__actions">
      <BaseButton variant="danger" size="sm" @click.stop="rate(1)">
        <ThumbsDown :size="14" />
        Hard
      </BaseButton>
      <BaseButton variant="secondary" size="sm" @click.stop="rate(3)">
        <Minus :size="14" />
        OK
      </BaseButton>
      <BaseButton size="sm" class="flashcard__easy" @click.stop="rate(5)">
        <ThumbsUp :size="14" />
        Easy
      </BaseButton>
    </div>
  </div>
</template>

<style scoped>
.flashcard {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-md);
  width: 100%;
}

.flashcard__meta {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-sm);
  flex-wrap: wrap;
}

.flashcard__badge {
  display: inline-flex;
  align-items: center;
  min-height: 28px;
  padding: 0 var(--space-sm);
  border-radius: var(--radius-pill);
  background: rgba(252, 213, 53, 0.12);
  color: var(--color-primary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.flashcard__meta-copy {
  font-size: var(--text-sm);
  color: var(--color-muted);
}

.flashcard__card {
  width: 100%;
  max-width: 560px;
  height: 280px;
  position: relative;
  cursor: pointer;
  perspective: 1000px;
  border: none;
  padding: 0;
  background: transparent;
}

.flashcard__face {
  position: absolute;
  inset: 0;
  background: var(--color-surface-card);
  border: 1px solid var(--color-hairline);
  border-radius: var(--radius-xl);
  padding: var(--space-xl);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-md);
  backface-visibility: hidden;
  transition: transform var(--transition-slow);
  box-shadow: var(--shadow-subtle);
}

.flashcard__face--back {
  transform: rotateY(180deg);
  background: linear-gradient(180deg, #2b3139 0%, #1e2329 100%);
}

.flashcard__card--flipped .flashcard__face--front {
  transform: rotateY(-180deg);
}

.flashcard__card--flipped .flashcard__face--back {
  transform: rotateY(0deg);
}

.flashcard__side-label {
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--color-muted);
}

.flashcard__side-label--back {
  color: var(--color-primary);
}

.flashcard__content {
  font-size: var(--text-md);
  font-weight: var(--weight-medium);
  color: var(--color-on-dark);
  text-align: center;
  line-height: var(--leading-relaxed);
  max-width: 420px;
}

.flashcard__hint {
  font-size: var(--text-xs);
  color: var(--color-muted);
}

.flashcard__actions {
  display: flex;
  gap: var(--space-md);
  flex-wrap: wrap;
  justify-content: center;
}

.flashcard__easy {
  background: var(--color-success);
  color: var(--color-on-dark);
}

.flashcard__easy:hover:not(:disabled) {
  background: #11d48a;
}

@media (max-width: 640px) {
  .flashcard__card {
    height: 320px;
  }

  .flashcard__actions {
    width: 100%;
  }

  .flashcard__actions :deep(.btn) {
    flex: 1 1 140px;
  }
}
</style>
