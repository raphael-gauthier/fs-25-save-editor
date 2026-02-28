import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { UpdateInfo } from "@/lib/types";

const updateAvailable = ref<UpdateInfo | null>(null);
const isChecking = ref(false);
const lastCheckError = ref<string | null>(null);

export function useUpdateChecker() {
  async function checkForUpdates(): Promise<UpdateInfo | null> {
    if (isChecking.value) return null;

    isChecking.value = true;
    lastCheckError.value = null;

    try {
      const result = await invoke<UpdateInfo | null>("check_for_updates");
      updateAvailable.value = result;
      return result;
    } catch (error) {
      lastCheckError.value = String(error);
      console.error("Update check failed:", error);
      return null;
    } finally {
      isChecking.value = false;
    }
  }

  function dismissUpdate() {
    updateAvailable.value = null;
  }

  return {
    updateAvailable,
    isChecking,
    lastCheckError,
    checkForUpdates,
    dismissUpdate,
  };
}
