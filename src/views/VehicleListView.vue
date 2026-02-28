<script setup lang="ts">
import { computed, watch } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { useVehicleStore } from "@/stores/vehicle";
import { useSettingsStore } from "@/stores/settings";
import { formatMoney, formatOperatingTime, vehicleType } from "@/lib/utils";
import { useVehicleImages } from "@/composables/useVehicleImages";
import VehicleBatchActions from "@/components/vehicles/VehicleBatchActions.vue";
import VehicleImage from "@/components/vehicles/VehicleImage.vue";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Search, RotateCcw } from "lucide-vue-next";

const { t } = useI18n();
const store = useVehicleStore();
const settings = useSettingsStore();
const router = useRouter();
const { loadBatch } = useVehicleImages();

watch(
  () => store.playerVehicles,
  (vehicles) => {
    if (settings.gamePath && vehicles.length > 0) {
      loadBatch(
        settings.gamePath,
        vehicles.map((v) => v.filename),
      );
    }
  },
  { immediate: true },
);

const totalCount = computed(() => store.playerVehicles.length);
const filteredCount = computed(() => store.filteredVehicles.length);
const hasFilters = computed(
  () => store.searchQuery || store.typeFilter || store.propertyStateFilter,
);

const allSelected = computed(
  () =>
    store.filteredVehicles.length > 0 &&
    store.filteredVehicles.every((v) => store.selectedVehicleIds.has(v.uniqueId)),
);

const someSelected = computed(
  () =>
    store.filteredVehicles.some((v) => store.selectedVehicleIds.has(v.uniqueId)) &&
    !allSelected.value,
);

const selectedCount = computed(() => store.selectedVehicleIds.size);

function toggleSelectAll(checked: boolean | "indeterminate") {
  if (checked === true) {
    store.selectAll();
  } else {
    store.deselectAll();
  }
}

function navigateToVehicle(uniqueId: string) {
  router.push(`/editor/vehicles/${uniqueId}`);
}

