<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Download } from "lucide-vue-next";
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

const { t } = useI18n();
const { updateAvailable, dismissUpdate } = useUpdateChecker();

const isOpen = computed({
  get: () => updateAvailable.value !== null,
  set: (val: boolean) => {
    if (!val) dismissUpdate();
  },
});

async function openReleasePage() {
  if (updateAvailable.value) {
    await openUrl(updateAvailable.value.release_url);
    dismissUpdate();
  }
}
</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent class="max-w-lg">
      <DialogHeader>
        <DialogTitle>{{ t("update.title") }}</DialogTitle>
        <DialogDescription>
          {{ t("update.description", { version: updateAvailable?.version }) }}
        </DialogDescription>
      </DialogHeader>

      <!-- Changelog -->
      <div
        v-if="updateAvailable?.body"
        class="max-h-64 overflow-y-auto rounded-md border p-4"
      >
        <div class="whitespace-pre-wrap text-sm">
          {{ updateAvailable.body }}
        </div>
      </div>

      <DialogFooter class="gap-2 sm:gap-0">
        <Button variant="outline" @click="dismissUpdate">
          {{ t("update.later") }}
        </Button>
        <Button @click="openReleasePage">
          <Download class="mr-2 h-4 w-4" />
          {{ t("update.download") }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
