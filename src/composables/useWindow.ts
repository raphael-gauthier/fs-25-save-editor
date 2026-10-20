import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

/** False in a plain browser (vite dev without Tauri). */
export const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

const isMaximized = ref(false);
let listening = false;

/**
 * Puts Mica behind the transparent window, tinted to the theme. Where the
 * OS has no Mica the page keeps painting its own wallpaper.
 */
export async function applyWindowMaterial(dark: boolean) {
  const html = document.documentElement;
  if (!inTauri) {
    html.classList.remove("has-mica");
    return;
  }
  try {
    const applied = await invoke<boolean>("apply_window_material", { dark });
    html.classList.toggle("has-mica", applied);
  } catch {
    html.classList.remove("has-mica");
  }
}

export function useWindowControls() {
  if (inTauri && !listening) {
    listening = true;
    const appWindow = getCurrentWindow();
    appWindow.isMaximized().then((v) => (isMaximized.value = v)).catch(() => {});
    appWindow
      .onResized(async () => {
        isMaximized.value = await appWindow.isMaximized();
      })
      .catch(() => {});
  }

  function minimize() {
    if (inTauri) getCurrentWindow().minimize();
  }

  function toggleMaximize() {
    if (inTauri) getCurrentWindow().toggleMaximize();
  }

  function close() {
    if (inTauri) getCurrentWindow().close();
  }

  return { isMaximized, minimize, toggleMaximize, close };
}
