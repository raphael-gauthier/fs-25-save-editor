import { ref, shallowRef } from "vue";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { inTauri } from "@/composables/useWindow";

/** A newer release found on GitHub, signed with the app's updater key. */
const updateAvailable = shallowRef<Update | null>(null);
const isChecking = ref(false);
const lastCheckError = ref<string | null>(null);
/** Download share from 0 to 1 while installing, null otherwise. */
const progress = ref<number | null>(null);

export function useUpdateChecker() {
  async function checkForUpdates(): Promise<Update | null> {
    if (!inTauri || isChecking.value) return null;

    isChecking.value = true;
    lastCheckError.value = null;

    try {
      const update = await check();
      updateAvailable.value = update;
      return update;
    } catch (error) {
      lastCheckError.value = String(error);
      console.error("Update check failed:", error);
      return null;
    } finally {
      isChecking.value = false;
    }
  }

  /** Downloads, verifies and installs the update, then restarts the app. */
  async function installUpdate() {
    const update = updateAvailable.value;
    if (!update || progress.value !== null) return;
    let total = 0;
    let received = 0;
    progress.value = 0;
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") total = event.data.contentLength ?? 0;
        if (event.event === "Progress") {
          received += event.data.chunkLength;
          progress.value = total ? Math.min(1, received / total) : null;
        }
        if (event.event === "Finished") progress.value = 1;
      });
      await relaunch();
    } catch (error) {
      lastCheckError.value = String(error);
      progress.value = null;
      throw error;
    }
  }

  function dismissUpdate() {
    if (progress.value === null) updateAvailable.value = null;
  }

  return {
    updateAvailable,
    isChecking,
    lastCheckError,
    progress,
    checkForUpdates,
    installUpdate,
    dismissUpdate,
  };
}
