import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type {
  Mission,
  Collectible,
  ContractSettings,
  MissionChangePayload,
  CollectibleChangePayload,
  ContractSettingsChangePayload,
  DensityEditPayload,
} from "@/lib/types";
import { useFieldStore } from "@/stores/field";

export const useMissionStore = defineStore("mission", () => {
  const missions = ref<Mission[]>([]);
  const originalMissions = ref<Mission[]>([]);
  const collectibles = ref<Collectible[]>([]);
  const originalCollectibles = ref<Collectible[]>([]);
  const contractSettings = ref<ContractSettings | null>(null);
  const originalContractSettings = ref<ContractSettings | null>(null);

  // Filters
  const searchQuery = ref("");
  const statusFilter = ref<string | null>(null);

  // Getters
  const filteredMissions = computed(() => {
    let result = missions.value;

    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase();
      result = result.filter(
        (m) =>
          m.missionType.toLowerCase().includes(query) ||
          m.uniqueId.includes(query) ||
          (m.fruitType && m.fruitType.toLowerCase().includes(query)),
      );
    }

    if (statusFilter.value) {
      result = result.filter((m) => m.status === statusFilter.value);
    }

    return result;
  });

  const activeMissions = computed(() =>
    missions.value.filter((m) => m.status === "Running"),
  );

  const availableMissions = computed(() =>
    missions.value.filter((m) => m.status === "Created"),
  );

  const collectedCount = computed(() =>
    collectibles.value.filter((c) => c.collected).length,
  );

  const isDirty = computed(() => {
    return isMissionsDirty() || isCollectiblesDirty() || isContractSettingsDirty();
  });

  const changeCount = computed(() => {
    let count = 0;
    const origMap = new Map(originalMissions.value.map((m) => [m.uniqueId, m]));
    for (const m of missions.value) {
      const orig = origMap.get(m.uniqueId);
      if (!orig) continue;
      if (m.reward !== orig.reward) count++;
      if (m.completion !== orig.completion) count++;
      if (m.status !== orig.status) count++;
      if (m.reimbursement !== orig.reimbursement) count++;
      if (m.depositedLiters !== orig.depositedLiters) count++;
    }
    for (let i = 0; i < collectibles.value.length; i++) {
      if (collectibles.value[i].collected !== originalCollectibles.value[i]?.collected) {
        count++;
      }
    }
    if (isContractSettingsDirty()) count++;
    return count;
  });

  function isMissionsDirty(): boolean {
    const origMap = new Map(originalMissions.value.map((m) => [m.uniqueId, m]));
    return missions.value.some((m) => {
      const orig = origMap.get(m.uniqueId);
      if (!orig) return false;
      return (
        m.reward !== orig.reward ||
        m.completion !== orig.completion ||
        m.status !== orig.status ||
        m.reimbursement !== orig.reimbursement ||
        m.depositedLiters !== orig.depositedLiters
      );
    });
  }

  function isCollectiblesDirty(): boolean {
    return collectibles.value.some(
      (c, i) => c.collected !== originalCollectibles.value[i]?.collected,
    );
  }

  function isContractSettingsDirty(): boolean {
    if (!contractSettings.value || !originalContractSettings.value) return false;
    const cs = contractSettings.value;
    const ocs = originalContractSettings.value;
    return (
      cs.leaseVehicle !== ocs.leaseVehicle ||
      cs.missionPerFarm !== ocs.missionPerFarm ||
      cs.allowClearAdd !== ocs.allowClearAdd
    );
  }

  // Actions
  function hydrate(
    missionData: Mission[],
    collectibleData: Collectible[],
    settings: ContractSettings | null,
  ) {
    const missionsSerialized = JSON.stringify(missionData);
    const collectiblesSerialized = JSON.stringify(collectibleData);
    missions.value = JSON.parse(missionsSerialized);
    originalMissions.value = JSON.parse(missionsSerialized);
    collectibles.value = JSON.parse(collectiblesSerialized);
    originalCollectibles.value = JSON.parse(collectiblesSerialized);
    contractSettings.value = settings ? { ...settings } : null;
    originalContractSettings.value = settings ? { ...settings } : null;
  }

  function updateMission(id: string, changes: Partial<Mission>) {
    const m = missions.value.find((mi) => mi.uniqueId === id);
    if (m) {
      Object.assign(m, changes);
    }
  }

  /**
   * Mission types that can be completed via density map edits.
   * Maps mission type → density edit generator.
   */
  function canCompleteMission(mission: Mission): boolean {
    // Harvest missions cannot be completed via save editing — the game requires
    // both terrain changes AND the grain delivery loop. Clearing terrain without
    // actual delivery causes the game to remove the mission as invalid.
    const completableTypes = [
      "plow", "cultivate", "hoe", "mow", "herbicide", "stonePick",
    ];
    return mission.fieldId != null && completableTypes.includes(mission.missionType);
  }

  function completeMission(id: string) {
    const m = missions.value.find((mi) => mi.uniqueId === id);
    if (!m || m.fieldId == null) return;

    const fieldStore = useFieldStore();
    const fieldId = m.fieldId;

    // Set mission XML state
    if (m.status === "Created") {
      m.status = "Running";
    }

    // Build density edits based on mission type
    const densityEdit: Partial<DensityEditPayload> = {};

    // Don't force completion=1.0 — the game recalculates from terrain/depositedLiters.
    // Setting it externally can cause the game to auto-complete and remove the mission.

    switch (m.missionType) {
      // harvest: not completable via save editing (requires gameplay delivery loop)
      case "plow":
        densityEdit.setGroundType = 4; // PLOWED
        densityEdit.setPlowLevel = 1;
        break;
      case "cultivate":
        densityEdit.setGroundType = 2; // CULTIVATED
        break;
      case "hoe":
        densityEdit.setGroundType = 2; // CULTIVATED
        densityEdit.clearWeeds = true;
        break;
      case "mow":
        densityEdit.setGroundType = 15; // GRASS_CUT
        densityEdit.setFruitName = "NONE";
        densityEdit.setGrowthState = 0;
        break;
      case "herbicide":
        densityEdit.clearWeeds = true;
        break;
      case "stonePick":
        densityEdit.clearStones = true;
        break;
    }

    // Only apply density edits if there are actual terrain changes
    const hasDensityChanges = Object.keys(densityEdit).length > 0;
    if (hasDensityChanges) {
      // cropAreaOnly (fruit_idx > 0): for harvest/mow where we clear crop pixels only.
      // fieldAreaOnly (ground_type 1-13): for plow/cultivate/hoe where the field
      // has no crops but has field-like ground types (excludes roads/grass).
      if (m.missionType === "harvest" || m.missionType === "mow") {
        densityEdit.cropAreaOnly = true;
      } else {
        densityEdit.fieldAreaOnly = true;
      }
      fieldStore.addDensityEdit(fieldId, densityEdit);
    }

    // Also update XML field state to keep UI consistent
    const xmlUpdates: Record<string, unknown> = {};
    switch (m.missionType) {
      case "plow":
        xmlUpdates.groundType = "PLOWED";
        xmlUpdates.plowLevel = 1;
        break;
      case "cultivate":
        xmlUpdates.groundType = "CULTIVATED";
        break;
      case "hoe":
        xmlUpdates.groundType = "CULTIVATED";
        xmlUpdates.weedState = 0;
        break;
      case "mow":
        xmlUpdates.groundType = "GRASS_CUT";
        xmlUpdates.fruitType = "UNKNOWN";
        xmlUpdates.growthState = 0;
        break;
      case "herbicide":
        xmlUpdates.weedState = 0;
        break;
      case "stonePick":
        xmlUpdates.stoneLevel = 0;
        break;
    }
    if (Object.keys(xmlUpdates).length > 0) {
      fieldStore.updateField(fieldId, xmlUpdates);
    }
  }

  function toggleCollectible(index: number) {
    const c = collectibles.value.find((co) => co.index === index);
    if (c) {
      c.collected = !c.collected;
    }
  }

  function collectAll() {
    for (const c of collectibles.value) {
      c.collected = true;
    }
  }

  function resetAllCollectibles() {
    for (const c of collectibles.value) {
      c.collected = false;
    }
  }

  function updateContractSettings(changes: Partial<ContractSettings>) {
    if (contractSettings.value) {
      Object.assign(contractSettings.value, changes);
    }
  }

  function resetChanges() {
    missions.value = JSON.parse(JSON.stringify(originalMissions.value));
    collectibles.value = JSON.parse(JSON.stringify(originalCollectibles.value));
    contractSettings.value = originalContractSettings.value
      ? { ...originalContractSettings.value }
      : null;
  }

  function getChanges(): {
    missions?: MissionChangePayload[];
    collectibles?: CollectibleChangePayload[];
    contractSettings?: ContractSettingsChangePayload;
  } | null {
    const result: {
      missions?: MissionChangePayload[];
      collectibles?: CollectibleChangePayload[];
      contractSettings?: ContractSettingsChangePayload;
    } = {};
    let hasChanges = false;

    // Mission changes
    const missionChanges: MissionChangePayload[] = [];
    const origMap = new Map(originalMissions.value.map((m) => [m.uniqueId, m]));
    for (const m of missions.value) {
      const orig = origMap.get(m.uniqueId);
      if (!orig) continue;
      const changed =
        m.reward !== orig.reward ||
        m.completion !== orig.completion ||
        m.status !== orig.status ||
        m.reimbursement !== orig.reimbursement ||
        m.depositedLiters !== orig.depositedLiters;
      if (!changed) continue;

      const change: MissionChangePayload = { uniqueId: m.uniqueId };
      if (m.reward !== orig.reward) change.reward = m.reward;
      if (m.completion !== orig.completion) change.completion = m.completion;
      if (m.status !== orig.status) change.status = missionStatusToString(m.status);
      if (m.reimbursement !== orig.reimbursement) change.reimbursement = m.reimbursement;
      if (m.depositedLiters !== orig.depositedLiters) change.depositedLiters = m.depositedLiters ?? undefined;
      missionChanges.push(change);
    }
    if (missionChanges.length > 0) {
      result.missions = missionChanges;
      hasChanges = true;
    }

    // Collectible changes
    const collectibleChanges: CollectibleChangePayload[] = [];
    for (let i = 0; i < collectibles.value.length; i++) {
      const c = collectibles.value[i];
      const orig = originalCollectibles.value[i];
      if (c.collected !== orig?.collected) {
        collectibleChanges.push({ index: c.index, collected: c.collected });
      }
    }
    if (collectibleChanges.length > 0) {
      result.collectibles = collectibleChanges;
      hasChanges = true;
    }

    // Contract settings changes
    if (isContractSettingsDirty() && contractSettings.value && originalContractSettings.value) {
      const cs = contractSettings.value;
      const ocs = originalContractSettings.value;
      const csChange: ContractSettingsChangePayload = {};
      if (cs.leaseVehicle !== ocs.leaseVehicle) csChange.leaseVehicle = cs.leaseVehicle;
      if (cs.missionPerFarm !== ocs.missionPerFarm) csChange.missionPerFarm = cs.missionPerFarm;
      if (cs.allowClearAdd !== ocs.allowClearAdd) csChange.allowClearAdd = cs.allowClearAdd;
      result.contractSettings = csChange;
      hasChanges = true;
    }

    return hasChanges ? result : null;
  }

  function commitChanges() {
    originalMissions.value = JSON.parse(JSON.stringify(missions.value));
    originalCollectibles.value = JSON.parse(JSON.stringify(collectibles.value));
    originalContractSettings.value = contractSettings.value
      ? { ...contractSettings.value }
      : null;
  }

  return {
    missions,
    collectibles,
    contractSettings,
    searchQuery,
    statusFilter,
    filteredMissions,
    activeMissions,
    availableMissions,
    collectedCount,
    isDirty,
    changeCount,
    hydrate,
    updateMission,
    canCompleteMission,
    completeMission,
    toggleCollectible,
    collectAll,
    resetAllCollectibles,
    updateContractSettings,
    resetChanges,
    getChanges,
    commitChanges,
  };
});

function missionStatusToString(status: Mission["status"]): string {
  switch (status) {
    case "Created":
      return "CREATED";
    case "Running":
      return "RUNNING";
    case "Completed":
      return "COMPLETED";
  }
}
