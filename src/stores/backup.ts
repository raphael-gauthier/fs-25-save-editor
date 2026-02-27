import { defineStore } from "pinia";
import { ref } from "vue";
import { useTauri } from "@/composables/useTauri";
import { useSavegameStore } from "@/stores/savegame";
import type { BackupInfo } from "@/lib/types";

export const useBackupStore = defineStore("backup", () => {
  const { invokeCommand } = useTauri();

  const backups = ref<BackupInfo[]>([]);
  const isLoading = ref(false);
  const isCreating = ref(false);
  const isRestoring = ref(false);

  function currentPath(): string | null {
    return useSavegameStore().currentPath;
  }

  async function listBackups() {
    const path = currentPath();
    if (!path) return;

    isLoading.value = true;
    try {
      backups.value = await invokeCommand<BackupInfo[]>("list_backups", {
        savegamePath: path,
      });
    } catch {
      backups.value = [];
    } finally {
      isLoading.value = false;
    }
  }

  async function create(): Promise<BackupInfo | null> {
    const path = currentPath();
    if (!path) return null;

    isCreating.value = true;
    try {
      const info = await invokeCommand<BackupInfo>("create_backup", {
        savegamePath: path,
      });
      await listBackups();
      return info;
    } finally {
      isCreating.value = false;
    }
  }

  async function restore(backupName: string) {
    const path = currentPath();
    if (!path) return;

    isRestoring.value = true;
    try {
      await invokeCommand("restore_backup", {
        savegamePath: path,
        backupName,
      });
      await listBackups();
    } finally {
      isRestoring.value = false;
    }
  }

  async function deleteBackup(backupName: string) {
    const path = currentPath();
    if (!path) return;

    await invokeCommand("delete_backup", {
      savegamePath: path,
      backupName,
    });
    await listBackups();
  }

  return {
    backups,
    isLoading,
    isCreating,
    isRestoring,
    listBackups,
    create,
    restore,
    deleteBackup,
  };
});
