<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useVehicleStore } from "@/stores/vehicle";
import { Button } from "@/components/ui/button";
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
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Droplets, RotateCcw, X, ChevronDown } from "lucide-vue-next";

const { t } = useI18n();
const store = useVehicleStore();

const selectedIds = computed(() => Array.from(store.selectedVehicleIds));
const count = computed(() => store.selectedVehicleIds.size);

const showResetDialog = ref(false);
const showFillDialog = ref(false);

function handleFillAll() {
  store.batchFillAll(selectedIds.value);
  showFillDialog.value = false;
}

function handleResetAge() {
  store.batchResetAge(selectedIds.value);
  showResetDialog.value = false;
}
</script>

<template>
  <div class="flex items-center gap-3 rounded-lg border bg-muted/50 px-4 py-3">
    <span class="text-sm font-medium">
      {{ t("vehicle.selected", { count }) }}
    </span>

    <div class="flex items-center gap-2">
      <!-- Fill tanks -->
      <AlertDialog v-model:open="showFillDialog">
        <Button variant="outline" size="sm" @click="showFillDialog = true">
          <Droplets class="size-4" />
          {{ t("vehicle.fillTanks") }}
        </Button>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{{ t("vehicle.fillTanks") }}</AlertDialogTitle>
            <AlertDialogDescription>
              {{ t("vehicle.fillTanksDesc", { count }) }}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{{ t("common.cancel") }}</AlertDialogCancel>
            <AlertDialogAction @click="handleFillAll">{{ t("common.confirm") }}</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <!-- Reset to new -->
      <AlertDialog v-model:open="showResetDialog">
        <Button variant="outline" size="sm" @click="showResetDialog = true">
          <RotateCcw class="size-4" />
          {{ t("vehicle.resetToNew") }}
        </Button>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {{ t("vehicle.resetToNewTitle", { count }) }}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {{ t("vehicle.resetToNewDesc", { count }) }}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{{ t("common.cancel") }}</AlertDialogCancel>
            <AlertDialogAction @click="handleResetAge">{{ t("common.confirm") }}</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <!-- More actions -->
      <DropdownMenu>
        <DropdownMenuTrigger as-child>
          <Button variant="ghost" size="sm">
            {{ t("common.more") }}
            <ChevronDown class="ml-1 size-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent>
          <DropdownMenuItem @click="showFillDialog = true">
            <Droplets class="size-4" />
            {{ t("vehicle.fillTanks") }}
          </DropdownMenuItem>
          <DropdownMenuItem @click="showResetDialog = true">
            <RotateCcw class="size-4" />
            {{ t("vehicle.resetToNew") }}
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem @click="store.deselectAll()">
            <X class="size-4" />
            {{ t("common.deselectAll") }}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>

    <Button
      variant="ghost"
      size="icon"
      class="ml-auto size-8"
      @click="store.deselectAll()"
    >
      <X class="size-4" />
    </Button>
  </div>
</template>
