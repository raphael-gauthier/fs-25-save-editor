<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useFieldStore } from "@/stores/field";
import { useSettingsStore } from "@/stores/settings";
import type { Field } from "@/lib/types";
import { MAX_GROWTH_STATE } from "@/lib/constants";
import FieldEditor from "@/components/fields/FieldEditor.vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Checkbox } from "@/components/ui/checkbox";
import { Alert, AlertDescription } from "@/components/ui/alert";
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
  Loader2,
  Info,
} from "lucide-vue-next";

const { t } = useI18n();
const store = useFieldStore();
const settings = useSettingsStore();

const editorOpen = ref(false);
const selectedField = ref<Field | null>(null);

const farmlandMap = computed(() =>
  new Map(store.farmlands.map((fl) => [fl.id, fl.farmId])),
);

// Whether we expect density data (game path configured)
const densityExpected = computed(() => !!settings.gamePath);

function ownerLabel(fieldId: number): string {
  const farmId = farmlandMap.value.get(fieldId);
  if (farmId === undefined || farmId === 0) return t("field.free");
  if (farmId === 1) return t("field.myFarm");
  return t("field.farm", { id: farmId });
}

function formatFruitName(name: string): string {
  const unknownMatch = name.match(/^UNKNOWN_(\d+)$/);
  if (unknownMatch) {
    return t("field.unknownFruit", { index: unknownMatch[1] });
  }
  return t(`fillTypes.${name}`, name);
}

function growthPercent(state: number): number {
  return Math.round((state / MAX_GROWTH_STATE) * 100);
}

function densityFruit(fieldId: number): string {
  const d = store.getFieldDensity(fieldId);
  if (!d || !d.dominantFruit) return t("field.none");
  return formatFruitName(d.dominantFruit);
}

function densityGrowthPercent(fieldId: number): number {
  const d = store.getFieldDensity(fieldId);
  if (!d) return 0;
  return Math.round((d.avgGrowthState / MAX_GROWTH_STATE) * 100);
}

function densityGroundLabel(fieldId: number): string {
  const d = store.getFieldDensity(fieldId);
  if (!d || d.groundTypeDistribution.length === 0) return "-";
  const top = d.groundTypeDistribution[0];
  return t(`groundTypes.${top.groundType}`, top.groundType);
}

function treatmentBadge(fieldId: number): { label: string; variant: "default" | "destructive" | "outline" | "secondary" } | null {
  const d = store.getFieldDensity(fieldId);
  if (!d) return null;

  const issues: string[] = [];
  if (d.limeStatus.pctAtZero > 50) issues.push(t("field.limeLevel"));
  if (d.weedCoverage > 30) issues.push(t("field.weedState"));
  if (d.sprayStatus.pctAtZero > 50) issues.push(t("field.sprayLevel"));

  if (issues.length === 0) return null;
  return { label: issues.join(", "), variant: "destructive" };
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
  const ids = Array.from(store.selectedFieldIds);
  store.batchMaxGrowth(ids);
  if (store.hasDensityData) {
    for (const id of ids) {
      const d = store.getFieldDensity(id);
      if (d?.dominantFruit) {
        store.addDensityEdit(id, { setFruitName: d.dominantFruit, setGrowthState: 10 });
      }
    }
  }
}

function handleBatchRemoveWeeds() {
  const ids = Array.from(store.selectedFieldIds);
  store.batchRemoveWeeds(ids);
  if (store.hasDensityData) {
    store.batchDensityEdit(ids, { clearWeeds: true });
  }
}

function handleBatchRemoveStones() {
  const ids = Array.from(store.selectedFieldIds);
  store.batchRemoveStones(ids);
  if (store.hasDensityData) {
    store.batchDensityEdit(ids, { clearStones: true });
  }
}

function handleBatchMaxLime() {
  const ids = Array.from(store.selectedFieldIds);
  store.batchMaxLime(ids);
  if (store.hasDensityData) {
    store.batchDensityEdit(ids, { setLimeLevel: 3 });
  }
}

