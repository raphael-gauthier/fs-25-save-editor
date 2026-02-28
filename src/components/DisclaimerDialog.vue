<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { TriangleAlert, Square, SquareCheck } from "lucide-vue-next";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { useSettingsStore } from "@/stores/settings";

const { t } = useI18n();
const settings = useSettingsStore();

const accepted = ref(false);
const open = computed(
  () => settings.settingsLoaded && !settings.disclaimerAccepted,
);

function toggle() {
  accepted.value = !accepted.value;
}

function onAccept() {
  settings.acceptDisclaimer();
}
</script>

<template>
  <AlertDialog :open="open">
    <AlertDialogContent class="max-w-lg" @escape-key-down.prevent @pointer-down-outside.prevent @interact-outside.prevent>
      <AlertDialogHeader>
        <AlertDialogTitle class="flex items-center gap-2">
          <TriangleAlert class="size-5 text-destructive" />
          {{ t("disclaimer.title") }}
        </AlertDialogTitle>
        <AlertDialogDescription class="space-y-3 text-sm">
          <p>{{ t("disclaimer.description") }}</p>
          <p class="font-semibold">{{ t("disclaimer.responsibility") }}</p>
          <p>{{ t("disclaimer.backupAdvice") }}</p>
        </AlertDialogDescription>
      </AlertDialogHeader>
      <div
        class="flex items-start gap-3 rounded-md border p-3 cursor-pointer select-none"
        role="checkbox"
        :aria-checked="accepted"
        tabindex="0"
        @click="toggle"
        @keydown.space.prevent="toggle"
      >
        <SquareCheck v-if="accepted" class="size-5 shrink-0 text-primary" />
        <Square v-else class="size-5 shrink-0 text-muted-foreground" />
        <span class="text-sm leading-snug">
          {{ t("disclaimer.accept") }}
        </span>
      </div>
      <AlertDialogFooter>
        <Button :disabled="!accepted" @click="onAccept">
          {{ t("disclaimer.continue") }}
        </Button>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>
