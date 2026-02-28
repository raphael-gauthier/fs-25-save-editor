import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { Vehicle } from "@/lib/types";

export interface VehicleChangePayload {
  uniqueId: string;
  delete: boolean;
  age?: number;
  price?: number;
  farmId?: number;
  propertyState?: string;
  operatingTime?: number;
  damage?: number;
  wear?: number;
  fillUnits?: { index: number; fillLevel: number }[];
}

export const useVehicleStore = defineStore("vehicle", () => {
  const vehicles = ref<Vehicle[]>([]);
  const originalVehicles = ref<Vehicle[]>([]);
  const selectedVehicleIds = ref<Set<string>>(new Set());
  const deletedVehicleIds = ref<Set<string>>(new Set());

  // Filtres
  const searchQuery = ref("");
  const typeFilter = ref<string | null>(null);
  const propertyStateFilter = ref<string | null>(null);

  // Getters
  const playerVehicles = computed(() =>
    vehicles.value.filter((v) => v.farmId > 0),
  );

  const filteredVehicles = computed(() => {
    let result = playerVehicles.value;

    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase();
      result = result.filter((v) =>
        v.displayName.toLowerCase().includes(query),
      );
    }

    if (typeFilter.value) {
      result = result.filter((v) => {
        const type = vehicleTypeFromFilename(v.filename);
        return type === typeFilter.value;
      });
    }

    if (propertyStateFilter.value) {
      result = result.filter(
        (v) => v.propertyState === propertyStateFilter.value,
      );
    }

    return result;
  });

  const availableTypes = computed(() => {
    const types = new Set<string>();
    for (const v of playerVehicles.value) {
      types.add(vehicleTypeFromFilename(v.filename));
    }
    return Array.from(types).sort();
  });

  const isDirty = computed(() => {
    if (deletedVehicleIds.value.size > 0) return true;
    if (vehicles.value.length !== originalVehicles.value.length) return true;

    const origMap = new Map(
      originalVehicles.value.filter((v) => v.uniqueId).map((v) => [v.uniqueId, v]),
    );
    return vehicles.value.some((v) => {
      if (!v.uniqueId) return false;
      const orig = origMap.get(v.uniqueId);
      if (!orig) return true;
      return isVehicleModified(v, orig);
    });
  });

  const changeCount = computed(() => {
    let count = deletedVehicleIds.value.size;
    const origMap = new Map(
      originalVehicles.value.filter((v) => v.uniqueId).map((v) => [v.uniqueId, v]),
    );
    for (const v of vehicles.value) {
      if (!v.uniqueId) continue;
      const orig = origMap.get(v.uniqueId);
      if (!orig) {
        count++;
        continue;
      }
      if (v.price !== orig.price) count++;
      if (v.operatingTime !== orig.operatingTime) count++;
      if (v.age !== orig.age) count++;
      if (v.propertyState !== orig.propertyState) count++;
      if (Math.abs(v.damage - orig.damage) > 0.001) count++;
      if (Math.abs(v.wear - orig.wear) > 0.001) count++;
      for (const unit of v.fillUnits) {
        const origUnit = orig.fillUnits.find((u) => u.index === unit.index);
        if (origUnit && Math.abs(unit.fillLevel - origUnit.fillLevel) > 0.001) count++;
      }
    }
    return count;
  });

  function getVehicleById(id: string): Vehicle | undefined {
    return vehicles.value.find((v) => v.uniqueId === id);
  }

  function getOriginalVehicleById(id: string): Vehicle | undefined {
    return originalVehicles.value.find((v) => v.uniqueId === id);
  }

  // Actions
  function hydrate(data: Vehicle[]) {
    const serialized = JSON.stringify(data);
    vehicles.value = JSON.parse(serialized);
    originalVehicles.value = JSON.parse(serialized);
    selectedVehicleIds.value = new Set();
    deletedVehicleIds.value = new Set();
  }

  function updateVehicle(id: string, changes: Partial<Vehicle>) {
    const vehicle = vehicles.value.find((v) => v.uniqueId === id);
    if (vehicle) {
      Object.assign(vehicle, changes);
    }
  }

  function updateFillLevel(vehicleId: string, unitIndex: number, level: number) {
    const vehicle = vehicles.value.find((v) => v.uniqueId === vehicleId);
    if (vehicle) {
      const unit = vehicle.fillUnits.find((u) => u.index === unitIndex);
      if (unit) {
        unit.fillLevel = Math.max(0, level);
      }
    }
  }

  function fillAllTanks(vehicleId: string) {
    const vehicle = vehicles.value.find((v) => v.uniqueId === vehicleId);
    if (vehicle) {
      for (const unit of vehicle.fillUnits) {
        if (unit.capacity !== null && unit.capacity > 0) {
          unit.fillLevel = unit.capacity;
        }
      }
    }
  }

  function emptyAllTanks(vehicleId: string) {
    const vehicle = vehicles.value.find((v) => v.uniqueId === vehicleId);
    if (vehicle) {
      for (const unit of vehicle.fillUnits) {
        unit.fillLevel = 0;
      }
    }
  }

  function resetVehicleAge(vehicleId: string) {
    const vehicle = vehicles.value.find((v) => v.uniqueId === vehicleId);
    if (vehicle) {
      vehicle.age = 0;
      vehicle.operatingTime = 0;
    }
  }

  function deleteVehicle(vehicleId: string) {
    deletedVehicleIds.value.add(vehicleId);
    deletedVehicleIds.value = new Set(deletedVehicleIds.value);
    vehicles.value = vehicles.value.filter((v) => v.uniqueId !== vehicleId);
  }

  // Batch actions
  function batchFillAll(vehicleIds: string[]) {
    for (const id of vehicleIds) {
      fillAllTanks(id);
    }
  }

  function batchResetAge(vehicleIds: string[]) {
    for (const id of vehicleIds) {
      resetVehicleAge(id);
    }
  }

  function batchChangeOwner(vehicleIds: string[], farmId: number) {
    for (const id of vehicleIds) {
      const vehicle = vehicles.value.find((v) => v.uniqueId === id);
      if (vehicle) {
        vehicle.farmId = farmId;
      }
    }
  }

  function selectVehicle(id: string) {
    selectedVehicleIds.value.add(id);
    selectedVehicleIds.value = new Set(selectedVehicleIds.value);
  }

  function deselectVehicle(id: string) {
    selectedVehicleIds.value.delete(id);
    selectedVehicleIds.value = new Set(selectedVehicleIds.value);
  }

  function toggleSelection(id: string) {
    if (selectedVehicleIds.value.has(id)) {
      deselectVehicle(id);
    } else {
      selectVehicle(id);
    }
  }

  function selectAll() {
    selectedVehicleIds.value = new Set(
      filteredVehicles.value.map((v) => v.uniqueId),
    );
  }

  function deselectAll() {
    selectedVehicleIds.value = new Set();
  }

  function resetChanges() {
    vehicles.value = JSON.parse(JSON.stringify(originalVehicles.value));
    deletedVehicleIds.value = new Set();
  }

  function getChanges(): VehicleChangePayload[] | null {
    const changes: VehicleChangePayload[] = [];

    // Deleted vehicles
    for (const id of deletedVehicleIds.value) {
      changes.push({ uniqueId: id, delete: true });
    }

    // Modified vehicles
    const origMap = new Map(
      originalVehicles.value.filter((v) => v.uniqueId).map((v) => [v.uniqueId, v]),
    );
    for (const v of vehicles.value) {
      if (!v.uniqueId) continue;
      const orig = origMap.get(v.uniqueId);
      if (!orig || !isVehicleModified(v, orig)) continue;

      const change: VehicleChangePayload = { uniqueId: v.uniqueId, delete: false };
      if (v.age !== orig.age) change.age = v.age;
      if (v.price !== orig.price) change.price = v.price;
      if (v.operatingTime !== orig.operatingTime) change.operatingTime = v.operatingTime;
      if (v.propertyState !== orig.propertyState) change.propertyState = v.propertyState;
      if (Math.abs(v.damage - orig.damage) > 0.001) change.damage = v.damage;
      if (Math.abs(v.wear - orig.wear) > 0.001) change.wear = v.wear;

      // Check fill unit changes
      const fillChanges: { index: number; fillLevel: number }[] = [];
      for (const unit of v.fillUnits) {
        const origUnit = orig.fillUnits.find((u) => u.index === unit.index);
        if (origUnit && Math.abs(unit.fillLevel - origUnit.fillLevel) > 0.001) {
          fillChanges.push({ index: unit.index, fillLevel: unit.fillLevel });
        }
      }
      if (fillChanges.length > 0) change.fillUnits = fillChanges;

      changes.push(change);
    }

    return changes.length > 0 ? changes : null;
  }

  function commitChanges() {
    originalVehicles.value = JSON.parse(JSON.stringify(vehicles.value));
    deletedVehicleIds.value = new Set();
  }

  return {
    vehicles,
    selectedVehicleIds,
    searchQuery,
    typeFilter,
    propertyStateFilter,
    playerVehicles,
    filteredVehicles,
    availableTypes,
    isDirty,
    changeCount,
    getVehicleById,
    getOriginalVehicleById,
    hydrate,
    updateVehicle,
    updateFillLevel,
    fillAllTanks,
    emptyAllTanks,
    resetVehicleAge,
    deleteVehicle,
    batchFillAll,
    batchResetAge,
    batchChangeOwner,
    selectVehicle,
    deselectVehicle,
    toggleSelection,
    selectAll,
    deselectAll,
    resetChanges,
    getChanges,
    commitChanges,
  };
});

