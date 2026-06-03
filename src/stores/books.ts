import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { assertSupabaseConfigured, supabase } from '@/lib/supabase'
import type { Book, Chapter } from '@/types'
import { useAuthStore } from './auth'

function buildChapterTree(flatChapters: Chapter[]): Chapter[] {
  const map = new Map<string, Chapter>()
  const roots: Chapter[] = []

  for (const chapter of flatChapters) {
    map.set(chapter.id, {
      ...chapter,
      children: [],
    })
  }

  for (const chapter of flatChapters) {
    const node = map.get(chapter.id)
    if (!node) continue

    if (chapter.parent_id) {
      const parent = map.get(chapter.parent_id)
      if (parent) {
        parent.children?.push(node)
      } else {
        roots.push(node)
      }
    } else {
      roots.push(node)
    }
  }

  return roots
}

function flattenChapterTree(tree: Chapter[]): Chapter[] {
  return tree.flatMap((chapter) => [chapter, ...flattenChapterTree(chapter.children ?? [])])
}

export const useBooksStore = defineStore('books', () => {
  const books = ref<Book[]>([])
  const chapters = ref<Chapter[]>([])
  const loading = ref(false)
  const currentBookId = ref<string | null>(null)

  const currentBook = computed(() => books.value.find((book) => book.id === currentBookId.value) ?? null)

  async function fetchBooks() {
    const auth = useAuthStore()
    if (!auth.user) {
      books.value = []
      return []
    }

    assertSupabaseConfigured()
    loading.value = true

    try {
      const { data, error } = await supabase
        .from('reading_books')
        .select('*')
        .eq('user_id', auth.user.id)
        .order('created_at', { ascending: false })

      if (error) throw error

      books.value = data ?? []
      return books.value
    } finally {
      loading.value = false
    }
  }

  async function addBook(title: string, author: string) {
    const auth = useAuthStore()
    if (!auth.user) return null

    assertSupabaseConfigured()

    const { data, error } = await supabase
      .from('reading_books')
      .insert({
        user_id: auth.user.id,
        title,
        author: author || null,
      })
      .select('*')
      .single()

    if (error) throw error

    books.value = [data, ...books.value]
    currentBookId.value = data.id
    return data
  }

  async function fetchChapters(bookId: string) {
    assertSupabaseConfigured()
    loading.value = true

    try {
      const { data, error } = await supabase
        .from('reading_chapters')
        .select('*')
        .eq('book_id', bookId)
        .order('sequence_number', { ascending: true })

      if (error) throw error

      chapters.value = buildChapterTree((data ?? []) as Chapter[])
      currentBookId.value = bookId
      return chapters.value
    } finally {
      loading.value = false
    }
  }

  async function addChapter(bookId: string, title: string, sequenceNumber: number, parentId?: string) {
    assertSupabaseConfigured()

    const { error } = await supabase
      .from('reading_chapters')
      .insert({
        book_id: bookId,
        title,
        sequence_number: sequenceNumber,
        parent_id: parentId || null,
      })

    if (error) throw error
  }

  function setCurrentBook(book: Book | string | null) {
    currentBookId.value = typeof book === 'string' ? book : book?.id ?? null
  }

  function flattenChapters(tree: Chapter[] = chapters.value) {
    return flattenChapterTree(tree)
  }

  return {
    books,
    currentBook,
    chapters,
    loading,
    fetchBooks,
    addBook,
    fetchChapters,
    addChapter,
    setCurrentBook,
    flattenChapters,
  }
})
