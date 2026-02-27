import { defineStore } from "pinia";
import { ref } from "vue";
import { useRouter } from "vue-router";
import type { SavegameSummary, SavegameData, LocalizedMessage } from "@/lib/types";
import { useTauri, TauriError } from "@/composables/useTauri";

export const useSavegameStore = defineStore("savegame", () => {
  const { invokeCommand } = useTauri();
  const router = useRouter();

  const savegames = ref<SavegameSummary[]>([]);
  const currentPath = ref<string | null>(null);
  const currentSavegame = ref<SavegameData | null>(null);
  const isLoading = ref(false);
  const error = ref<LocalizedMessage | null>(null);
  const warnings = ref<LocalizedMessage[]>([]);

  async function listSavegames(customPath?: string) {
    isLoading.value = true;
    error.value = null;
    try {
      savegames.value = await invokeCommand<SavegameSummary[]>(
        "list_savegames",
        { customPath: customPath ?? null },
      );
    } catch (e: unknown) {
      if (e instanceof TauriError) {
        error.value = { code: e.code, params: e.params };
      } else {
        error.value = { code: "errors.unexpected", params: {} };
      }
      savegames.value = [];
    } finally {
      isLoading.value = false;
    }
  }

  function selectSavegame(path: string) {
    currentPath.value = path;
    currentSavegame.value = null;
    error.value = null;
    warnings.value = [];
    router.push("/editor");
  }

  async function loadSavegame(path: string) {
    isLoading.value = true;
    error.value = null;
    warnings.value = [];
    try {
      const data = await invokeCommand<SavegameData>("load_savegame", {
        path,
      });
      currentSavegame.value = data;
      warnings.value = data.warnings;
    } catch (e: unknown) {
      if (e instanceof TauriError) {
        error.value = { code: e.code, params: e.params };
      } else {
        error.value = { code: "errors.unexpected", params: {} };
      }
      currentSavegame.value = null;
    } finally {
      isLoading.value = false;
    }
  }

  async function reloadFromDisk() {
    if (currentPath.value) {
      await loadSavegame(currentPath.value);
    }
  }

  function closeSavegame() {
    currentPath.value = null;
    currentSavegame.value = null;
    warnings.value = [];
    error.value = null;
    router.push("/");
  }

  return {
    savegames,
    currentPath,
    currentSavegame,
    isLoading,
    error,
    warnings,
    listSavegames,
    selectSavegame,
    loadSavegame,
    reloadFromDisk,
    closeSavegame,
  };
});
