import { defineStore } from 'pinia';
import { ref } from 'vue';
import { assertSupabaseConfigured, supabase } from '@/lib/supabase';
import type { Progress, ReadingStatus } from '@/types';
import { useAuthStore } from './auth';

type ProgressWithBook = Progress & {
  reading_chapters: { book_id: string } | { book_id: string }[] | null;
};

export const useProgressStore = defineStore('progress', () => {
  const progressMap = ref<Record<string, Progress>>({});

  async function fetchProgressForBook(bookId: string) {
    const auth = useAuthStore();
    if (!auth.user) return;

    assertSupabaseConfigured();

    const { data, error } = await supabase
      .from('reading_progress')
      .select(
        'id, user_id, chapter_id, status, time_spent_seconds, updated_at, reading_chapters!inner(book_id)',
      )
      .eq('user_id', auth.user.id)
      .eq('reading_chapters.book_id', bookId)
      .range(0, 4999);

    if (error) throw error;

    const rows = (data ?? []) as ProgressWithBook[];
    const validChapterIds = new Set<string>();
    const nextMap = { ...progressMap.value };

    for (const row of rows) {
      const { reading_chapters: _readingChapters, ...progress } = row;
      nextMap[progress.chapter_id] = progress;
      validChapterIds.add(progress.chapter_id);
    }

    for (const key of Object.keys(nextMap)) {
      if (!validChapterIds.has(key)) {
        delete nextMap[key];
      }
    }

    progressMap.value = nextMap;
  }

  async function updateStatus(chapterId: string, status: ReadingStatus) {
    const auth = useAuthStore();
    if (!auth.user) return null;

    assertSupabaseConfigured();

    const existing = progressMap.value[chapterId];
    const { data, error } = await supabase
      .from('reading_progress')
      .upsert(
        {
          user_id: auth.user.id,
          chapter_id: chapterId,
          status,
          time_spent_seconds: existing?.time_spent_seconds ?? 0,
          updated_at: new Date().toISOString(),
        },
        { onConflict: 'user_id,chapter_id' },
      )
      .select('*')
      .single();

    if (error) throw error;

    progressMap.value = {
      ...progressMap.value,
      [chapterId]: data,
    };

    return data;
  }

  async function logTimeSpent(chapterId: string, seconds: number) {
    const auth = useAuthStore();
    if (!auth.user || seconds <= 0) return null;

    assertSupabaseConfigured();

    const existing = progressMap.value[chapterId];
    const { data, error } = await supabase
      .from('reading_progress')
      .upsert(
        {
          user_id: auth.user.id,
          chapter_id: chapterId,
          status: existing?.status ?? 'in_progress',
          time_spent_seconds: (existing?.time_spent_seconds ?? 0) + seconds,
          updated_at: new Date().toISOString(),
        },
        { onConflict: 'user_id,chapter_id' },
      )
      .select('*')
      .single();

    if (error) throw error;

    progressMap.value = {
      ...progressMap.value,
      [chapterId]: data,
    };

    return data;
  }

  function getProgress(chapterId: string) {
    return progressMap.value[chapterId];
  }

  return { progressMap, fetchProgressForBook, updateStatus, logTimeSpent, getProgress };
});
