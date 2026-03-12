import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { load } from "@tauri-apps/plugin-store";
import type {
  Field,
  Farmland,
  FieldChangePayload,
  FarmlandChangePayload,
  FieldDensityData,
  DensityEditPayload,
} from "@/lib/types";

const DENSITY_CACHE_FILE = "density-cache.json";

interface DensityCacheEntry {
  data: FieldDensityData[];
  timestamp: number;
  mapId: string;
}

function cacheKey(savegamePath: string): string {
  // Use the savegame folder name as key (e.g., "savegame1")
  const parts = savegamePath.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || parts[parts.length - 2] || savegamePath;
}

export const useFieldStore = defineStore("field", () => {
  const fields = ref<Field[]>([]);
  const farmlands = ref<Farmland[]>([]);
  const originalFields = ref<Field[]>([]);
  const originalFarmlands = ref<Farmland[]>([]);
  const selectedFieldIds = ref<Set<number>>(new Set());

  // Density map data
  const densityData = ref<FieldDensityData[]>([]);
  const densityLoading = ref(false);
  const densityError = ref<string | null>(null);
  const densityFromCache = ref(false);

  // Density edit tracking
  const densityEdits = ref<Map<number, DensityEditPayload>>(new Map());

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
      if (hasDensityData.value) {
        result = result.filter((f) => {
          const d = densityMap.value.get(f.id);
          if (!d) return false;
          return d.dominantFruit === fruitFilter.value ||
            d.fruitDistribution.some((fc) => fc.fruitType === fruitFilter.value);
        });
      } else {
        result = result.filter(
          (f) => f.fruitType === fruitFilter.value || f.plannedFruit === fruitFilter.value,
        );
      }
    }

    if (ownerFilter.value !== null) {
      const farmlandMap = new Map(farmlands.value.map((fl) => [fl.id, fl.farmId]));
      result = result.filter((f) => farmlandMap.get(f.id) === ownerFilter.value);
    }

    return result.sort((a, b) => a.id - b.id);
  });

  const hasDensityData = computed(() => densityData.value.length > 0);

  const densityMap = computed(() => {
    const map = new Map<number, FieldDensityData>();
    for (const d of densityData.value) {
      map.set(d.farmlandId, d);
    }
    return map;
  });

  function getFieldDensity(farmlandId: number): FieldDensityData | undefined {
    return densityMap.value.get(farmlandId);
  }

  const availableFruits = computed(() => {
    const fruits = new Set<string>();
    if (hasDensityData.value) {
      for (const d of densityData.value) {
        if (d.dominantFruit) fruits.add(d.dominantFruit);
        for (const fc of d.fruitDistribution) {
          fruits.add(fc.fruitType);
        }
      }
    } else {
      for (const f of fields.value) {
        if (f.fruitType && f.fruitType !== "UNKNOWN") fruits.add(f.fruitType);
      }
    }
    return Array.from(fruits).sort();
  });

  const hasDensityEdits = computed(() => densityEdits.value.size > 0);

  const densityEditCount = computed(() => {
    let count = 0;
    for (const edit of densityEdits.value.values()) {
      if (edit.setFruitName !== undefined) count++;
      if (edit.setLimeLevel !== undefined) count++;
      if (edit.setSprayLevel !== undefined) count++;
      if (edit.setPlowLevel !== undefined) count++;
      if (edit.setRollerLevel !== undefined) count++;
      if (edit.setStubbleShredLevel !== undefined) count++;
      if (edit.clearWeeds) count++;
      if (edit.clearStones) count++;
    }
    return count;
  });

  const isDirty = computed(() => {
    if (isFieldsDirty()) return true;
    if (isFarmlandsDirty()) return true;
    if (hasDensityEdits.value) return true;
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

    count += densityEditCount.value;

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

  function addDensityEdit(farmlandId: number, edit: Partial<DensityEditPayload>) {
    const existing = densityEdits.value.get(farmlandId) ?? { farmlandId };
    const merged = { ...existing, ...edit };
    densityEdits.value.set(farmlandId, merged);
    densityEdits.value = new Map(densityEdits.value);
  }

  function batchDensityEdit(farmlandIds: number[], edit: Partial<DensityEditPayload>) {
    for (const id of farmlandIds) {
      addDensityEdit(id, edit);
    }
  }

  function getDensityEdits(): DensityEditPayload[] {
    return Array.from(densityEdits.value.values());
  }

  async function saveDensityEdits(savegamePath: string, gamePath: string, mapId: string): Promise<string[]> {
    const edits = getDensityEdits();
    if (edits.length === 0) return [];
    const result = await invoke<string[]>("save_density_edits", {
      savegamePath,
      gamePath,
      mapId,
      edits,
    });
    densityEdits.value = new Map();
    return result;
  }

  function resetChanges() {
    fields.value = JSON.parse(JSON.stringify(originalFields.value));
    farmlands.value = JSON.parse(JSON.stringify(originalFarmlands.value));
    densityEdits.value = new Map();
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

    if (fieldChanges.length === 0 && farmlandChanges.length === 0 && !hasDensityEdits.value)
      return null;

    const result: { fields?: FieldChangePayload[]; farmlands?: FarmlandChangePayload[] } = {};
    if (fieldChanges.length > 0) result.fields = fieldChanges;
    if (farmlandChanges.length > 0) result.farmlands = farmlandChanges;
    return result;
  }

  function commitChanges() {
    originalFields.value = JSON.parse(JSON.stringify(fields.value));
    originalFarmlands.value = JSON.parse(JSON.stringify(farmlands.value));
  }

  async function loadDensityData(savegamePath: string, gamePath: string, mapId: string) {
    densityLoading.value = true;
    densityError.value = null;
    densityFromCache.value = false;

    // Try loading from cache first
    const key = cacheKey(savegamePath);
    try {
      const store = await load(DENSITY_CACHE_FILE);
      const cached = await store.get<DensityCacheEntry>(key);
      if (cached && cached.mapId === mapId && cached.data.length > 0) {
        densityData.value = cached.data;
        densityFromCache.value = true;
      }
    } catch {
      // Cache miss — continue to load from backend
    }

    // Refresh from backend
    try {
      const data = await invoke<FieldDensityData[]>("load_field_density_data", {
        savegamePath,
        gamePath,
        mapId,
      });
      densityData.value = data;
      densityFromCache.value = false;

      // Update cache
      try {
        const store = await load(DENSITY_CACHE_FILE);
        await store.set(key, { data, timestamp: Date.now(), mapId } as DensityCacheEntry);
        await store.save();
      } catch {
        // Cache write failure is non-critical
      }
    } catch (error: unknown) {
      const err = error as { code?: string; params?: { message?: string } };
      densityError.value = err?.params?.message || String(error);
      // Keep cached data if available, otherwise clear
      if (!densityFromCache.value) {
        densityData.value = [];
      }
    } finally {
      densityLoading.value = false;
    }
  }

  function clearDensityData() {
    densityData.value = [];
    densityError.value = null;
    densityLoading.value = false;
    densityFromCache.value = false;
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
    // Density map data
    densityData,
    densityLoading,
    densityError,
    densityFromCache,
    hasDensityData,
    getFieldDensity,
    loadDensityData,
    clearDensityData,
    // Density edits
    densityEdits,
    hasDensityEdits,
    densityEditCount,
    addDensityEdit,
    batchDensityEdit,
    getDensityEdits,
    saveDensityEdits,
    // Actions
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
