<script setup lang="ts">
import { computed, ref } from "vue";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";
import { useEventListener } from "@vueuse/core";
import { toast } from "vue-sonner";
import { Button } from "@/components/ui/button";
import { ChevronRight, RotateCcw, Search } from "@lucide/vue";
import { Badge } from "@/components/ui/badge";
import { Kbd } from "@/components/ui/kbd";
import { Spinner } from "@/components/ui/spinner";
import { useSavegameStore } from "@/stores/savegame";
import { useFinanceStore } from "@/stores/finance";
import { useVehicleStore } from "@/stores/vehicle";
import { useSaleStore } from "@/stores/sale";
import { useFieldStore } from "@/stores/field";
import { useBuildingStore } from "@/stores/building";
import { useAnimalStore } from "@/stores/animal";
import { useMissionStore } from "@/stores/mission";
import { useWorldStore } from "@/stores/world";
import { useEconomyStore } from "@/stores/economy";
import { useGameSettingsStore } from "@/stores/gameSettings";
import { useSettingsStore } from "@/stores/settings";
import { useBackupStore } from "@/stores/backup";
import { useTauri, translateError } from "@/composables/useTauri";
import { useUnsavedChanges } from "@/composables/useUnsavedChanges";
import type { SaveResult } from "@/lib/types";
import WindowTitleBar from "./WindowTitleBar.vue";
import CommandPalette from "./CommandPalette.vue";
import WithTooltip from "@/components/WithTooltip.vue";

const { t } = useI18n();
const route = useRoute();
const savegameStore = useSavegameStore();
const financeStore = useFinanceStore();
const vehicleStore = useVehicleStore();
const saleStore = useSaleStore();
const fieldStore = useFieldStore();
const buildingStore = useBuildingStore();
const animalStore = useAnimalStore();
const missionStore = useMissionStore();
const worldStore = useWorldStore();
const economyStore = useEconomyStore();
const gameSettingsStore = useGameSettingsStore();
const settings = useSettingsStore();
const backupStore = useBackupStore();
const { invokeCommand } = useTauri();
const { isDirty, changeCount, collectAllChanges, confirmDiscardIfDirty, resetAll } =
  useUnsavedChanges();

const isSaving = ref(false);
const isReloading = ref(false);
const paletteOpen = ref(false);

// Global keyboard shortcuts
useEventListener(document, "keydown", (e: KeyboardEvent) => {
  const mod = e.ctrlKey || e.metaKey;
  if (!mod) return;

  if (e.key === "k") {
    e.preventDefault();
    paletteOpen.value = !paletteOpen.value;
  } else if (e.key === "s") {
    e.preventDefault();
    if (isDirty.value) handleSave();
  } else if (e.key === "z" && !e.shiftKey) {
    e.preventDefault();
    if (isDirty.value) handleDiscard();
  }
});

const breadcrumbKeyMap: Record<string, string> = {
  finance: "sidebar.finance",
  vehicles: "sidebar.vehicles",
  "vehicle-detail": "sidebar.vehicles",
  sales: "sidebar.sales",
  "sale-add": "sidebar.sales",
  fields: "sidebar.fields",
  farmlands: "sidebar.farmlands",
  storage: "sidebar.storage",
  game: "sidebar.game",
  world: "sidebar.world",
  buildings: "sidebar.buildings",
  animals: "sidebar.animals",
  missions: "sidebar.missions",
  collectibles: "sidebar.collectibles",
  economy: "sidebar.economy",
  settings: "sidebar.settings",
  backups: "sidebar.backups",
};

const savegameLabel = computed(
  () => savegameStore.currentSavegame?.career.mapTitle || t("common.savegames"),
);

const breadcrumbs = computed(() => {
  const crumbs: { label: string; to?: string }[] = [];

  if (route.name && typeof route.name === "string") {
    const key = breadcrumbKeyMap[route.name];
    const label = key ? t(key) : route.name;
    crumbs.push({ label });
  }

  return crumbs;
});

async function handleSave() {
  if (!savegameStore.currentPath || isSaving.value) return;

  const changes = collectAllChanges();
  const hasDensityEdits = fieldStore.hasDensityEdits;

  if (!Object.values(changes).some(Boolean) && !hasDensityEdits) {
    toast.info(t("savegame.noChanges"));
    return;
  }

  isSaving.value = true;
  try {
    let totalFilesModified = 0;
    const hasXmlChanges = Object.values(changes).some(Boolean);

    // Save XML changes (save_changes backs the savegame up first)
    if (hasXmlChanges) {
      const result = await invokeCommand<SaveResult>("save_changes", {
        path: savegameStore.currentPath,
        changes,
      });

      if (!result.success) {
        toast.error(t("savegame.saveError"), {
          description: result.errors.map(e => t(e.code, e.params)).join(", "),
        });
        return;
      }
      totalFilesModified += result.filesModified.length;
    }

    // Save density map edits
    if (hasDensityEdits && settings.gamePath && savegameStore.currentSavegame?.career.mapId) {
      const densityFiles = await fieldStore.saveDensityEdits(
        savegameStore.currentPath,
        settings.gamePath,
        savegameStore.currentSavegame.career.mapId,
        !hasXmlChanges,
      );
      totalFilesModified += densityFiles.length;
    }

    // Commit all store changes
    financeStore.commitChanges();
    vehicleStore.commitChanges();
    saleStore.commitChanges();
    fieldStore.commitChanges();
    buildingStore.commitChanges();
    animalStore.commitChanges();
    missionStore.commitChanges();
    worldStore.commitChanges();
    economyStore.commitChanges();
    gameSettingsStore.commitChanges();

    // Every save made a backup: keep only as many as the settings allow
    await backupStore.prune();

    // Reload density data to reflect edits
    if (hasDensityEdits && settings.gamePath && savegameStore.currentSavegame?.career.mapId) {
      fieldStore.loadDensityData(
        savegameStore.currentPath,
        settings.gamePath,
        savegameStore.currentSavegame.career.mapId,
      );
    }

    toast.success(t("savegame.saveSuccess"), {
      description: t("savegame.saveSuccessDesc", {
        count: totalFilesModified,
      }),
    });
  } catch (e: unknown) {
    toast.error(t("savegame.saveError"), {
      description: translateError(t, e),
    });
  } finally {
    isSaving.value = false;
  }
}

