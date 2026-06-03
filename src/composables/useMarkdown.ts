import hljs from 'highlight.js';
import { marked } from 'marked';
import { ref } from 'vue';

const renderer = new marked.Renderer();

renderer.code = ({ text, lang }) => {
  const language = lang && hljs.getLanguage(lang) ? lang : 'plaintext';
  const highlighted =
    language === 'plaintext'
      ? hljs.highlightAuto(text).value
      : hljs.highlight(text, { language }).value;

  return `<pre><code class="hljs language-${language}">${highlighted}</code></pre>`;
};

marked.setOptions({
  gfm: true,
  breaks: true,
  renderer,
});

export function useMarkdown() {
  const isPreview = ref(false);

  function renderMarkdown(content: string) {
    return marked.parse(content || '') as string;
  }

  function togglePreview() {
    isPreview.value = !isPreview.value;
  }

  return { isPreview, renderMarkdown, togglePreview };
}
