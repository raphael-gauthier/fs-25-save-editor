<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { translateError } from "@/composables/useTauri";
import { useBackupStore } from "@/stores/backup";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Archive,
  Download,
  Trash2,
  RotateCcw,
  Loader2,
} from "lucide-vue-next";

const { t } = useI18n();
const store = useBackupStore();

const confirmDialog = ref<{
  open: boolean;
  title: string;
  description: string;
  action: () => Promise<void>;
}>({
  open: false,
  title: "",
  description: "",
  action: async () => {},
});

onMounted(() => {
  store.listBackups();
});

const totalSize = computed(() =>
  store.backups.reduce((sum, b) => sum + b.sizeBytes, 0),
);

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} o`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} Ko`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} Mo`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} Go`;
}

function formatDate(iso: string): string {
  try {
    return new Date(iso).toLocaleDateString("fr-FR", {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return iso;
  }
}

async function handleCreate() {
  try {
    const info = await store.create();
    if (info) {
      toast.success(t("backup.created"), {
        description: info.name,
      });
    }
  } catch (e: unknown) {
    toast.error(t("backup.createError"), {
      description: translateError(t, e),
    });
  }
}

function confirmRestore(backupName: string) {
  confirmDialog.value = {
    open: true,
    title: t("backup.restoreTitle"),
    description: t("backup.restoreDesc"),
    action: async () => {
      try {
        await store.restore(backupName);
        toast.success(t("backup.restored"), { description: backupName });
      } catch (e: unknown) {
        toast.error(t("backup.restoreError"), {
          description: translateError(t, e),
        });
      }
    },
  };
}

function confirmDelete(backupName: string) {
  confirmDialog.value = {
    open: true,
    title: t("backup.deleteTitle"),
    description: t("backup.deleteDesc"),
    action: async () => {
      try {
        await store.deleteBackup(backupName);
        toast.success(t("backup.deleted"), { description: backupName });
      } catch (e: unknown) {
        toast.error(t("backup.deleteError"), {
          description: translateError(t, e),
        });
      }
    },
  };
}

async function executeConfirmAction() {
  await confirmDialog.value.action();
  confirmDialog.value.open = false;
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-2xl font-semibold">{{ t("backup.title") }}</h2>
        <p class="text-sm text-muted-foreground">
          {{ t("backup.subtitle") }}
        </p>
      </div>
      <Button @click="handleCreate" :disabled="store.isCreating">
        <Loader2 v-if="store.isCreating" class="size-4 animate-spin" />
        <Download v-else class="size-4" />
        {{ t("backup.create") }}
      </Button>
    </div>

    <!-- Loading -->
    <div v-if="store.isLoading" class="space-y-3">
      <Skeleton class="h-10 w-full" />
      <Skeleton class="h-10 w-full" />
      <Skeleton class="h-10 w-full" />
    </div>

    <!-- Empty state -->
    <div
      v-else-if="store.backups.length === 0"
      class="flex flex-col items-center gap-3 rounded-xl border border-dashed p-10 text-center"
    >
      <Archive class="size-10 text-muted-foreground" />
      <p class="font-medium">{{ t("backup.emptyTitle") }}</p>
      <p class="text-sm text-muted-foreground">
        {{ t("backup.emptyDesc") }}
      </p>
    </div>

    <!-- Backup table -->
    <template v-else>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>{{ t("backup.name") }}</TableHead>
            <TableHead>{{ t("backup.date") }}</TableHead>
            <TableHead>{{ t("backup.size") }}</TableHead>
            <TableHead class="w-24 text-right">{{ t("backup.actions") }}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="backup in store.backups" :key="backup.name">
            <TableCell class="font-mono text-sm">{{ backup.name }}</TableCell>
            <TableCell>{{ formatDate(backup.createdAt) }}</TableCell>
            <TableCell>{{ formatSize(backup.sizeBytes) }}</TableCell>
            <TableCell class="text-right">
              <div class="flex justify-end gap-1">
                <Button
                  variant="ghost"
                  size="icon"
                  class="size-8"
                  :title="t('backup.restore')"
                  @click="confirmRestore(backup.name)"
                >
                  <RotateCcw class="size-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  class="size-8 text-destructive hover:text-destructive"
                  :title="t('backup.delete')"
                  @click="confirmDelete(backup.name)"
                >
                  <Trash2 class="size-4" />
                </Button>
              </div>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>

      <p class="text-sm text-muted-foreground">
        {{ t("backup.spaceUsed", { size: formatSize(totalSize), count: store.backups.length }) }}
      </p>
    </template>

    <!-- Confirmation dialog -->
    <Dialog v-model:open="confirmDialog.open">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{{ confirmDialog.title }}</DialogTitle>
          <DialogDescription>
            {{ confirmDialog.description }}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" @click="confirmDialog.open = false">
            {{ t("common.cancel") }}
          </Button>
          <Button @click="executeConfirmAction">{{ t("common.confirm") }}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
