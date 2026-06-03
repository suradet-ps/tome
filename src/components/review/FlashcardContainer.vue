<script setup lang="ts">
import { Minus, ThumbsDown, ThumbsUp } from 'lucide-vue-next';
import { ref } from 'vue';
import BaseButton from '@/components/common/BaseButton.vue';
import type { Flashcard } from '@/types';

interface Props {
  card: Flashcard;
}

const props = defineProps<Props>();
const emit = defineEmits<{ rated: [cardId: string, quality: number] }>();

const flipped = ref(false);

function flip() {
  flipped.value = !flipped.value;
}

function rate(quality: number) {
  emit('rated', props.card.id, quality);
  flipped.value = false;
}
</script>

<template>
  <div class="flashcard">
    <button type="button" class="flashcard__card" :class="{ 'flashcard__card--flipped': flipped }" @click="flip">
      <div class="flashcard__face flashcard__face--front">
        <span class="flashcard__label">Question</span>
        <p class="flashcard__content">{{ props.card.front }}</p>
        <span class="flashcard__hint">Click to reveal</span>
      </div>
      <div class="flashcard__face flashcard__face--back">
        <span class="flashcard__label flashcard__label--accent">Answer</span>
        <p class="flashcard__content">{{ props.card.back }}</p>
      </div>
    </button>

    <div v-if="flipped" class="flashcard__actions">
      <BaseButton variant="danger" size="sm" @click.stop="rate(1)">
        <ThumbsDown :size="13" />
        Hard
      </BaseButton>
      <BaseButton variant="secondary" size="sm" @click.stop="rate(3)">
        <Minus :size="13" />
        OK
      </BaseButton>
      <BaseButton size="sm" class="flashcard__easy" @click.stop="rate(5)">
        <ThumbsUp :size="13" />
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

.flashcard__card {
  width: 100%;
  max-width: 520px;
  height: 260px;
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
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-hairline);
  border-radius: var(--radius-xl);
  padding: var(--space-xl);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-sm);
  backface-visibility: hidden;
  transition: transform var(--transition-slow), border-color var(--transition-fast);
}

.flashcard__face:hover {
  border-color: rgba(252, 213, 53, 0.32);
}

.flashcard__face--back {
  transform: rotateY(180deg);
  background: var(--color-canvas);
}

.flashcard__card--flipped .flashcard__face--front {
  transform: rotateY(-180deg);
}

.flashcard__card--flipped .flashcard__face--back {
  transform: rotateY(0deg);
}

.flashcard__label {
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--color-muted);
}

.flashcard__label--accent {
  color: var(--color-primary);
}

.flashcard__content {
  font-size: var(--text-md);
  font-weight: var(--weight-medium);
  color: var(--color-on-dark);
  text-align: center;
  line-height: var(--leading-relaxed);
  max-width: 400px;
}

.flashcard__hint {
  font-size: var(--text-xs);
  color: var(--color-muted);
}

.flashcard__actions {
  display: flex;
  gap: var(--space-xs);
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
    height: 280px;
  }

  .flashcard__actions {
    width: 100%;
  }

  .flashcard__actions :deep(.btn) {
    flex: 1;
  }
}
</style>
