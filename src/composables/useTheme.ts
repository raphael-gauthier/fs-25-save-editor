import { ref, watch } from "vue";
import { applyWindowMaterial } from "@/composables/useWindow";

export type ThemeMode = "light" | "dark" | "system";

const theme = ref<ThemeMode>("dark");
let mediaQuery: MediaQueryList | null = null;
let mediaListener: ((e: MediaQueryListEvent) => void) | null = null;

function applyTheme(mode: ThemeMode) {
  const dark =
    mode === "system" ? window.matchMedia("(prefers-color-scheme: dark)").matches : mode === "dark";
  document.documentElement.classList.toggle("dark", dark);
  applyWindowMaterial(dark);
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
