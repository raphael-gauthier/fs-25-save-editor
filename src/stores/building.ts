import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  Placeable,
  PlaceableChangePayload,
  ProductionStockChangePayload,
  SiloStorageKind,
  StorageCapacities,
} from "@/lib/types";
import { groupObjects, objectStorageChange, siloStockChanges } from "@/lib/storageChanges";

export const useBuildingStore = defineStore("building", () => {
  const placeables = ref<Placeable[]>([]);
  const originalPlaceables = ref<Placeable[]>([]);
  const selectedIds = ref<Set<number>>(new Set());
  /** Capacities and accepted fill types of the storages, from the placeables' XML */
  const storageCapacities = ref<StorageCapacities>({ silos: [], objectStorages: [] });
  /** Placeables whose fermenting bales are to finish fermenting */
  const fermentationToFinish = ref<Set<number>>(new Set());

  // Filters
  const searchQuery = ref("");
  const ownerFilter = ref<number | null>(null);
  const stateFilter = ref<string | null>(null);

  // Getters
  const filteredPlaceables = computed(() => {
    let result = placeables.value;

    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase();
      result = result.filter(
        (p) =>
          p.displayName.toLowerCase().includes(query) ||
          p.filename.toLowerCase().includes(query),
      );
    }

    if (ownerFilter.value !== null) {
      result = result.filter((p) => p.farmId === ownerFilter.value);
    }

    if (stateFilter.value === "ready") {
      result = result.filter((p) => !p.isUnderConstruction && !p.isPrePlaced);
    } else if (stateFilter.value === "underConstruction") {
      result = result.filter((p) => p.isUnderConstruction);
    } else if (stateFilter.value === "prePlaced") {
      result = result.filter((p) => p.isPrePlaced && !p.isDeleted);
    } else if (stateFilter.value === "deleted") {
      result = result.filter((p) => p.isDeleted);
    }

    return result.sort((a, b) => a.index - b.index);
  });

  const playerBuildings = computed(() =>
    placeables.value.filter((p) => p.farmId > 0),
  );

  const underConstruction = computed(() =>
    placeables.value.filter((p) => p.isUnderConstruction),
  );

  const isDirty = computed(() => {
    const origMap = new Map(originalPlaceables.value.map((p) => [p.index, p]));
    return placeables.value.some((p) => {
      const orig = origMap.get(p.index);
      if (!orig) return false;
      return isPlaceableModified(p, orig);
    });
  });

  const changeCount = computed(() => {
    let count = 0;
    const origMap = new Map(originalPlaceables.value.map((p) => [p.index, p]));
    for (const p of placeables.value) {
      const orig = origMap.get(p.index);
      if (!orig) continue;
      if (p.farmId !== orig.farmId) count++;
      if (p.price !== orig.price) count++;
      if (p.isUnderConstruction !== orig.isUnderConstruction) count++;
      if (p.isDeleted !== orig.isDeleted) count++;
      if (isProductionModified(p.productionStorage, orig.productionStorage)) count++;
      count += siloStockChanges(p.siloStorages, orig.siloStorages).length;
      if (objectChange(p, orig)) count++;
    }
    return count;
  });

  function objectChange(p: Placeable, orig: Placeable) {
    return objectStorageChange(p.storedObjects, orig.storedObjects, fermentationToFinish.value.has(p.index));
  }

  function isPlaceableModified(p: Placeable, orig: Placeable): boolean {
    return (
      p.farmId !== orig.farmId ||
      p.price !== orig.price ||
      p.isUnderConstruction !== orig.isUnderConstruction ||
      p.isDeleted !== orig.isDeleted ||
      isProductionModified(p.productionStorage, orig.productionStorage) ||
      siloStockChanges(p.siloStorages, orig.siloStorages).length > 0 ||
      objectChange(p, orig) !== null
    );
  }

  function isProductionModified(
    current: Placeable["productionStorage"],
    original: Placeable["productionStorage"],
  ): boolean {
    if (current.length !== original.length) return true;
    return current.some((s, i) => s.amount !== original[i].amount);
  }

  // Actions
  function hydrate(data: Placeable[]) {
    const serialized = JSON.stringify(data);
    placeables.value = JSON.parse(serialized);
    originalPlaceables.value = JSON.parse(serialized);
    selectedIds.value = new Set();
    fermentationToFinish.value = new Set();
  }

  async function loadStorageCapacities(savegamePath: string, gamePath: string, mapId: string) {
    try {
      storageCapacities.value = await invoke<StorageCapacities>("load_storage_capacities", {
        savegamePath,
        gamePath,
        mapId,
      });
    } catch {
      // Without capacities, levels can still be edited; the game clamps them on load
      storageCapacities.value = { silos: [], objectStorages: [] };
    }
  }

  /** Sets a silo storage's fill level, adding the fill type if missing; 0 empties it. */
  function setSiloStock(index: number, kind: SiloStorageKind, storageIndex: number, fillType: string, amount: number) {
    const storage = placeables.value
      .find((pl) => pl.index === index)
      ?.siloStorages.find((s) => s.kind === kind && s.index === storageIndex);
    if (!storage) return;
    const stock = storage.stocks.find((s) => s.fillType === fillType);
    if (stock) stock.amount = Math.max(0, amount);
    else storage.stocks.push({ fillType, amount: Math.max(0, amount) });
  }

  /** Adds copies of a group of identical stored objects, or removes its last ones. */
  function changeObjectCount(index: number, groupKey: string, delta: number) {
    const p = placeables.value.find((pl) => pl.index === index);
    const group = p && groupObjects(p.storedObjects).find((g) => g.key === groupKey);
    if (!p || !group) return;
    if (delta > 0) {
      for (let i = 0; i < delta; i++) p.storedObjects.push({ ...group.object });
    } else {
      const removed = new Set(group.indices.slice(delta));
      p.storedObjects = p.storedObjects.filter((_, i) => !removed.has(i));
    }
  }

  /** Brings back a deleted pre-placed building on the next load. */
  function restorePlaceable(index: number) {
    const p = placeables.value.find((pl) => pl.index === index);
    if (p) p.isDeleted = false;
  }

  function finishFermentation(index: number) {
    const p = placeables.value.find((pl) => pl.index === index);
    if (!p) return;
    fermentationToFinish.value = new Set(fermentationToFinish.value).add(index);
  }

  function updatePlaceable(index: number, changes: Partial<Placeable>) {
    const p = placeables.value.find((pl) => pl.index === index);
    if (p) {
      Object.assign(p, changes);
    }
  }

  function completeConstruction(index: number) {
    const p = placeables.value.find((pl) => pl.index === index);
    if (p) {
      p.isUnderConstruction = false;
      for (const input of p.construction?.inputs ?? []) {
        input.amountRemaining = 0;
      }
    }
  }

  function updateProductionStock(index: number, fillType: string, amount: number) {
    const p = placeables.value.find((pl) => pl.index === index);
    if (!p) return;
    const stock = p.productionStorage.find((s) => s.fillType === fillType);
    if (stock) {
      stock.amount = amount;
    }
  }

  function getOriginalByIndex(index: number): Placeable | undefined {
    return originalPlaceables.value.find((p) => p.index === index);
  }

  // Selection
  function toggleSelection(index: number) {
    if (selectedIds.value.has(index)) {
      selectedIds.value.delete(index);
    } else {
      selectedIds.value.add(index);
    }
    selectedIds.value = new Set(selectedIds.value);
  }

  function selectAll() {
    selectedIds.value = new Set(filteredPlaceables.value.map((p) => p.index));
  }

  function deselectAll() {
    selectedIds.value = new Set();
  }

  function resetChanges() {
    placeables.value = JSON.parse(JSON.stringify(originalPlaceables.value));
    fermentationToFinish.value = new Set();
  }

  function getChanges(): PlaceableChangePayload[] | null {
    const changes: PlaceableChangePayload[] = [];

    const origMap = new Map(originalPlaceables.value.map((p) => [p.index, p]));
    for (const p of placeables.value) {
      const orig = origMap.get(p.index);
      if (!orig || !isPlaceableModified(p, orig)) continue;

      const change: PlaceableChangePayload = {
        index: p.index,
        completeConstruction: orig.isUnderConstruction && !p.isUnderConstruction,
      };
      if (p.farmId !== orig.farmId) change.farmId = p.farmId;
      if (orig.isDeleted && !p.isDeleted) change.restore = true;
      if (p.price !== orig.price) change.price = p.price;

      if (isProductionModified(p.productionStorage, orig.productionStorage)) {
        change.productionStorage = p.productionStorage
          .filter((s, i) => s.amount !== orig.productionStorage[i]?.amount)
          .map((s): ProductionStockChangePayload => ({
            fillType: s.fillType,
            amount: s.amount,
          }));
      }

      const siloStocks = siloStockChanges(p.siloStorages, orig.siloStorages);
      if (siloStocks.length > 0) change.siloStocks = siloStocks;
      const objectStorage = objectChange(p, orig);
      if (objectStorage) change.objectStorage = objectStorage;

      changes.push(change);
    }

    return changes.length > 0 ? changes : null;
  }

  function commitChanges() {
    originalPlaceables.value = JSON.parse(JSON.stringify(placeables.value));
    fermentationToFinish.value = new Set();
  }

  return {
    placeables,
    selectedIds,
    searchQuery,
    ownerFilter,
    stateFilter,
    filteredPlaceables,
    playerBuildings,
    underConstruction,
    isDirty,
    changeCount,
    hydrate,
    updatePlaceable,
    completeConstruction,
    updateProductionStock,
    storageCapacities,
    fermentationToFinish,
    loadStorageCapacities,
    setSiloStock,
    changeObjectCount,
    finishFermentation,
    restorePlaceable,
    getOriginalByIndex,
    toggleSelection,
    selectAll,
    deselectAll,
    resetChanges,
    getChanges,
    commitChanges,
  };
});
