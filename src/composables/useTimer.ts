import { onUnmounted, ref } from 'vue';

export function useTimer() {
  const seconds = ref(0);
  const isRunning = ref(false);
  let interval: ReturnType<typeof setInterval> | null = null;

  function start() {
    if (isRunning.value) return;

    isRunning.value = true;
    interval = setInterval(() => {
      seconds.value += 1;
    }, 1000);
  }

  function pause() {
    isRunning.value = false;
    if (interval) {
      clearInterval(interval);
      interval = null;
    }
  }

  function reset() {
    pause();
    seconds.value = 0;
  }

  function formatTime(totalSeconds: number) {
    const minutes = Math.floor(totalSeconds / 60)
      .toString()
      .padStart(2, '0');
    const remainingSeconds = (totalSeconds % 60).toString().padStart(2, '0');

    return `${minutes}:${remainingSeconds}`;
  }

  onUnmounted(() => {
    if (interval) {
      clearInterval(interval);
    }
  });

  return { seconds, isRunning, start, pause, reset, formatTime };
}
