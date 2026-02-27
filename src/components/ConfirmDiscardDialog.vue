<script setup lang="ts">
import { useI18n } from "vue-i18n";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { useUnsavedChanges } from "@/composables/useUnsavedChanges";

const { t } = useI18n();
const { showDiscardDialog, changeCount, resolveDiscard } = useUnsavedChanges();
</script>

<template>
  <AlertDialog :open="showDiscardDialog">
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle>{{ t("common.unsavedChanges") }}</AlertDialogTitle>
        <AlertDialogDescription>
          {{ t("dirtyTracking.pendingChanges", { count: changeCount }) }}
          {{ t("dirtyTracking.discardConfirm") }}
        </AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel @click="resolveDiscard(false)">
          {{ t("common.cancel") }}
        </AlertDialogCancel>
        <AlertDialogAction @click="resolveDiscard(true)">
          {{ t("dirtyTracking.discardAction") }}
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>
