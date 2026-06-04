import { defineStore } from 'pinia';
import { ref } from 'vue';
import { assertSupabaseConfigured, supabase } from '@/lib/supabase';
import type { Note } from '@/types';
import { useAuthStore } from './auth';

const MAX_NOTE_LENGTH = 200_000;

export const useNotesStore = defineStore('notes', () => {
  const notesMap = ref<Record<string, Note>>({});

  async function fetchNote(chapterId: string) {
    const auth = useAuthStore();
    if (!auth.user) return null;

    assertSupabaseConfigured();

    const { data, error } = await supabase
      .from('reading_notes')
      .select('*')
      .eq('user_id', auth.user.id)
      .eq('chapter_id', chapterId)
      .maybeSingle();

    if (error) throw error;

    if (data) {
      notesMap.value = {
        ...notesMap.value,
        [chapterId]: data,
      };
    }

    return data;
  }

  async function saveNote(chapterId: string, content: string) {
    const auth = useAuthStore();
    if (!auth.user) return null;

    if (content.length > MAX_NOTE_LENGTH) {
      throw new Error(`Note exceeds maximum length of ${MAX_NOTE_LENGTH} characters.`);
    }

    assertSupabaseConfigured();

    const existing = notesMap.value[chapterId];
    const { data, error } = await supabase
      .from('reading_notes')
      .upsert(
        {
          id: existing?.id,
          user_id: auth.user.id,
          chapter_id: chapterId,
          content,
          created_at: existing?.created_at,
          updated_at: new Date().toISOString(),
        },
        { onConflict: 'user_id,chapter_id' },
      )
      .select('*')
      .single();

    if (error) throw error;

    notesMap.value = {
      ...notesMap.value,
      [chapterId]: data,
    };

    return data;
  }

  return { notesMap, fetchNote, saveNote };
});
