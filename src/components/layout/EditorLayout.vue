<script setup lang="ts">
import { onMounted } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { SidebarProvider, SidebarInset } from "@/components/ui/sidebar";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { AlertCircle, ArrowLeft } from "lucide-vue-next";
import { useSavegameStore } from "@/stores/savegame";
import { useFinanceStore } from "@/stores/finance";
import { useVehicleStore } from "@/stores/vehicle";
import { useSaleStore } from "@/stores/sale";
import { useFieldStore } from "@/stores/field";
import { useBuildingStore } from "@/stores/building";
import { useMissionStore } from "@/stores/mission";
import { useWorldStore } from "@/stores/world";
import AppSidebar from "./AppSidebar.vue";
import AppHeader from "./AppHeader.vue";

const { t } = useI18n();
const store = useSavegameStore();
const financeStore = useFinanceStore();
const vehicleStore = useVehicleStore();
const saleStore = useSaleStore();
const fieldStore = useFieldStore();
const buildingStore = useBuildingStore();
const missionStore = useMissionStore();
const worldStore = useWorldStore();
const router = useRouter();

onMounted(async () => {
  if (!store.currentPath) {
    router.replace("/");
    return;
  }
  if (!store.currentSavegame) {
    await store.loadSavegame(store.currentPath);
  }
  if (store.currentSavegame) {
    financeStore.hydrate(store.currentSavegame);
    vehicleStore.hydrate(store.currentSavegame.vehicles);
    saleStore.hydrate(store.currentSavegame.sales);
    fieldStore.hydrate(store.currentSavegame.fields, store.currentSavegame.farmlands);
    buildingStore.hydrate(store.currentSavegame.placeables);
    missionStore.hydrate(
      store.currentSavegame.missions,
      store.currentSavegame.collectibles,
      store.currentSavegame.contractSettings,
    );
    if (store.currentSavegame.environment) {
      worldStore.hydrate(store.currentSavegame.environment);
    }
  }
});
</script>

<template>
  <SidebarProvider>
    <AppSidebar />
    <SidebarInset>
      <AppHeader />
      <main class="flex-1 overflow-auto p-6">
        <!-- Loading state -->
        <div v-if="store.isLoading" class="space-y-4">
          <Skeleton class="h-8 w-64" />
          <Skeleton class="h-32 w-full" />
          <Skeleton class="h-32 w-full" />
        </div>

        <!-- Error state -->
        <div v-else-if="store.error" class="max-w-lg space-y-4">
          <Alert variant="destructive">
            <AlertCircle class="size-4" />
            <AlertTitle>{{ t("savegame.loadingError") }}</AlertTitle>
            <AlertDescription>{{ t(store.error.code, store.error.params) }}</AlertDescription>
          </Alert>
          <Button variant="outline" @click="store.closeSavegame()">
            <ArrowLeft class="size-4" />
            {{ t("common.back") }}
          </Button>
        </div>

        <!-- Warnings -->
        <template v-else>
          <Alert
            v-if="store.warnings.length > 0"
            class="mb-4"
          >
            <AlertCircle class="size-4" />
            <AlertTitle>{{ t("savegame.warnings") }}</AlertTitle>
            <AlertDescription>
              <ul class="list-disc pl-4">
                <li v-for="(w, i) in store.warnings" :key="i">{{ t(w.code, w.params) }}</li>
              </ul>
            </AlertDescription>
          </Alert>
          <router-view />
        </template>
      </main>
    </SidebarInset>
  </SidebarProvider>
</template>
