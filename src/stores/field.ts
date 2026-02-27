import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { Field, Farmland, FieldChangePayload, FarmlandChangePayload } from "@/lib/types";

export const useFieldStore = defineStore("field", () => {
  const fields = ref<Field[]>([]);
  const farmlands = ref<Farmland[]>([]);
  const originalFields = ref<Field[]>([]);
  const originalFarmlands = ref<Farmland[]>([]);
  const selectedFieldIds = ref<Set<number>>(new Set());

  // Filters
  const searchQuery = ref("");
  const fruitFilter = ref<string | null>(null);
  const ownerFilter = ref<number | null>(null);

  // Getters
  const filteredFields = computed(() => {
    let result = fields.value;

    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase();
      result = result.filter(
        (f) =>
          f.id.toString().includes(query) ||
          f.fruitType.toLowerCase().includes(query) ||
          f.plannedFruit.toLowerCase().includes(query),
      );
    }

    if (fruitFilter.value) {
      result = result.filter(
        (f) => f.fruitType === fruitFilter.value || f.plannedFruit === fruitFilter.value,
      );
    }

    if (ownerFilter.value !== null) {
      const farmlandMap = new Map(farmlands.value.map((fl) => [fl.id, fl.farmId]));
      result = result.filter((f) => farmlandMap.get(f.id) === ownerFilter.value);
    }

    return result.sort((a, b) => a.id - b.id);
  });

  const availableFruits = computed(() => {
    const fruits = new Set<string>();
    for (const f of fields.value) {
      if (f.fruitType && f.fruitType !== "UNKNOWN") fruits.add(f.fruitType);
    }
    return Array.from(fruits).sort();
  });

  const isDirty = computed(() => {
    if (isFieldsDirty()) return true;
    if (isFarmlandsDirty()) return true;
    return false;
  });

  const changeCount = computed(() => {
    let count = 0;

    const origFieldMap = new Map(originalFields.value.map((f) => [f.id, f]));
    for (const f of fields.value) {
      const orig = origFieldMap.get(f.id);
      if (!orig) continue;
      if (f.fruitType !== orig.fruitType) count++;
      if (f.plannedFruit !== orig.plannedFruit) count++;
      if (f.growthState !== orig.growthState) count++;
      if (f.groundType !== orig.groundType) count++;
      if (f.weedState !== orig.weedState) count++;
      if (f.stoneLevel !== orig.stoneLevel) count++;
      if (f.sprayLevel !== orig.sprayLevel) count++;
      if (f.limeLevel !== orig.limeLevel) count++;
      if (f.plowLevel !== orig.plowLevel) count++;
      if (f.rollerLevel !== orig.rollerLevel) count++;
      if (f.stubbleShredLevel !== orig.stubbleShredLevel) count++;
      if (f.waterLevel !== orig.waterLevel) count++;
    }

    const origFarmlandMap = new Map(originalFarmlands.value.map((fl) => [fl.id, fl]));
    for (const fl of farmlands.value) {
      const orig = origFarmlandMap.get(fl.id);
      if (orig && fl.farmId !== orig.farmId) count++;
    }

    return count;
  });

  function isFieldsDirty(): boolean {
    const origMap = new Map(originalFields.value.map((f) => [f.id, f]));
    return fields.value.some((f) => {
      const orig = origMap.get(f.id);
      if (!orig) return false;
      return isFieldModified(f, orig);
    });
  }

  function isFarmlandsDirty(): boolean {
    const origMap = new Map(originalFarmlands.value.map((fl) => [fl.id, fl]));
    return farmlands.value.some((fl) => {
      const orig = origMap.get(fl.id);
      return orig !== undefined && fl.farmId !== orig.farmId;
    });
  }

  function isFieldModified(f: Field, orig: Field): boolean {
    return (
      f.fruitType !== orig.fruitType ||
      f.plannedFruit !== orig.plannedFruit ||
      f.growthState !== orig.growthState ||
      f.groundType !== orig.groundType ||
      f.weedState !== orig.weedState ||
      f.stoneLevel !== orig.stoneLevel ||
      f.sprayLevel !== orig.sprayLevel ||
      f.limeLevel !== orig.limeLevel ||
      f.plowLevel !== orig.plowLevel ||
      f.rollerLevel !== orig.rollerLevel ||
      f.stubbleShredLevel !== orig.stubbleShredLevel ||
      f.waterLevel !== orig.waterLevel
    );
  }

  // Actions
  function hydrate(fieldData: Field[], farmlandData: Farmland[]) {
    const fieldsSerialized = JSON.stringify(fieldData);
    const farmlandsSerialized = JSON.stringify(farmlandData);
    fields.value = JSON.parse(fieldsSerialized);
    farmlands.value = JSON.parse(farmlandsSerialized);
    originalFields.value = JSON.parse(fieldsSerialized);
    originalFarmlands.value = JSON.parse(farmlandsSerialized);
    selectedFieldIds.value = new Set();
  }

  function updateField(id: number, changes: Partial<Field>) {
    const field = fields.value.find((f) => f.id === id);
    if (field) {
      Object.assign(field, changes);
    }
  }

  function updateFarmland(id: number, farmId: number) {
    const farmland = farmlands.value.find((fl) => fl.id === id);
    if (farmland) {
      farmland.farmId = farmId;
    }
  }

  function getFieldById(id: number): Field | undefined {
    return fields.value.find((f) => f.id === id);
  }

  function getOriginalFieldById(id: number): Field | undefined {
    return originalFields.value.find((f) => f.id === id);
  }

  function getFarmlandByFieldId(fieldId: number): Farmland | undefined {
    return farmlands.value.find((fl) => fl.id === fieldId);
  }

  // Batch actions
  function batchMaxGrowth(fieldIds: number[]) {
    for (const id of fieldIds) {
      updateField(id, { growthState: 10 });
    }
  }

  function batchRemoveWeeds(fieldIds: number[]) {
    for (const id of fieldIds) {
      updateField(id, { weedState: 0 });
    }
  }

  function batchRemoveStones(fieldIds: number[]) {
    for (const id of fieldIds) {
      updateField(id, { stoneLevel: 0 });
    }
  }

  function batchMaxLime(fieldIds: number[]) {
    for (const id of fieldIds) {
      updateField(id, { limeLevel: 3 });
    }
  }

  function batchMaxFertilizer(fieldIds: number[]) {
    for (const id of fieldIds) {
      updateField(id, { sprayLevel: 2 });
    }
  }

  // Selection
  function toggleSelection(id: number) {
    if (selectedFieldIds.value.has(id)) {
      selectedFieldIds.value.delete(id);
    } else {
      selectedFieldIds.value.add(id);
    }
    selectedFieldIds.value = new Set(selectedFieldIds.value);
  }

  function selectAll() {
    selectedFieldIds.value = new Set(filteredFields.value.map((f) => f.id));
  }

  function deselectAll() {
    selectedFieldIds.value = new Set();
  }

  function resetChanges() {
    fields.value = JSON.parse(JSON.stringify(originalFields.value));
    farmlands.value = JSON.parse(JSON.stringify(originalFarmlands.value));
  }

  function getChanges(): { fields?: FieldChangePayload[]; farmlands?: FarmlandChangePayload[] } | null {
    const fieldChanges: FieldChangePayload[] = [];
    const farmlandChanges: FarmlandChangePayload[] = [];

    const origFieldMap = new Map(originalFields.value.map((f) => [f.id, f]));
    for (const f of fields.value) {
      const orig = origFieldMap.get(f.id);
      if (!orig || !isFieldModified(f, orig)) continue;

      const change: FieldChangePayload = { id: f.id };
      if (f.fruitType !== orig.fruitType) change.fruitType = f.fruitType;
      if (f.plannedFruit !== orig.plannedFruit) change.plannedFruit = f.plannedFruit;
      if (f.growthState !== orig.growthState) change.growthState = f.growthState;
      if (f.groundType !== orig.groundType) change.groundType = f.groundType;
      if (f.weedState !== orig.weedState) change.weedState = f.weedState;
      if (f.stoneLevel !== orig.stoneLevel) change.stoneLevel = f.stoneLevel;
      if (f.sprayLevel !== orig.sprayLevel) change.sprayLevel = f.sprayLevel;
      if (f.limeLevel !== orig.limeLevel) change.limeLevel = f.limeLevel;
      if (f.plowLevel !== orig.plowLevel) change.plowLevel = f.plowLevel;
      if (f.rollerLevel !== orig.rollerLevel) change.rollerLevel = f.rollerLevel;
      if (f.stubbleShredLevel !== orig.stubbleShredLevel) change.stubbleShredLevel = f.stubbleShredLevel;
      if (f.waterLevel !== orig.waterLevel) change.waterLevel = f.waterLevel;
      fieldChanges.push(change);
    }

    const origFarmlandMap = new Map(originalFarmlands.value.map((fl) => [fl.id, fl]));
    for (const fl of farmlands.value) {
      const orig = origFarmlandMap.get(fl.id);
      if (orig && fl.farmId !== orig.farmId) {
        farmlandChanges.push({ id: fl.id, farmId: fl.farmId });
      }
    }

    if (fieldChanges.length === 0 && farmlandChanges.length === 0) return null;

    const result: { fields?: FieldChangePayload[]; farmlands?: FarmlandChangePayload[] } = {};
    if (fieldChanges.length > 0) result.fields = fieldChanges;
    if (farmlandChanges.length > 0) result.farmlands = farmlandChanges;
    return result;
  }

  function commitChanges() {
    originalFields.value = JSON.parse(JSON.stringify(fields.value));
    originalFarmlands.value = JSON.parse(JSON.stringify(farmlands.value));
  }

  return {
    fields,
    farmlands,
    selectedFieldIds,
    searchQuery,
    fruitFilter,
    ownerFilter,
    filteredFields,
    availableFruits,
    isDirty,
    changeCount,
    hydrate,
    updateField,
    updateFarmland,
    getFieldById,
    getOriginalFieldById,
    getFarmlandByFieldId,
    batchMaxGrowth,
    batchRemoveWeeds,
    batchRemoveStones,
    batchMaxLime,
    batchMaxFertilizer,
    toggleSelection,
    selectAll,
    deselectAll,
    resetChanges,
    getChanges,
    commitChanges,
  };
});
