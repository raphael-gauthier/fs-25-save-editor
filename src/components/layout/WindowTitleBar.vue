<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { Moon, Sun } from "@lucide/vue";
import { useSettingsStore } from "@/stores/settings";
import WindowControls from "./WindowControls.vue";
import WithTooltip from "@/components/WithTooltip.vue";

const { t } = useI18n();
const settings = useSettingsStore();

function isDark() {
  return document.documentElement.classList.contains("dark");
}

function toggleTheme() {
  settings.setTheme(isDark() ? "light" : "dark");
}
</script>

<template>
  <!-- The bar and its empty spaces drag the window; double-click maximizes -->
  <header
    data-tauri-drag-region
    class="relative z-20 flex h-12 shrink-0 select-none items-center gap-3 pl-4"
  >
    <slot name="leading">
      <span
        data-tauri-drag-region
        class="flex items-center gap-2.5 text-sm font-semibold"
      >
        <span
          class="inline-flex size-6 items-center justify-center rounded-[7px] bg-primary text-primary-foreground"
          aria-hidden="true"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 21V9" /><path d="M12 9c0-3 2-5 5-5 0 3-2 5-5 5Z" /><path d="M12 9c0-3-2-5-5-5 0 3 2 5 5 5Z" />
            <path d="M12 15c0-3 2-5 5-5 0 3-2 5-5 5Z" /><path d="M12 15c0-3-2-5-5-5 0 3 2 5 5 5Z" />
          </svg>
        </span>
        {{ t("common.appName") }}
      </span>
    </slot>

    <slot />

    <div data-tauri-drag-region class="ml-auto flex h-full items-center gap-1.5">
      <slot name="actions" />
      <WithTooltip :label="t('window.toggleTheme')">
        <button
          type="button"
          class="inline-flex size-9 items-center justify-center rounded-lg text-foreground transition-colors hover:bg-hover"
          :aria-label="t('window.toggleTheme')"
          @click="toggleTheme"
        >
          <Sun class="size-4 dark:hidden" aria-hidden="true" />
          <Moon class="hidden size-4 dark:block" aria-hidden="true" />
        </button>
      </WithTooltip>
      <WindowControls />
    </div>
  </header>
</template>
