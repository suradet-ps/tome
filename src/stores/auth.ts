import type { Session, Subscription, User } from '@supabase/supabase-js';
import { defineStore } from 'pinia';
import { ref } from 'vue';
import { assertSupabaseConfigured, supabase, supabaseConfigError } from '@/lib/supabase';
import type { Profile } from '@/types';

export const useAuthStore = defineStore('auth', () => {
  const user = ref<User | null>(null);
  const profile = ref<Profile | null>(null);
  const session = ref<Session | null>(null);
  const loading = ref(false);
  const initialized = ref(false);

  let initPromise: Promise<void> | null = null;
  let authSubscription: Subscription | null = null;

  async function signIn(email: string, password: string) {
    assertSupabaseConfigured();
    loading.value = true;

    try {
      const { data, error } = await supabase.auth.signInWithPassword({ email, password });
      if (error) throw error;

      session.value = data.session;
      user.value = data.user;
      await fetchProfile();
    } finally {
      loading.value = false;
    }
  }

  async function signUp(email: string, password: string, username: string) {
    assertSupabaseConfigured();
    loading.value = true;

    try {
      const { data, error } = await supabase.auth.signUp({
        email,
        password,
        options: {
          data: {
            username,
          },
        },
      });
      if (error) throw error;

      session.value = data.session ?? null;
      user.value = data.user ?? null;

      if (session.value && user.value) {
        await fetchProfile();
      }

      return data;
    } finally {
      loading.value = false;
    }
  }

  async function signOut() {
    if (supabaseConfigError) {
      user.value = null;
      profile.value = null;
      session.value = null;
      return;
    }

    await supabase.auth.signOut();
    user.value = null;
    profile.value = null;
    session.value = null;
  }

  async function fetchProfile() {
    if (!user.value || supabaseConfigError) return null;

    const { data, error } = await supabase
      .from('reading_profiles')
      .select('*')
      .eq('id', user.value.id)
      .maybeSingle();

    if (error) throw error;

    profile.value = data;
    return data;
  }

  async function initAuth() {
    if (initialized.value) return;
    if (initPromise) return initPromise;

    initPromise = (async () => {
      if (supabaseConfigError) {
        initialized.value = true;
        return;
      }

      const { data, error } = await supabase.auth.getSession();
      if (error) throw error;

      session.value = data.session;
      user.value = data.session?.user ?? null;

      if (user.value) {
        await fetchProfile();
      }

      const { data: subData } = supabase.auth.onAuthStateChange((_event, newSession) => {
        session.value = newSession;
        user.value = newSession?.user ?? null;

        if (user.value) {
          void fetchProfile().catch((caught) => {
            console.error('Failed to fetch profile on auth change', caught);
          });
        } else {
          profile.value = null;
        }
      });
      authSubscription = subData.subscription;

      initialized.value = true;
    })();

    try {
      await initPromise;
    } finally {
      initPromise = null;
    }
  }

  function disposeAuth() {
    authSubscription?.unsubscribe();
    authSubscription = null;
    initPromise = null;
    initialized.value = false;
  }

  return {
    user,
    profile,
    session,
    loading,
    initialized,
    signIn,
    signUp,
    signOut,
    fetchProfile,
    initAuth,
    disposeAuth,
  };
});