async function handleDiscard() {
  const confirmed = await confirmDiscardIfDirty();
  if (confirmed) {
    resetAll();
  }
}

async function handleReload() {
  if (isDirty.value) {
    const confirmed = await confirmDiscardIfDirty();
    if (!confirmed) return;
  }
  isReloading.value = true;
  try {
    await savegameStore.reloadFromDisk();
    if (savegameStore.currentSavegame) {
      financeStore.hydrate(savegameStore.currentSavegame);
      vehicleStore.hydrate(savegameStore.currentSavegame.vehicles);
      saleStore.hydrate(savegameStore.currentSavegame.sales);
      fieldStore.hydrate(savegameStore.currentSavegame.fields, savegameStore.currentSavegame.farmlands);
      buildingStore.hydrate(savegameStore.currentSavegame.placeables);
      missionStore.hydrate(
        savegameStore.currentSavegame.missions,
        savegameStore.currentSavegame.collectibles,
        savegameStore.currentSavegame.contractSettings,
      );
      animalStore.hydrate(savegameStore.currentSavegame.husbandries);
      if (savegameStore.currentSavegame.environment) {
        worldStore.hydrate(savegameStore.currentSavegame.environment);
      }
      economyStore.hydrate(savegameStore.currentSavegame.economy);
      gameSettingsStore.hydrate(savegameStore.currentSavegame.gameSettings);
    }
  } finally {
    isReloading.value = false;
  }
}
</script>

<template>
  <WindowTitleBar>
    <template #leading>
      <WithTooltip :label="t('common.savegames')">
        <router-link
          to="/"
          class="inline-flex size-6 shrink-0 items-center justify-center rounded-[7px] bg-primary text-primary-foreground"
          :aria-label="t('common.savegames')"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M12 21V9" /><path d="M12 9c0-3 2-5 5-5 0 3-2 5-5 5Z" /><path d="M12 9c0-3-2-5-5-5 0 3 2 5 5 5Z" />
            <path d="M12 15c0-3 2-5 5-5 0 3-2 5-5 5Z" /><path d="M12 15c0-3-2-5-5-5 0 3 2 5 5 5Z" />
          </svg>
        </router-link>
      </WithTooltip>
    </template>

    <nav
      :aria-label="t('window.breadcrumb')"
      data-tauri-drag-region
      class="flex min-w-0 items-center gap-1.5 text-[13px]"
    >
      <router-link to="/" class="truncate text-muted-foreground hover:text-foreground">
        {{ savegameLabel }}
      </router-link>
      <template v-for="(crumb, i) in breadcrumbs" :key="i">
        <ChevronRight class="size-3.5 shrink-0 text-muted-foreground" aria-hidden="true" />
        <span aria-current="page" class="truncate font-semibold">{{ crumb.label }}</span>
      </template>
    </nav>

    <button
      type="button"
      class="mx-auto flex h-[34px] w-full max-w-[400px] min-w-0 items-center gap-2.5 rounded-[10px] border bg-glass pl-3 pr-2 text-[13px] text-muted-foreground transition-colors hover:bg-hover"
      :aria-label="t('palette.title')"
      aria-keyshortcuts="Control+K"
      @click="paletteOpen = true"
    >
      <Search class="size-[15px] shrink-0" aria-hidden="true" />
      <span class="flex-1 truncate text-left">{{ t("palette.placeholder") }}</span>
      <Kbd>Ctrl K</Kbd>
    </button>

    <CommandPalette
      v-model:open="paletteOpen"
      :actions="{ save: handleSave, discard: handleDiscard, reload: handleReload, canSave: isDirty }"
    />

    <template #actions>
      <Badge v-if="isDirty" role="status" variant="warning" class="gap-1.5 px-2.5 py-1 text-[13px]">
        <span class="size-[7px] rounded-[2px] bg-current" aria-hidden="true" />
        {{ t("dirtyTracking.pendingCount", { count: changeCount }, changeCount) }}
      </Badge>
      <WithTooltip :label="t('dirtyTracking.reloadFromDisk')">
        <Button
          v-if="isDirty"
          variant="ghost"
          size="icon"
          class="size-9"
          :disabled="isReloading"
          :aria-label="t('dirtyTracking.reloadFromDisk')"
          @click="handleReload"
        >
          <RotateCcw class="size-4" :class="{ 'animate-spin': isReloading }" />
        </Button>
      </WithTooltip>
      <Button v-if="isDirty" variant="ghost" size="sm" class="h-9" @click="handleDiscard">
        {{ t("dirtyTracking.discardChanges") }}
      </Button>
      <Button size="sm" class="h-9 gap-2" :disabled="!isDirty || isSaving" @click="handleSave">
        <Spinner v-if="isSaving" />
        {{ t("common.save") }}
        <Kbd class="bg-primary-foreground/20 text-primary-foreground">Ctrl S</Kbd>
      </Button>
    </template>
  </WindowTitleBar>
</template>
