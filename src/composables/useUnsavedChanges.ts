import { computed, ref } from "vue";
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
import type { SavegameChanges } from "@/lib/types";

const showDiscardDialog = ref(false);
let discardResolve: ((value: boolean) => void) | null = null;

export function useUnsavedChanges() {
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

  const isDirty = computed(
    () =>
      financeStore.isDirty ||
      vehicleStore.isDirty ||
      saleStore.isDirty ||
      fieldStore.isDirty ||
      buildingStore.isDirty ||
      animalStore.isDirty ||
      missionStore.isDirty ||
      worldStore.isDirty ||
      economyStore.isDirty ||
      gameSettingsStore.isDirty,
  );

  const changeCount = computed(
    () =>
      financeStore.changeCount +
      vehicleStore.changeCount +
      saleStore.changeCount +
      fieldStore.changeCount +
      buildingStore.changeCount +
      animalStore.changeCount +
      missionStore.changeCount +
      worldStore.changeCount +
      economyStore.changeCount +
      gameSettingsStore.changeCount,
  );

  function collectAllChanges(): SavegameChanges {
    const changes: SavegameChanges = {};
    const financeChanges = financeStore.getChanges();
    if (financeChanges) changes.finance = financeChanges;
    const vehicleChanges = vehicleStore.getChanges();
    if (vehicleChanges) changes.vehicles = vehicleChanges;
    const saleResult = saleStore.getChanges();
    if (saleResult.sales) changes.sales = saleResult.sales;
    if (saleResult.saleAdditions) changes.saleAdditions = saleResult.saleAdditions;
    const fieldChanges = fieldStore.getChanges();
    if (fieldChanges) {
      if (fieldChanges.fields) changes.fields = fieldChanges.fields;
      if (fieldChanges.farmlands) changes.farmlands = fieldChanges.farmlands;
    }
    const buildingChanges = buildingStore.getChanges();
    if (buildingChanges) changes.placeables = buildingChanges;
    const animalChanges = animalStore.getChanges();
    if (animalChanges) changes.husbandries = animalChanges;
    const missionChanges = missionStore.getChanges();
    if (missionChanges) {
      if (missionChanges.missions) changes.missions = missionChanges.missions;
      if (missionChanges.collectibles) changes.collectibles = missionChanges.collectibles;
      if (missionChanges.contractSettings) changes.contractSettings = missionChanges.contractSettings;
    }
    const envChanges = worldStore.getChanges();
    if (envChanges) changes.environment = envChanges;
    const economyChanges = economyStore.getChanges();
    if (economyChanges) changes.economy = economyChanges;
    const gameSettings = gameSettingsStore.getChanges();
    if (gameSettings) changes.gameSettings = gameSettings;
    return changes;
  }

  function resetAll() {
    financeStore.resetChanges();
    vehicleStore.resetChanges();
    saleStore.resetChanges();
    fieldStore.resetChanges();
    buildingStore.resetChanges();
    animalStore.resetChanges();
    missionStore.resetChanges();
    worldStore.resetChanges();
    economyStore.resetChanges();
    gameSettingsStore.resetChanges();
  }

  function confirmDiscardIfDirty(): Promise<boolean> {
    if (!isDirty.value) return Promise.resolve(true);

    return new Promise((resolve) => {
      discardResolve = resolve;
      showDiscardDialog.value = true;
    });
  }

  function resolveDiscard(confirmed: boolean) {
    showDiscardDialog.value = false;
    if (discardResolve) {
      discardResolve(confirmed);
      discardResolve = null;
    }
  }

  return {
    isDirty,
    changeCount,
    showDiscardDialog,
    collectAllChanges,
    resetAll,
    confirmDiscardIfDirty,
    resolveDiscard,
  };
}
