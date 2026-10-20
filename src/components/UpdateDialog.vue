<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { Download, Loader2 } from "@lucide/vue";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { useUpdateChecker } from "@/composables/useUpdateChecker";
import { useUnsavedChanges } from "@/composables/useUnsavedChanges";

const { t } = useI18n();
const { updateAvailable, progress, installUpdate, dismissUpdate } = useUpdateChecker();
const { confirmDiscardIfDirty } = useUnsavedChanges();

const isOpen = computed({
  get: () => updateAvailable.value !== null,
  set: (val: boolean) => {
    if (!val) dismissUpdate();
  },
});

const installing = computed(() => progress.value !== null);

async function install() {
  // Installing closes the app: unsaved edits would be lost
  if (!(await confirmDiscardIfDirty())) return;
  try {
    await installUpdate();
  } catch (error) {
    toast.error(t("update.failed"), { description: String(error) });
  }
}
</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent class="max-w-lg" :show-close-button="!installing">
      <DialogHeader>
        <DialogTitle>{{ t("update.title") }}</DialogTitle>
        <DialogDescription>
          {{ t("update.description", { version: updateAvailable?.version }) }}
        </DialogDescription>
      </DialogHeader>

      <!-- Release notes -->
      <div
        v-if="updateAvailable?.body"
        class="max-h-64 overflow-y-auto rounded-xl border bg-card p-4"
      >
        <div class="whitespace-pre-wrap text-sm">{{ updateAvailable.body }}</div>
      </div>

      <div v-if="installing" role="status" class="flex flex-col gap-2">
        <span class="text-sm text-muted-foreground">{{ t("update.installing") }}</span>
        <div class="h-2 overflow-hidden rounded-full bg-border">
          <div
            class="h-full rounded-full bg-primary transition-[width] duration-300"
            :style="{ width: Math.round((progress ?? 0) * 100) + '%' }"
          />
        </div>
      </div>

      <DialogFooter class="gap-2">
        <Button variant="outline" :disabled="installing" @click="dismissUpdate">
          {{ t("update.later") }}
        </Button>
        <Button :disabled="installing" @click="install">
          <Loader2 v-if="installing" class="size-4 animate-spin" />
          <Download v-else class="size-4" />
          {{ t("update.install") }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
