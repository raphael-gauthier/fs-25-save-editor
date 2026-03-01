<script setup lang="ts">
import { computed, ref } from "vue";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";
import { useEventListener } from "@vueuse/core";
import { toast } from "vue-sonner";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Save, Loader2, RotateCcw, Undo2 } from "lucide-vue-next";
import { useSavegameStore } from "@/stores/savegame";
import { useFinanceStore } from "@/stores/finance";
import { useVehicleStore } from "@/stores/vehicle";
import { useSaleStore } from "@/stores/sale";
import { useFieldStore } from "@/stores/field";
import { useBuildingStore } from "@/stores/building";
import { useMissionStore } from "@/stores/mission";
import { useWorldStore } from "@/stores/world";
import { useEconomyStore } from "@/stores/economy";
import { useTauri, translateError } from "@/composables/useTauri";
import { useUnsavedChanges } from "@/composables/useUnsavedChanges";
import type { SaveResult } from "@/lib/types";

const { t } = useI18n();
const route = useRoute();
const savegameStore = useSavegameStore();
const financeStore = useFinanceStore();
const vehicleStore = useVehicleStore();
const saleStore = useSaleStore();
const fieldStore = useFieldStore();
const buildingStore = useBuildingStore();
const missionStore = useMissionStore();
const worldStore = useWorldStore();
const economyStore = useEconomyStore();
const { invokeCommand } = useTauri();
const { isDirty, changeCount, collectAllChanges, confirmDiscardIfDirty } =
  useUnsavedChanges();

const isSaving = ref(false);
const isReloading = ref(false);

// Global keyboard shortcuts
useEventListener(document, "keydown", (e: KeyboardEvent) => {
  const mod = e.ctrlKey || e.metaKey;
  if (!mod) return;

  if (e.key === "s") {
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
  fields: "sidebar.fields",
  world: "sidebar.world",
  buildings: "sidebar.buildings",
  missions: "sidebar.missions",
  collectibles: "sidebar.collectibles",
  economy: "sidebar.economy",
  settings: "sidebar.settings",
  backups: "sidebar.backups",
};

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

  if (!Object.values(changes).some(Boolean)) {
    toast.info(t("savegame.noChanges"));
    return;
  }

  isSaving.value = true;
  try {
    const result = await invokeCommand<SaveResult>("save_changes", {
      path: savegameStore.currentPath,
      changes,
    });

    if (result.success) {
      financeStore.commitChanges();
      vehicleStore.commitChanges();
      saleStore.commitChanges();
      fieldStore.commitChanges();
      buildingStore.commitChanges();
      missionStore.commitChanges();
      worldStore.commitChanges();
      economyStore.commitChanges();
      toast.success(t("savegame.saveSuccess"), {
        description: t("savegame.saveSuccessDesc", {
          count: result.filesModified.length,
        }),
      });
    } else {
      toast.error(t("savegame.saveError"), {
        description: result.errors.map(e => t(e.code, e.params)).join(", "),
      });
    }
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
    financeStore.resetChanges();
    vehicleStore.resetChanges();
    saleStore.resetChanges();
    fieldStore.resetChanges();
    worldStore.resetChanges();
    economyStore.resetChanges();
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
      if (savegameStore.currentSavegame.environment) {
        worldStore.hydrate(savegameStore.currentSavegame.environment);
      }
      economyStore.hydrate(savegameStore.currentSavegame.economy);
    }
  } finally {
    isReloading.value = false;
  }
}
</script>

<template>
  <header class="flex h-14 items-center gap-2 border-b px-4">
    <SidebarTrigger class="-ml-1" />
    <Separator orientation="vertical" class="mr-2 !h-4" />

    <Breadcrumb>
      <BreadcrumbList>
        <BreadcrumbItem>
          <BreadcrumbLink as-child>
            <router-link to="/">{{ t("common.savegames") }}</router-link>
          </BreadcrumbLink>
        </BreadcrumbItem>
        <template v-for="(crumb, i) in breadcrumbs" :key="i">
          <BreadcrumbSeparator />
          <BreadcrumbItem>
            <BreadcrumbLink v-if="crumb.to" as-child>
              <router-link :to="crumb.to">{{ crumb.label }}</router-link>
            </BreadcrumbLink>
            <BreadcrumbPage v-else>{{ crumb.label }}</BreadcrumbPage>
          </BreadcrumbItem>
        </template>
      </BreadcrumbList>
    </Breadcrumb>

    <div class="ml-auto flex items-center gap-2">
      <Button
        v-if="isDirty"
        variant="ghost"
        size="sm"
        :disabled="isReloading"
        @click="handleReload"
      >
        <RotateCcw class="mr-2 size-4" :class="{ 'animate-spin': isReloading }" />
        {{ t("dirtyTracking.reloadFromDisk") }}
      </Button>

      <Button
        v-if="isDirty"
        variant="ghost"
        size="sm"
        @click="handleDiscard"
      >
        <Undo2 class="mr-2 size-4" />
        {{ t("dirtyTracking.discardChanges") }}
      </Button>

      <Button
        variant="outline"
        size="sm"
        :disabled="!isDirty || isSaving"
        @click="handleSave"
      >
        <Loader2 v-if="isSaving" class="mr-2 size-4 animate-spin" />
        <Save v-else class="mr-2 size-4" />
        {{ t("common.save") }}
        <Badge v-if="changeCount > 0" variant="secondary" class="ml-2">
          {{ changeCount }}
        </Badge>
      </Button>
    </div>
  </header>
</template>