function vehicleTypeFromFilename(filename: string): string {
  const parts = filename.replace(/\\/g, "/").toLowerCase().split("/");
  const knownTypes = new Set([
    "tractors", "harvesters", "trailers", "tools", "cars", "trucks",
    "cutters", "forageharvesters", "loaders", "telehandlers", "wheelloaders",
    "placeables", "sprayers", "mowers", "balers", "spreaders", "cultivators",
    "plows", "seeders", "weeders", "rollers", "levelers", "forklifts",
    "conveyors", "augerwagons", "mixerwagons", "animals", "pallets",
  ]);
  const caseMap: Record<string, string> = {
    forageharvesters: "forageHarvesters",
    telehandlers: "teleHandlers",
    wheelloaders: "wheelLoaders",
    augerwagons: "augerWagons",
    mixerwagons: "mixerWagons",
  };
  for (const part of parts) {
    if (knownTypes.has(part)) {
      return caseMap[part] ?? part;
    }
  }
  return "other";
}

function isVehicleModified(v: Vehicle, orig: Vehicle): boolean {
  if (
    v.price !== orig.price ||
    v.operatingTime !== orig.operatingTime ||
    v.age !== orig.age ||
    v.propertyState !== orig.propertyState ||
    Math.abs(v.damage - orig.damage) > 0.001 ||
    Math.abs(v.wear - orig.wear) > 0.001
  ) {
    return true;
  }
  // Check fill units
  for (const unit of v.fillUnits) {
    const origUnit = orig.fillUnits.find((u) => u.index === unit.index);
    if (origUnit && Math.abs(unit.fillLevel - origUnit.fillLevel) > 0.001) {
      return true;
    }
  }
  return false;
}