function handleBatchMaxFertilizer() {
  const ids = Array.from(store.selectedFieldIds);
  store.batchMaxFertilizer(ids);
  if (store.hasDensityData) {
    store.batchDensityEdit(ids, { setSprayLevel: 2 });
  }
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
      <div v-if="store.densityLoading && !store.densityFromCache" class="flex items-center gap-2 text-sm text-muted-foreground">
        <Loader2 class="size-4 animate-spin" />
        {{ t("field.densityLoading") }}
      </div>
    </div>

    <!-- Density data info/error banners -->
    <Alert v-if="store.densityError && !store.densityFromCache" variant="destructive">
      <Info class="size-4" />
      <AlertDescription>{{ t("field.densityError", { details: store.densityError }) }}</AlertDescription>
    </Alert>
    <Alert v-if="store.densityFromCache && store.densityLoading">
      <Loader2 class="size-4 animate-spin" />
      <AlertDescription>{{ t("field.densityCacheRefreshing") }}</AlertDescription>
    </Alert>
    <Alert v-else-if="store.densityFromCache && store.densityError">
      <Info class="size-4" />
      <AlertDescription>{{ t("field.densityCacheStale") }}</AlertDescription>
    </Alert>
    <Alert v-else-if="!store.hasDensityData && !store.densityLoading && !settings.gamePath">
      <Info class="size-4" />
      <AlertDescription>{{ t("field.gamePathRequired") }}</AlertDescription>
    </Alert>

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
            {{ formatFruitName(fruit) }}
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
            <TableHead v-if="store.hasDensityData">{{ t("field.treatments") }}</TableHead>
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

            <!-- Fruit -->
            <TableCell>
              <div class="flex items-center gap-2">
                <Sprout class="size-4 text-muted-foreground" />
                <template v-if="store.hasDensityData">
                  <span>{{ densityFruit(field.id) }}</span>
                  <Badge
                    v-if="(store.getFieldDensity(field.id)?.fruitDistribution?.length ?? 0) > 1"
                    variant="outline"
                    class="text-xs"
                  >
                    +{{ (store.getFieldDensity(field.id)?.fruitDistribution?.length ?? 1) - 1 }}
                  </Badge>
                </template>
                <template v-else-if="densityExpected">
                  <div class="h-4 w-20 animate-pulse rounded bg-muted" />
                </template>
                <template v-else>
                  <span>{{ field.fruitType !== "UNKNOWN" ? t(`fillTypes.${field.fruitType}`, field.fruitType) : t("field.none") }}</span>
                  <Badge
                    v-if="field.plannedFruit !== 'FALLOW' && field.plannedFruit !== field.fruitType"
                    variant="outline"
                    class="text-xs"
                  >
                    {{ t(`fillTypes.${field.plannedFruit}`, field.plannedFruit) }}
                  </Badge>
                </template>
              </div>
            </TableCell>

            <!-- Growth -->
            <TableCell>
              <template v-if="store.hasDensityData">
                <div v-if="store.getFieldDensity(field.id)?.dominantFruit" class="flex items-center gap-2">
                  <div class="h-2 w-20 rounded-full bg-muted">
                    <div
                      class="h-2 rounded-full"
                      :class="densityGrowthPercent(field.id) >= 100 ? 'bg-green-500' : 'bg-amber-500'"
                      :style="{ width: Math.min(densityGrowthPercent(field.id), 100) + '%' }"
                    />
                  </div>
                  <span class="font-mono text-xs text-muted-foreground">
                    {{ densityGrowthPercent(field.id) }}%
                  </span>
                </div>
                <span v-else class="text-muted-foreground">-</span>
              </template>
              <template v-else-if="densityExpected">
                <div class="h-2 w-20 animate-pulse rounded-full bg-muted" />
              </template>
              <template v-else>
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
              </template>
            </TableCell>

            <!-- Ground -->
            <TableCell>
              <template v-if="store.hasDensityData">
                <Badge variant="secondary" class="text-xs">{{ densityGroundLabel(field.id) }}</Badge>
              </template>
              <template v-else-if="densityExpected">
                <div class="h-5 w-16 animate-pulse rounded bg-muted" />
              </template>
              <template v-else>
                <Badge variant="secondary" class="text-xs">{{ t(`groundTypes.${field.groundType}`, field.groundType) }}</Badge>
              </template>
            </TableCell>

            <!-- Treatment status (density only) -->
            <TableCell v-if="store.hasDensityData">
              <Badge
                v-if="treatmentBadge(field.id)"
                :variant="treatmentBadge(field.id)!.variant"
                class="text-xs"
              >
                {{ treatmentBadge(field.id)!.label }}
              </Badge>
              <span v-else class="text-xs text-green-600 dark:text-green-400">OK</span>
            </TableCell>

            <!-- Owner -->
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