function propertyStateBadgeVariant(state: string) {
  switch (state) {
    case "Owned":
      return "default";
    case "Rented":
      return "secondary";
    case "Mission":
      return "secondary";
    default:
      return "outline";
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleTypeFilter(value: any) {
  store.typeFilter = value === "all" ? null : String(value);
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleStateFilter(value: any) {
  store.propertyStateFilter = value === "all" ? null : String(value);
}

function clearFilters() {
  store.searchQuery = "";
  store.typeFilter = null;
  store.propertyStateFilter = null;
}

const typeOptions = computed(() =>
  store.availableTypes.map((type) => ({
    value: type,
    label: t(`vehicleTypes.${type}`),
  })),
);
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div>
      <h2 class="text-2xl font-semibold">
        {{ t("vehicle.title") }}
        <span class="text-muted-foreground font-normal">({{ totalCount }})</span>
      </h2>
      <p class="text-sm text-muted-foreground">
        {{ t("vehicle.subtitle") }}
      </p>
    </div>

    <!-- Filters bar -->
    <div class="flex flex-wrap items-center gap-3">
      <div class="relative flex-1 min-w-[200px] max-w-sm">
        <Search class="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          v-model="store.searchQuery"
          :placeholder="t('vehicle.searchPlaceholder')"
          class="pl-9"
        />
      </div>

      <Select
        :model-value="store.typeFilter ?? 'all'"
        @update:model-value="handleTypeFilter"
      >
        <SelectTrigger class="w-[180px]">
          <SelectValue :placeholder="t('vehicle.typePlaceholder')" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">{{ t("vehicle.allTypes") }}</SelectItem>
          <SelectItem
            v-for="opt in typeOptions"
            :key="opt.value"
            :value="opt.value"
          >
            {{ opt.label }}
          </SelectItem>
        </SelectContent>
      </Select>

      <Select
        :model-value="store.propertyStateFilter ?? 'all'"
        @update:model-value="handleStateFilter"
      >
        <SelectTrigger class="w-[150px]">
          <SelectValue :placeholder="t('vehicle.statePlaceholder')" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">{{ t("vehicle.allStates") }}</SelectItem>
          <SelectItem value="Owned">{{ t("propertyStates.Owned") }}</SelectItem>
          <SelectItem value="Rented">{{ t("propertyStates.Rented") }}</SelectItem>
          <SelectItem value="Mission">{{ t("propertyStates.Mission") }}</SelectItem>
        </SelectContent>
      </Select>

      <Button
        v-if="hasFilters"
        variant="ghost"
        size="sm"
        @click="clearFilters"
      >
        <RotateCcw class="size-4" />
        {{ t("common.clear") }}
      </Button>
    </div>

    <!-- Batch actions toolbar -->
    <VehicleBatchActions v-if="selectedCount > 0" />

    <!-- Filtered info -->
    <p
      v-if="hasFilters"
      class="text-sm text-muted-foreground"
    >
      {{ t("vehicle.found", { count: filteredCount }) }}
    </p>

    <!-- Table -->
    <div class="rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead class="w-12">
              <Checkbox
                :model-value="allSelected ? true : someSelected ? 'indeterminate' : false"
                @update:model-value="toggleSelectAll"
              />
            </TableHead>
            <TableHead v-if="settings.gamePath" class="w-14" />
            <TableHead>{{ t("vehicle.name") }}</TableHead>
            <TableHead>{{ t("vehicle.type") }}</TableHead>
            <TableHead>{{ t("vehicle.state") }}</TableHead>
            <TableHead class="text-right">{{ t("vehicle.price") }}</TableHead>
            <TableHead class="text-right">{{ t("vehicle.hours") }}</TableHead>
            <TableHead class="text-right">{{ t("vehicle.condition") }}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow
            v-if="filteredCount === 0"
          >
            <TableCell :colspan="settings.gamePath ? 8 : 7" class="text-center py-8 text-muted-foreground">
              <template v-if="hasFilters">
                {{ t("vehicle.noMatch") }}
              </template>
              <template v-else>
                {{ t("vehicle.empty") }}
              </template>
            </TableCell>
          </TableRow>
          <TableRow
            v-for="vehicle in store.filteredVehicles"
            :key="vehicle.uniqueId"
          >
            <TableCell>
              <Checkbox
                :model-value="store.selectedVehicleIds.has(vehicle.uniqueId)"
                @update:model-value="store.toggleSelection(vehicle.uniqueId)"
              />
            </TableCell>
            <TableCell v-if="settings.gamePath">
              <VehicleImage :filename="vehicle.filename" size="sm" />
            </TableCell>
            <TableCell
              class="cursor-pointer font-medium hover:underline"
              @click="navigateToVehicle(vehicle.uniqueId)"
            >
              {{ vehicle.displayName }}
            </TableCell>
            <TableCell class="text-muted-foreground">
              {{ t(`vehicleTypes.${vehicleType(vehicle.filename)}`) }}
            </TableCell>
            <TableCell>
              <Badge :variant="propertyStateBadgeVariant(vehicle.propertyState)">
                {{ t(`propertyStates.${vehicle.propertyState}`) }}
              </Badge>
            </TableCell>
            <TableCell class="text-right font-mono">
              {{ formatMoney(vehicle.price) }} $
            </TableCell>
            <TableCell class="text-right font-mono">
              {{ formatOperatingTime(vehicle.operatingTime) }}
            </TableCell>
            <TableCell class="text-right">
              <Badge
                :variant="vehicle.wear < 0.3 ? 'default' : vehicle.wear < 0.7 ? 'secondary' : 'destructive'"
              >
                {{ Math.round((1 - vehicle.wear) * 100) }}%
              </Badge>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>

    <!-- Dirty indicator -->
    <div v-if="store.isDirty" class="flex items-center gap-3">
      <span class="text-sm text-amber-600">{{ t("common.unsavedChanges") }}</span>
      <Button variant="ghost" size="sm" @click="store.resetChanges()">
        {{ t("common.cancelChanges") }}
      </Button>
    </div>
  </div>
</template>
