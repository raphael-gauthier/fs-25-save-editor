import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type {
  Economy,
  GreatDemand,
  FillTypePrice,
  EconomyChanges,
  GreatDemandChangePayload,
  GreatDemandAdditionPayload,
} from "@/lib/types";

export const useEconomyStore = defineStore("economy", () => {
  const greatDemands = ref<GreatDemand[]>([]);
  const originalGreatDemands = ref<GreatDemand[]>([]);
  const fillTypes = ref<FillTypePrice[]>([]);

  // Additions and deletions tracked separately
  const additions = ref<GreatDemandAdditionPayload[]>([]);
  const deletions = ref<Set<number>>(new Set());

  // Filters
  const priceSearchQuery = ref("");
  const showAllProducts = ref(false);

  // Getters
  const filteredFillTypes = computed(() => {
    let result = fillTypes.value;

    if (!showAllProducts.value) {
      result = result.filter(
        (ft) =>
          ft.totalAmount !== null &&
          ft.totalAmount > 0 &&
          ft.priceHistory.length > 0,
      );
    }

    if (priceSearchQuery.value) {
      const query = priceSearchQuery.value.toLowerCase();
      result = result.filter((ft) =>
        ft.fillType.toLowerCase().includes(query),
      );
    }

    return result;
  });

  const activeGreatDemands = computed(() =>
    greatDemands.value.filter((d) => !deletions.value.has(d.index)),
  );

  const isDirty = computed(() => {
    if (additions.value.length > 0) return true;
    if (deletions.value.size > 0) return true;
    return isGreatDemandsDirty();
  });

  const changeCount = computed(() => {
    let count = additions.value.length + deletions.value.size;
    const origMap = new Map(originalGreatDemands.value.map((d) => [d.index, d]));
    for (const d of greatDemands.value) {
      const orig = origMap.get(d.index);
      if (!orig) continue;
      if (d.fillTypeName !== orig.fillTypeName) count++;
      else if (d.demandMultiplier !== orig.demandMultiplier) count++;
      else if (d.demandStartDay !== orig.demandStartDay) count++;
      else if (d.demandStartHour !== orig.demandStartHour) count++;
      else if (d.demandDuration !== orig.demandDuration) count++;
      else if (d.isRunning !== orig.isRunning) count++;
      else if (d.isValid !== orig.isValid) count++;
    }
    return count;
  });

  function isGreatDemandsDirty(): boolean {
    const origMap = new Map(originalGreatDemands.value.map((d) => [d.index, d]));
    return greatDemands.value.some((d) => {
      const orig = origMap.get(d.index);
      if (!orig) return false;
      return (
        d.fillTypeName !== orig.fillTypeName ||
        d.demandMultiplier !== orig.demandMultiplier ||
        d.demandStartDay !== orig.demandStartDay ||
        d.demandStartHour !== orig.demandStartHour ||
        d.demandDuration !== orig.demandDuration ||
        d.isRunning !== orig.isRunning ||
        d.isValid !== orig.isValid
      );
    });
  }

  // Actions
  function hydrate(data: Economy | null) {
    if (!data) {
      greatDemands.value = [];
      originalGreatDemands.value = [];
      fillTypes.value = [];
      additions.value = [];
      deletions.value = new Set();
      return;
    }
    const demandsSerialized = JSON.stringify(data.greatDemands);
    greatDemands.value = JSON.parse(demandsSerialized);
    originalGreatDemands.value = JSON.parse(demandsSerialized);
    fillTypes.value = data.fillTypes;
    additions.value = [];
    deletions.value = new Set();
  }

  function updateDemand(index: number, changes: Partial<GreatDemand>) {
    const d = greatDemands.value.find((dem) => dem.index === index);
    if (d) {
      Object.assign(d, changes);
    }
  }

  function deleteDemand(index: number) {
    deletions.value = new Set([...deletions.value, index]);
  }

  function addDemand(addition: GreatDemandAdditionPayload) {
    additions.value = [...additions.value, addition];
  }

  function resetChanges() {
    greatDemands.value = JSON.parse(JSON.stringify(originalGreatDemands.value));
    additions.value = [];
    deletions.value = new Set();
  }

  function getChanges(): EconomyChanges | null {
    const result: EconomyChanges = {};
    let hasChanges = false;

    // Modified demands
    const demandChanges: GreatDemandChangePayload[] = [];
    const origMap = new Map(originalGreatDemands.value.map((d) => [d.index, d]));
    for (const d of greatDemands.value) {
      const orig = origMap.get(d.index);
      if (!orig) continue;
      const changed =
        d.fillTypeName !== orig.fillTypeName ||
        d.demandMultiplier !== orig.demandMultiplier ||
        d.demandStartDay !== orig.demandStartDay ||
        d.demandStartHour !== orig.demandStartHour ||
        d.demandDuration !== orig.demandDuration ||
        d.isRunning !== orig.isRunning ||
        d.isValid !== orig.isValid;
      if (!changed) continue;

      const change: GreatDemandChangePayload = { index: d.index };
      if (d.fillTypeName !== orig.fillTypeName) change.fillTypeName = d.fillTypeName;
      if (d.demandMultiplier !== orig.demandMultiplier) change.demandMultiplier = d.demandMultiplier;
      if (d.demandStartDay !== orig.demandStartDay) change.demandStartDay = d.demandStartDay;
      if (d.demandStartHour !== orig.demandStartHour) change.demandStartHour = d.demandStartHour;
      if (d.demandDuration !== orig.demandDuration) change.demandDuration = d.demandDuration;
      if (d.isRunning !== orig.isRunning) change.isRunning = d.isRunning;
      if (d.isValid !== orig.isValid) change.isValid = d.isValid;
      demandChanges.push(change);
    }
    if (demandChanges.length > 0) {
      result.greatDemandChanges = demandChanges;
      hasChanges = true;
    }

    // Additions
    if (additions.value.length > 0) {
      result.greatDemandAdditions = [...additions.value];
      hasChanges = true;
    }

    // Deletions
    if (deletions.value.size > 0) {
      result.greatDemandDeletions = [...deletions.value];
      hasChanges = true;
    }

    return hasChanges ? result : null;
  }

  function commitChanges() {
    originalGreatDemands.value = JSON.parse(JSON.stringify(greatDemands.value));
    additions.value = [];
    deletions.value = new Set();
  }

  return {
    greatDemands,
    fillTypes,
    additions,
    deletions,
    priceSearchQuery,
    showAllProducts,
    filteredFillTypes,
    activeGreatDemands,
    isDirty,
    changeCount,
    hydrate,
    updateDemand,
    deleteDemand,
    addDemand,
    resetChanges,
    getChanges,
    commitChanges,
  };
});
