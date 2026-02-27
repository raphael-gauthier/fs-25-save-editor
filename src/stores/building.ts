import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { Placeable, PlaceableChangePayload, ProductionStockChangePayload } from "@/lib/types";

export const useBuildingStore = defineStore("building", () => {
  const placeables = ref<Placeable[]>([]);
  const originalPlaceables = ref<Placeable[]>([]);
  const selectedIds = ref<Set<number>>(new Set());

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
      result = result.filter((p) => p.isPrePlaced);
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
      if (isProductionModified(p.productionInputs, orig.productionInputs)) count++;
      if (isProductionModified(p.productionOutputs, orig.productionOutputs)) count++;
    }
    return count;
  });

  function isPlaceableModified(p: Placeable, orig: Placeable): boolean {
    return (
      p.farmId !== orig.farmId ||
      p.price !== orig.price ||
      p.isUnderConstruction !== orig.isUnderConstruction ||
      isProductionModified(p.productionInputs, orig.productionInputs) ||
      isProductionModified(p.productionOutputs, orig.productionOutputs)
    );
  }

  function isProductionModified(
    current: Placeable["productionInputs"],
    original: Placeable["productionInputs"],
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
      for (const step of p.constructionSteps) {
        for (const mat of step.materials) {
          mat.amountRemaining = 0;
        }
      }
    }
  }

  function updateProductionStock(
    index: number,
    type: "input" | "output",
    fillType: string,
    amount: number,
  ) {
    const p = placeables.value.find((pl) => pl.index === index);
    if (!p) return;
    const stocks = type === "input" ? p.productionInputs : p.productionOutputs;
    const stock = stocks.find((s) => s.fillType === fillType);
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
      if (p.price !== orig.price) change.price = p.price;

      if (isProductionModified(p.productionInputs, orig.productionInputs)) {
        change.productionInputs = p.productionInputs
          .filter((s, i) => s.amount !== orig.productionInputs[i]?.amount)
          .map((s): ProductionStockChangePayload => ({
            fillType: s.fillType,
            amount: s.amount,
          }));
      }

      if (isProductionModified(p.productionOutputs, orig.productionOutputs)) {
        change.productionOutputs = p.productionOutputs
          .filter((s, i) => s.amount !== orig.productionOutputs[i]?.amount)
          .map((s): ProductionStockChangePayload => ({
            fillType: s.fillType,
            amount: s.amount,
          }));
      }

      changes.push(change);
    }

    return changes.length > 0 ? changes : null;
  }

  function commitChanges() {
    originalPlaceables.value = JSON.parse(JSON.stringify(placeables.value));
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
    getOriginalByIndex,
    toggleSelection,
    selectAll,
    deselectAll,
    resetChanges,
    getChanges,
    commitChanges,
  };
});
