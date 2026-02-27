import { defineStore } from "pinia";
import { ref } from "vue";
import { load } from "@tauri-apps/plugin-store";
import { useTheme, type ThemeMode } from "@/composables/useTheme";
import i18n from "@/plugins/i18n";

const STORE_FILE = "settings.json";

export const useSettingsStore = defineStore("settings", () => {
  const locale = ref<string>("fr");
  const theme = ref<ThemeMode>("system");
  const advancedMode = ref(false);
  const disclaimerAccepted = ref(false);
  const defaultPath = ref<string>("");
  const maxBackups = ref(10);

  const { initTheme, setTheme: applyTheme } = useTheme();

  async function loadSettings() {
    try {
      const store = await load(STORE_FILE);
      const savedLocale = await store.get<string>("locale");
      const savedTheme = await store.get<ThemeMode>("theme");
      const savedAdvanced = await store.get<boolean>("advancedMode");
      const savedDisclaimer = await store.get<boolean>("disclaimerAccepted");
      const savedPath = await store.get<string>("defaultPath");
      const savedMaxBackups = await store.get<number>("maxBackups");

      if (savedLocale) {
        locale.value = savedLocale;
      } else {
        // Detect system locale on first run
        const sysLang = navigator.language.toLowerCase();
        locale.value = sysLang.startsWith("fr") ? "fr" : "en";
      }

      theme.value = savedTheme ?? "system";
      advancedMode.value = savedAdvanced ?? false;
      disclaimerAccepted.value = savedDisclaimer ?? false;
      defaultPath.value = savedPath ?? "";
      maxBackups.value = savedMaxBackups ?? 10;

      // Apply locale and theme
      i18n.global.locale.value = locale.value as "fr" | "en";
      initTheme(theme.value);
    } catch {
      // First run or store error — use defaults
      const sysLang = navigator.language.toLowerCase();
      locale.value = sysLang.startsWith("fr") ? "fr" : "en";
      i18n.global.locale.value = locale.value as "fr" | "en";
      initTheme("system");
    }
  }

  async function persist() {
    try {
      const store = await load(STORE_FILE);
      await store.set("locale", locale.value);
      await store.set("theme", theme.value);
      await store.set("advancedMode", advancedMode.value);
      await store.set("disclaimerAccepted", disclaimerAccepted.value);
      await store.set("defaultPath", defaultPath.value);
      await store.set("maxBackups", maxBackups.value);
      await store.save();
    } catch {
      // Silently fail — settings are still in memory
    }
  }

  async function setLocale(newLocale: string) {
    locale.value = newLocale;
    i18n.global.locale.value = newLocale as "fr" | "en";
    await persist();
  }

  async function setTheme(newTheme: ThemeMode) {
    theme.value = newTheme;
    applyTheme(newTheme);
    await persist();
  }

  async function toggleAdvancedMode() {
    advancedMode.value = !advancedMode.value;
    await persist();
  }

  async function setAdvancedMode(value: boolean) {
    advancedMode.value = value;
    await persist();
  }

  async function setDefaultPath(path: string) {
    defaultPath.value = path;
    await persist();
  }

  async function acceptDisclaimer() {
    disclaimerAccepted.value = true;
    await persist();
  }

  async function setMaxBackups(count: number) {
    maxBackups.value = count;
    await persist();
  }

  return {
    locale,
    theme,
    advancedMode,
    disclaimerAccepted,
    defaultPath,
    maxBackups,
    loadSettings,
    setLocale,
    setTheme,
    toggleAdvancedMode,
    setAdvancedMode,
    acceptDisclaimer,
    setDefaultPath,
    setMaxBackups,
  };
});
