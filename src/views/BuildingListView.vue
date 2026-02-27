<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useBuildingStore } from "@/stores/building";
import type { Placeable } from "@/lib/types";
import BuildingEditor from "@/components/buildings/BuildingEditor.vue";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
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
import { Building2, Hammer } from "lucide-vue-next";

const { t } = useI18n();
const store = useBuildingStore();

const editorOpen = ref(false);
const selectedPlaceable = ref<Placeable | null>(null);

function openEditor(placeable: Placeable) {
  selectedPlaceable.value = placeable;
  editorOpen.value = true;
}

function stateLabel(p: Placeable): string {
  if (p.isPrePlaced) return t("building.prePlaced");
  if (p.isUnderConstruction) return t("building.underConstruction");
  return t("building.ready");
}

function stateBadgeVariant(p: Placeable): "default" | "secondary" | "outline" {
  if (p.isUnderConstruction) return "secondary";
  if (p.isPrePlaced) return "outline";
  return "default";
}

function ownerLabel(farmId: number): string {
  if (farmId === 0) return t("field.free");
  if (farmId === 1) return t("field.myFarm");
  return t("field.farm", { id: farmId });
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleOwnerFilter(value: any) {
  store.ownerFilter = value === "__all__" ? null : Number(value);
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleStateFilter(value: any) {
  store.stateFilter = value === "__all__" ? null : String(value);
}
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-2xl font-semibold">{{ t("building.title") }}</h2>
        <p class="text-sm text-muted-foreground">{{ t("building.subtitle") }}</p>
      </div>
    </div>

    <!-- Filters -->
    <div class="flex flex-wrap items-center gap-3">
      <Input
        v-model="store.searchQuery"
        :placeholder="t('building.searchPlaceholder')"
        class="max-w-xs"
      />
      <Select
        :model-value="store.ownerFilter !== null ? String(store.ownerFilter) : '__all__'"
        @update:model-value="handleOwnerFilter"
      >
        <SelectTrigger class="w-44">
          <SelectValue :placeholder="t('building.allOwners')" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="__all__">{{ t("building.allOwners") }}</SelectItem>
          <SelectItem value="0">{{ t("field.free") }}</SelectItem>
          <SelectItem value="1">{{ t("field.myFarm") }}</SelectItem>
        </SelectContent>
      </Select>
      <Select
        :model-value="store.stateFilter ?? '__all__'"
        @update:model-value="handleStateFilter"
      >
        <SelectTrigger class="w-48">
          <SelectValue :placeholder="t('building.allStates')" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="__all__">{{ t("building.allStates") }}</SelectItem>
          <SelectItem value="ready">{{ t("building.ready") }}</SelectItem>
          <SelectItem value="underConstruction">{{ t("building.underConstruction") }}</SelectItem>
          <SelectItem value="prePlaced">{{ t("building.prePlaced") }}</SelectItem>
        </SelectContent>
      </Select>
      <span class="text-sm text-muted-foreground">
        {{ t("building.found", { count: store.filteredPlaceables.length }) }}
      </span>
    </div>

    <!-- Table -->
    <div v-if="store.filteredPlaceables.length > 0" class="rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>{{ t("building.name") }}</TableHead>
            <TableHead>{{ t("building.owner") }}</TableHead>
            <TableHead>{{ t("building.state") }}</TableHead>
            <TableHead class="text-right">{{ t("building.price") }}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow
            v-for="p in store.filteredPlaceables"
            :key="p.index"
            class="cursor-pointer"
            @click="openEditor(p)"
          >
            <TableCell>
              <div class="flex items-center gap-2">
                <Building2 class="size-4 text-muted-foreground" />
                <span>{{ p.displayName }}</span>
              </div>
            </TableCell>
            <TableCell>
              <Badge
                :variant="p.farmId === 1 ? 'default' : 'outline'"
                class="text-xs"
              >
                {{ ownerLabel(p.farmId) }}
              </Badge>
            </TableCell>
            <TableCell>
              <Badge :variant="stateBadgeVariant(p)" class="text-xs">
                <Hammer v-if="p.isUnderConstruction" class="mr-1 size-3" />
                {{ stateLabel(p) }}
              </Badge>
            </TableCell>
            <TableCell class="text-right font-mono">
              {{ p.price > 0 ? `${Math.round(p.price).toLocaleString()} $` : "-" }}
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>

    <!-- Empty state -->
    <div v-else-if="store.placeables.length === 0" class="py-12 text-center">
      <Building2 class="mx-auto size-12 text-muted-foreground/50" />
      <p class="mt-4 text-muted-foreground">{{ t("building.empty") }}</p>
    </div>

    <!-- No matches -->
    <div v-else class="py-12 text-center">
      <p class="text-muted-foreground">{{ t("building.noMatch") }}</p>
    </div>

    <!-- Building Editor Sheet -->
    <BuildingEditor
      :placeable="selectedPlaceable"
      :open="editorOpen"
      @update:open="editorOpen = $event"
    />
  </div>
</template>
