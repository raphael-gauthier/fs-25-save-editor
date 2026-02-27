import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type {
  Mission,
  Collectible,
  ContractSettings,
  MissionChangePayload,
  CollectibleChangePayload,
  ContractSettingsChangePayload,
} from "@/lib/types";

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
        m.reimbursement !== orig.reimbursement
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

  function completeMission(id: string) {
    const m = missions.value.find((mi) => mi.uniqueId === id);
    if (m) {
      m.completion = 1.0;
      m.status = "Completed";
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
        m.reimbursement !== orig.reimbursement;
      if (!changed) continue;

      const change: MissionChangePayload = { uniqueId: m.uniqueId };
      if (m.reward !== orig.reward) change.reward = m.reward;
      if (m.completion !== orig.completion) change.completion = m.completion;
      if (m.status !== orig.status) change.status = missionStatusToString(m.status);
      if (m.reimbursement !== orig.reimbursement) change.reimbursement = m.reimbursement;
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
      return "0";
    case "Running":
      return "1";
    case "Completed":
      return "2";
  }
}
