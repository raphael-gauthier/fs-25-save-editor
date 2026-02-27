<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useFieldStore } from "@/stores/field";
import type { Field } from "@/lib/types";
import { MAX_GROWTH_STATE } from "@/lib/constants";
import FieldEditor from "@/components/fields/FieldEditor.vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
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
import {
  Sprout,
  Sparkles,
  Bug,
  Gem,
  Droplets,
  FlaskConical,
} from "lucide-vue-next";

const { t } = useI18n();
const store = useFieldStore();

const editorOpen = ref(false);
const selectedField = ref<Field | null>(null);

const farmlandMap = computed(() =>
  new Map(store.farmlands.map((fl) => [fl.id, fl.farmId])),
);

function ownerLabel(fieldId: number): string {
  const farmId = farmlandMap.value.get(fieldId);
  if (farmId === undefined || farmId === 0) return t("field.free");
  if (farmId === 1) return t("field.myFarm");
  return t("field.farm", { id: farmId });
}

function growthPercent(state: number): number {
  return Math.round((state / MAX_GROWTH_STATE) * 100);
}

function openEditor(field: Field) {
  selectedField.value = field;
  editorOpen.value = true;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleFruitFilter(value: any) {
  store.fruitFilter = value === "__all__" ? null : String(value);
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleOwnerFilter(value: any) {
  store.ownerFilter = value === "__all__" ? null : Number(value);
}

function handleBatchMaxGrowth() {
  store.batchMaxGrowth(Array.from(store.selectedFieldIds));
}

function handleBatchRemoveWeeds() {
  store.batchRemoveWeeds(Array.from(store.selectedFieldIds));
}

function handleBatchRemoveStones() {
  store.batchRemoveStones(Array.from(store.selectedFieldIds));
}

function handleBatchMaxLime() {
  store.batchMaxLime(Array.from(store.selectedFieldIds));
}

function handleBatchMaxFertilizer() {
  store.batchMaxFertilizer(Array.from(store.selectedFieldIds));
}

const allSelected = computed(() => {
  if (store.filteredFields.length === 0) return false;
  return store.filteredFields.every((f) => store.selectedFieldIds.has(f.id));
});

function toggleSelectAll() {
  if (allSelected.value) {
    store.deselectAll();
  } else {
    store.selectAll();
  }
}
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-2xl font-semibold">{{ t("field.title") }}</h2>
        <p class="text-sm text-muted-foreground">{{ t("field.subtitle") }}</p>
      </div>
    </div>

    <!-- Filters -->
    <div class="flex flex-wrap items-center gap-3">
      <Input
        v-model="store.searchQuery"
        :placeholder="t('field.searchPlaceholder')"
        class="max-w-xs"
      />
      <Select
        :model-value="store.fruitFilter ?? '__all__'"
        @update:model-value="handleFruitFilter"
      >
        <SelectTrigger class="w-44">
          <SelectValue :placeholder="t('field.fruitPlaceholder')" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="__all__">{{ t("field.allFruits") }}</SelectItem>
          <SelectItem v-for="fruit in store.availableFruits" :key="fruit" :value="fruit">
            {{ fruit }}
          </SelectItem>
        </SelectContent>
      </Select>
      <Select
        :model-value="store.ownerFilter !== null ? String(store.ownerFilter) : '__all__'"
        @update:model-value="handleOwnerFilter"
      >
        <SelectTrigger class="w-44">
          <SelectValue :placeholder="t('field.ownerPlaceholder')" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="__all__">{{ t("field.allOwners") }}</SelectItem>
          <SelectItem value="0">{{ t("field.free") }}</SelectItem>
          <SelectItem value="1">{{ t("field.myFarm") }}</SelectItem>
        </SelectContent>
      </Select>
      <span class="text-sm text-muted-foreground">
        {{ t("field.found", { count: store.filteredFields.length }) }}
      </span>
    </div>

    <!-- Batch actions -->
    <div v-if="store.selectedFieldIds.size > 0" class="flex flex-wrap items-center gap-2">
      <span class="text-sm font-medium">
        {{ t("field.selected", { count: store.selectedFieldIds.size }) }}
      </span>
      <Button variant="outline" size="sm" @click="handleBatchMaxGrowth">
        <Sparkles class="size-4" />
        {{ t("field.maxGrowth") }}
      </Button>
      <Button variant="outline" size="sm" @click="handleBatchRemoveWeeds">
        <Bug class="size-4" />
        {{ t("field.removeWeeds") }}
      </Button>
      <Button variant="outline" size="sm" @click="handleBatchRemoveStones">
        <Gem class="size-4" />
        {{ t("field.removeStones") }}
      </Button>
      <Button variant="outline" size="sm" @click="handleBatchMaxLime">
        <Droplets class="size-4" />
        {{ t("field.maxLime") }}
      </Button>
      <Button variant="outline" size="sm" @click="handleBatchMaxFertilizer">
        <FlaskConical class="size-4" />
        {{ t("field.maxFertilizer") }}
      </Button>
      <Button variant="ghost" size="sm" @click="store.deselectAll()">
        {{ t("common.deselectAll") }}
      </Button>
    </div>

    <!-- Table -->
    <div v-if="store.filteredFields.length > 0" class="rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead class="w-10">
              <Checkbox :model-value="allSelected" @update:model-value="toggleSelectAll" />
            </TableHead>
            <TableHead class="w-16">{{ t("field.id") }}</TableHead>
            <TableHead>{{ t("field.fruit") }}</TableHead>
            <TableHead>{{ t("field.growth") }}</TableHead>
            <TableHead>{{ t("field.ground") }}</TableHead>
            <TableHead>{{ t("field.owner") }}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow
            v-for="field in store.filteredFields"
            :key="field.id"
            class="cursor-pointer"
            @click="openEditor(field)"
          >
            <TableCell @click.stop>
              <Checkbox
                :model-value="store.selectedFieldIds.has(field.id)"
                @update:model-value="store.toggleSelection(field.id)"
              />
            </TableCell>
            <TableCell class="font-mono">{{ field.id }}</TableCell>
            <TableCell>
              <div class="flex items-center gap-2">
                <Sprout class="size-4 text-muted-foreground" />
                <span>{{ field.fruitType !== "UNKNOWN" ? field.fruitType : t("field.none") }}</span>
                <Badge
                  v-if="field.plannedFruit !== 'FALLOW' && field.plannedFruit !== field.fruitType"
                  variant="outline"
                  class="text-xs"
                >
                  {{ field.plannedFruit }}
                </Badge>
              </div>
            </TableCell>
            <TableCell>
              <div v-if="field.fruitType !== 'UNKNOWN'" class="flex items-center gap-2">
                <div class="h-2 w-20 rounded-full bg-muted">
                  <div
                    class="h-2 rounded-full"
                    :class="field.growthState >= MAX_GROWTH_STATE ? 'bg-green-500' : 'bg-amber-500'"
                    :style="{ width: growthPercent(field.growthState) + '%' }"
                  />
                </div>
                <span class="font-mono text-xs text-muted-foreground">
                  {{ field.growthState }}/{{ MAX_GROWTH_STATE }}
                </span>
              </div>
              <span v-else class="text-muted-foreground">-</span>
            </TableCell>
            <TableCell>
              <Badge variant="secondary" class="text-xs">{{ field.groundType }}</Badge>
            </TableCell>
            <TableCell>
              <Badge
                :variant="farmlandMap.get(field.id) === 1 ? 'default' : 'outline'"
                class="text-xs"
              >
                {{ ownerLabel(field.id) }}
              </Badge>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>

    <!-- Empty state -->
    <div v-else-if="store.fields.length === 0" class="py-12 text-center">
      <Sprout class="mx-auto size-12 text-muted-foreground/50" />
      <p class="mt-4 text-muted-foreground">{{ t("field.empty") }}</p>
    </div>

    <!-- No matches -->
    <div v-else class="py-12 text-center">
      <p class="text-muted-foreground">{{ t("field.noMatch") }}</p>
    </div>

    <!-- Field Editor Sheet -->
    <FieldEditor
      :field="selectedField"
      :open="editorOpen"
      @update:open="editorOpen = $event"
    />
  </div>
</template>
