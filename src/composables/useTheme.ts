import { ref, watch } from "vue";

export type ThemeMode = "light" | "dark" | "system";

const theme = ref<ThemeMode>("system");
let mediaQuery: MediaQueryList | null = null;
let mediaListener: ((e: MediaQueryListEvent) => void) | null = null;

function applyTheme(mode: ThemeMode) {
  const html = document.documentElement;
  if (mode === "system") {
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    html.classList.toggle("dark", prefersDark);
  } else {
    html.classList.toggle("dark", mode === "dark");
  }
}

function setupMediaListener() {
  if (mediaListener && mediaQuery) {
    mediaQuery.removeEventListener("change", mediaListener);
  }
  mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  mediaListener = () => {
    if (theme.value === "system") {
      applyTheme("system");
    }
  };
  mediaQuery.addEventListener("change", mediaListener);
}

export function useTheme() {
  function setTheme(mode: ThemeMode) {
    theme.value = mode;
    applyTheme(mode);
  }

  function initTheme(mode: ThemeMode) {
    theme.value = mode;
    applyTheme(mode);
    setupMediaListener();
  }

  watch(theme, (newTheme) => {
    applyTheme(newTheme);
  });

  return { theme, setTheme, initTheme };
}
