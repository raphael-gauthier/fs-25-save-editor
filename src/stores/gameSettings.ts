import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { GameSetting } from "@/lib/types";

/** Editable game settings of careerSavegame.xml, as saved text. */
export const useGameSettingsStore = defineStore("gameSettings", () => {
  const settings = ref<GameSetting[]>([]);
  const originalSettings = ref<GameSetting[]>([]);

  const changed = computed(() =>
    settings.value.filter((s) => originalSettings.value.find((o) => o.key === s.key)?.value !== s.value),
  );
  const isDirty = computed(() => changed.value.length > 0);
  const changeCount = computed(() => changed.value.length);

  function hydrate(data: GameSetting[]) {
    settings.value = data.map((s) => ({ ...s }));
    originalSettings.value = data.map((s) => ({ ...s }));
  }

  function get(key: string): string | undefined {
    return settings.value.find((s) => s.key === key)?.value;
  }

  function set(key: string, value: string) {
    const setting = settings.value.find((s) => s.key === key);
    if (setting) setting.value = value;
  }

  function getChanges(): GameSetting[] | null {
    return changed.value.length > 0 ? changed.value.map((s) => ({ ...s })) : null;
  }

  function resetChanges() {
    settings.value = originalSettings.value.map((s) => ({ ...s }));
  }

  function commitChanges() {
    originalSettings.value = settings.value.map((s) => ({ ...s }));
  }

  return { settings, isDirty, changeCount, hydrate, get, set, getChanges, resetChanges, commitChanges };
});
