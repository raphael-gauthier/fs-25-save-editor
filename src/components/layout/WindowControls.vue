<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { inTauri, useWindowControls } from "@/composables/useWindow";
import { useUnsavedChanges } from "@/composables/useUnsavedChanges";
import WithTooltip from "@/components/WithTooltip.vue";

const { t } = useI18n();
const { isMaximized, minimize, toggleMaximize, close } = useWindowControls();
const { confirmDiscardIfDirty } = useUnsavedChanges();

async function closeWindow() {
  if (await confirmDiscardIfDirty()) close();
}
</script>

<template>
  <div v-if="inTauri" class="flex self-stretch">
    <WithTooltip :label="t('window.minimize')">
      <button
        type="button"
        class="inline-flex w-[46px] items-center justify-center text-foreground transition-colors hover:bg-hover"
        :aria-label="t('window.minimize')"
        @click="minimize"
      >
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" aria-hidden="true">
          <path d="M0 5h10" />
        </svg>
      </button>
    </WithTooltip>
    <WithTooltip :label="isMaximized ? t('window.restore') : t('window.maximize')">
      <button
        type="button"
        class="inline-flex w-[46px] items-center justify-center text-foreground transition-colors hover:bg-hover"
        :aria-label="isMaximized ? t('window.restore') : t('window.maximize')"
        @click="toggleMaximize"
      >
        <svg v-if="isMaximized" width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" aria-hidden="true">
          <rect x="0.5" y="2.5" width="7" height="7" rx="1" />
          <path d="M2.5 2.5V1.5a1 1 0 0 1 1-1h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1h-1" />
        </svg>
        <svg v-else width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" aria-hidden="true">
          <rect x="0.5" y="0.5" width="9" height="9" rx="1.5" />
        </svg>
      </button>
    </WithTooltip>
    <WithTooltip :label="t('window.close')">
      <button
        type="button"
        class="inline-flex w-[46px] items-center justify-center text-foreground transition-colors hover:bg-[#C42B1C] hover:text-white"
        :aria-label="t('window.close')"
        @click="closeWindow"
      >
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" aria-hidden="true">
          <path d="m0.5 0.5 9 9M9.5 0.5l-9 9" />
        </svg>
      </button>
    </WithTooltip>
  </div>
</template>
