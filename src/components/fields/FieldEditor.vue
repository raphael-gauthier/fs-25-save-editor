<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useFieldStore } from "@/stores/field";
import { useSettingsStore } from "@/stores/settings";
import type { Field, MapFruitType } from "@/lib/types";
import { growthStageLabel } from "@/lib/utils";
import { cropEdit, soilWorkEdit, type SoilWork } from "@/lib/fieldWork";
import { formatMoney } from "@/lib/utils";
import { useFarmlandTrade, PLAYER_FARM_ID } from "@/composables/useFarmlandTrade";
import { FRUIT_TYPES, GROUND_TYPES, HARVEST_READY_STATES, MAX_GROWTH_STATE } from "@/lib/constants";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Separator } from "@/components/ui/separator";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Sprout, Sparkles, Bug, Gem, Droplets, FlaskConical, Tractor, Shovel, Wheat } from "@lucide/vue";

interface Props {
  field: Field | null;
  open: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  "update:open": [value: boolean];
}>();

const { t } = useI18n();
const store = useFieldStore();
const settings = useSettingsStore();

const original = computed(() =>
  props.field ? store.getOriginalFieldById(props.field.id) : undefined,
);

const farmland = computed(() =>
  props.field ? store.getFarmlandByFieldId(props.field.id) : undefined,
);

const trade = useFarmlandTrade();
const farmlandInfo = computed(() =>
  props.field ? store.getFarmlandInfo(props.field.id) : undefined,
);

const density = computed(() =>
  props.field ? store.getFieldDensity(props.field.id) : undefined,
);

/** Harvest-ready growth state of the field's fruit, when the game defines one. */
const readyState = computed(() =>
  props.field ? HARVEST_READY_STATES[props.field.fruitType] : undefined,
);

/** Known fruit types plus the field's own values, which may come from a map mod. */
const fruitOptions = computed(() => {
  const options: string[] = [...FRUIT_TYPES];
  for (const value of [props.field?.fruitType, props.field?.plannedFruit]) {
    if (value && !options.includes(value)) options.push(value);
  }
  return options;
});

function fieldModifiedClass(key: keyof Field): string {
  if (!props.field || !original.value) return "";
  return props.field[key] !== original.value[key]
    ? "border-l-2 border-amber-500 pl-2"
    : "";
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleFruitTypeChange(value: any) {
  if (props.field) {
    store.updateField(props.field.id, { fruitType: String(value) });
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handlePlannedFruitChange(value: any) {
  if (props.field) {
    store.updateField(props.field.id, { plannedFruit: String(value) });
  }
}

function handleGrowthChange(value: number[] | undefined) {
  if (value && props.field) {
    store.updateField(props.field.id, { growthState: value[0] });
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleGroundTypeChange(value: any) {
  if (props.field) {
    store.updateField(props.field.id, { groundType: String(value) });
  }
}

function handleMaxGrowth() {
  if (props.field && readyState.value !== undefined) {
    store.updateField(props.field.id, { growthState: readyState.value });
  }
}

function handleSliderChange(key: keyof Field, value: number[] | undefined) {
  if (value && props.field) {
    store.updateField(props.field.id, { [key]: value[0] } as Partial<Field>);
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleOwnerChange(value: any) {
  if (props.field) {
    store.updateFarmland(props.field.id, Number(value));
  }
}

// Density edit actions
function handleDensityMaxGrowth() {
  if (!props.field || !density.value?.dominantFruit) return;
  // Each crop pixel keeps its fruit and moves to that fruit's own ready state
  store.addDensityEdit(props.field.id, { setHarvestReady: true });
  if (readyState.value !== undefined) {
    store.updateField(props.field.id, { growthState: readyState.value });
  }
}

function handleDensityMaxLime() {
  if (!props.field) return;
  store.addDensityEdit(props.field.id, { setLimeLevel: 3 });
  store.updateField(props.field.id, { limeLevel: 3 });
}

function handleDensityMaxSpray() {
  if (!props.field) return;
  store.addDensityEdit(props.field.id, { setSprayLevel: 2 });
  store.updateField(props.field.id, { sprayLevel: 2 });
}

function handleDensityMaxPlow() {
  if (!props.field) return;
  store.addDensityEdit(props.field.id, { setPlowLevel: 1 });
  store.updateField(props.field.id, { plowLevel: 1 });
}

function handleDensityClearWeeds() {
  if (!props.field) return;
  store.addDensityEdit(props.field.id, { clearWeeds: true });
  store.updateField(props.field.id, { weedState: 0 });
}

function handleDensityClearStones() {
  if (!props.field) return;
  store.addDensityEdit(props.field.id, { clearStones: true });
  store.updateField(props.field.id, { stoneLevel: 0 });
}

const hasPendingDensityEdit = computed(() => {
  if (!props.field) return false;
  return store.densityEdits.has(props.field.id);
});

function stageLabel(fruit: MapFruitType | undefined, state: number): string {
  const { key, params } = growthStageLabel(fruit, state);
  return t(key, params);
}

// Crop picker: a fruit of the map and one of its growth states
const cropChoice = ref("NONE");
const stageChoice = ref(1);
const chosenFruit = computed(() => store.getMapFruitType(cropChoice.value));
const sortedFruitTypes = computed(() =>
  [...store.mapFruitTypes].sort((a, b) => formatFruitName(a.name).localeCompare(formatFruitName(b.name))),
);

function resetCropChoice() {
  const dominant = density.value?.dominantFruit;
  const fruit = (dominant ? store.getMapFruitType(dominant) : undefined) ?? store.mapFruitTypes[0];
  cropChoice.value = fruit?.name ?? "NONE";
  stageChoice.value = fruit?.harvestReadyState ?? 1;
}
watch(() => [props.field?.id, store.mapFruitTypes.length], resetCropChoice, { immediate: true });

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleCropChoice(value: any) {
  cropChoice.value = String(value);
  stageChoice.value = chosenFruit.value?.harvestReadyState ?? 1;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleStageChoice(value: any) {
  stageChoice.value = Number(value);
}

function handleApplyCrop() {
  if (!props.field) return;
  store.addDensityEdit(props.field.id, cropEdit(cropChoice.value, stageChoice.value));
}

function handleSoilWork(work: SoilWork) {
  if (!props.field) return;
  store.addDensityEdit(props.field.id, soilWorkEdit(work));
}

/** Labels of the density edits waiting to be saved for this field. */
const pendingItems = computed<string[]>(() => {
  const edit = props.field ? store.densityEdits.get(props.field.id) : undefined;
  if (!edit) return [];
  const items: string[] = [];
  if (edit.setFruitName === "NONE") {
    items.push(t("field.pendingNoCrop"));
  } else if (edit.setFruitName) {
    const fruit = store.getMapFruitType(edit.setFruitName);
    items.push(t("field.pendingCrop", {
      fruit: formatFruitName(edit.setFruitName),
      stage: stageLabel(fruit, edit.setGrowthState ?? 0),
    }));
  }
  if (edit.setHarvestReady) items.push(t("field.maxGrowth"));
  if (edit.setGroundType !== undefined) {
    const ground = GROUND_TYPES[edit.setGroundType - 1];
    items.push(t("field.pendingGround", { ground: t(`groundTypes.${ground}`, ground ?? "NONE") }));
  }
  if (edit.setLimeLevel !== undefined) items.push(t("field.maxLime"));
  if (edit.setSprayLevel !== undefined) items.push(t("field.maxFertilizer"));
  if (edit.setPlowLevel !== undefined) items.push(t("field.maxPlow"));
  if (edit.setRollerLevel !== undefined) items.push(t("field.soilRoll"));
  if (edit.setStubbleShredLevel !== undefined) items.push(t("field.soilMulch"));
  if (edit.clearWeeds) items.push(t("field.removeWeeds"));
  if (edit.clearStones) items.push(t("field.removeStones"));
  return items;
});

function formatFruitName(name: string): string {
  const unknownMatch = name.match(/^UNKNOWN_(\d+)$/);
  if (unknownMatch) {
    return t("field.unknownFruit", { index: unknownMatch[1] });
  }
  return t(`fillTypes.${name}`, name);
}
</script>

<template>
  <Sheet :open="open" @update:open="emit('update:open', $event)">
    <SheetContent class="overflow-y-auto sm:max-w-md">
      <template v-if="field">
        <SheetHeader>
          <SheetTitle class="flex items-center gap-2">
            <Sprout class="size-5" />
            {{ t("field.title") }} #{{ field.id }}
          </SheetTitle>
          <SheetDescription>
            <template v-if="density?.dominantFruit">{{ formatFruitName(density.dominantFruit) }}</template>
            <template v-else>{{ field.fruitType !== "UNKNOWN" ? t(`fillTypes.${field.fruitType}`, field.fruitType) : t("field.none") }}</template>
          </SheetDescription>
        </SheetHeader>

        <div class="mt-6 space-y-6">
          <!-- Density map data (actual field state with edit actions) -->
          <template v-if="density">
            <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
              {{ t("field.densityTitle") }}
            </p>

            <!-- Pending edit indicator -->
            <div v-if="hasPendingDensityEdit" class="rounded-md bg-amber-500/10 px-3 py-2 text-xs text-amber-600 dark:text-amber-400">
              {{ t("field.densityEditPending") }}
              <ul v-if="pendingItems.length > 0" class="mt-1 list-inside list-disc">
                <li v-for="item in pendingItems" :key="item">{{ item }}</li>
              </ul>
            </div>

            <!-- Crop distribution -->
            <div class="space-y-2">
              <Label>{{ t("field.fruitDistribution") }}</Label>
              <div v-if="density.fruitDistribution.length > 0" class="space-y-1.5">
                <div
                  v-for="fc in density.fruitDistribution"
                  :key="fc.fruitType"
                  class="flex items-center gap-2"
                >
                  <span class="w-24 truncate text-sm">{{ formatFruitName(fc.fruitType) }}</span>
                  <div class="h-2 flex-1 rounded-full bg-muted">
                    <div class="h-2 rounded-full bg-green-500" :style="{ width: fc.percentage + '%' }" />
                  </div>
                  <span class="w-28 truncate text-xs text-muted-foreground">
                    {{ stageLabel(store.getMapFruitType(fc.fruitType), Math.round(fc.avgGrowth)) }}
                  </span>
                  <span class="w-12 text-right font-mono text-xs text-muted-foreground">{{ fc.percentage.toFixed(0) }}%</span>
                </div>
              </div>
              <p v-else class="text-sm text-muted-foreground">{{ t("field.none") }}</p>
            </div>

            <!-- Treatment levels with edit buttons -->
            <div class="grid grid-cols-2 gap-3 text-sm">
              <div class="flex items-center justify-between rounded-md bg-muted/50 px-3 py-2">
                <span>{{ t("field.limeLevel") }}</span>
                <span :class="density.limeStatus.pctAtZero > 50 ? 'font-medium text-red-500' : 'text-green-600 dark:text-green-400'">
                  {{ (100 - density.limeStatus.pctAtZero).toFixed(0) }}%
                </span>
              </div>
              <div class="flex items-center justify-between rounded-md bg-muted/50 px-3 py-2">
                <span>{{ t("field.sprayLevel") }}</span>
                <span :class="density.sprayStatus.pctAtZero > 50 ? 'font-medium text-red-500' : 'text-green-600 dark:text-green-400'">
                  {{ (100 - density.sprayStatus.pctAtZero).toFixed(0) }}%
                </span>
              </div>
              <div class="flex items-center justify-between rounded-md bg-muted/50 px-3 py-2">
                <span>{{ t("field.plowLevel") }}</span>
                <span :class="density.plowStatus.pctAtZero > 50 ? 'font-medium text-red-500' : 'text-green-600 dark:text-green-400'">
                  {{ (100 - density.plowStatus.pctAtZero).toFixed(0) }}%
                </span>
              </div>
              <div class="flex items-center justify-between rounded-md bg-muted/50 px-3 py-2">
                <span>{{ t("field.weedState") }}</span>
                <span :class="density.weedCoverage > 30 ? 'font-medium text-red-500' : 'text-green-600 dark:text-green-400'">
                  {{ density.weedCoverage.toFixed(0) }}%
                </span>
              </div>
            </div>

            <!-- Density edit quick actions -->
            <div class="space-y-2">
              <Label>{{ t("field.densityActions") }}</Label>
              <div class="flex flex-wrap gap-2">
                <Button v-if="density.dominantFruit" variant="outline" size="sm" @click="handleDensityMaxGrowth">
                  <Sparkles class="size-4" />
                  {{ t("field.maxGrowth") }}
                </Button>
                <Button v-if="density.limeStatus.pctAtZero > 0" variant="outline" size="sm" @click="handleDensityMaxLime">
                  <Droplets class="size-4" />
                  {{ t("field.maxLime") }}
                </Button>
                <Button v-if="density.sprayStatus.pctAtZero > 0" variant="outline" size="sm" @click="handleDensityMaxSpray">
                  <FlaskConical class="size-4" />
                  {{ t("field.maxFertilizer") }}
                </Button>
                <Button v-if="density.plowStatus.pctAtZero > 0" variant="outline" size="sm" @click="handleDensityMaxPlow">
                  <Tractor class="size-4" />
                  {{ t("field.maxPlow") }}
                </Button>
                <Button v-if="density.weedCoverage > 0" variant="outline" size="sm" @click="handleDensityClearWeeds">
                  <Bug class="size-4" />
                  {{ t("field.removeWeeds") }}
                </Button>
                <Button v-if="density.stoneCoverage > 0" variant="outline" size="sm" @click="handleDensityClearStones">
                  <Gem class="size-4" />
                  {{ t("field.removeStones") }}
                </Button>
              </div>
            </div>

            <!-- Crop picker: fruit and growth stage from the map's own crops -->
            <div v-if="store.mapFruitTypes.length > 0" class="space-y-2">
              <Label>{{ t("field.cropTitle") }}</Label>
              <div class="grid grid-cols-2 gap-2">
                <Select :model-value="cropChoice" @update:model-value="handleCropChoice">
                  <SelectTrigger :aria-label="t('field.cropFruit')">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="NONE">{{ t("field.noCrop") }}</SelectItem>
                    <SelectItem v-for="fruit in sortedFruitTypes" :key="fruit.name" :value="fruit.name">
                      {{ formatFruitName(fruit.name) }}
                    </SelectItem>
                  </SelectContent>
                </Select>
                <Select :model-value="String(stageChoice)" :disabled="!chosenFruit" @update:model-value="handleStageChoice">
                  <SelectTrigger :aria-label="t('field.cropStage')">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem
                      v-for="(_, i) in chosenFruit?.states ?? []"
                      :key="i"
                      :value="String(i + 1)"
                    >
                      {{ i + 1 }} · {{ stageLabel(chosenFruit, i + 1) }}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <Button variant="outline" size="sm" @click="handleApplyCrop">
                <Wheat class="size-4" />
                {{ t("field.applyCrop") }}
              </Button>
            </div>

            <!-- Soil work presets -->
            <div class="space-y-2">
              <Label>{{ t("field.soilTitle") }}</Label>
              <div class="flex flex-wrap gap-2">
                <Button variant="outline" size="sm" @click="handleSoilWork('plow')">
                  <Tractor class="size-4" />
                  {{ t("field.soilPlow") }}
                </Button>
                <Button variant="outline" size="sm" @click="handleSoilWork('cultivate')">
                  <Shovel class="size-4" />
                  {{ t("field.soilCultivate") }}
                </Button>
                <Button v-if="density.rollerStatus.pctAtZero < 100" variant="outline" size="sm" @click="handleSoilWork('roll')">
                  <Tractor class="size-4" />
                  {{ t("field.soilRoll") }}
                </Button>
                <Button v-if="density.stubbleShredStatus.pctAtMax < 100" variant="outline" size="sm" @click="handleSoilWork('mulch')">
                  <Shovel class="size-4" />
                  {{ t("field.soilMulch") }}
                </Button>
              </div>
            </div>

            <template v-if="settings.advancedMode">
              <Separator />
              <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                {{ t("field.xmlEditTitle") }}
              </p>
            </template>
          </template>

          <!-- XML fields: show always when no density, or behind advanced mode when density exists -->
          <template v-if="!density || settings.advancedMode">
          <!-- Fruit type -->
          <div class="space-y-2" :class="fieldModifiedClass('fruitType')">
            <Label>{{ t("field.fruitType") }}</Label>
            <Select :model-value="field.fruitType" @update:model-value="handleFruitTypeChange">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="fruit in fruitOptions" :key="fruit" :value="fruit">
                  {{ t(`fillTypes.${fruit}`, fruit) }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <!-- Planned fruit -->
          <div class="space-y-2" :class="fieldModifiedClass('plannedFruit')">
            <Label>{{ t("field.plannedFruit") }}</Label>
            <Select :model-value="field.plannedFruit" @update:model-value="handlePlannedFruitChange">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="fruit in fruitOptions" :key="fruit" :value="fruit">
                  {{ t(`fillTypes.${fruit}`, fruit) }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <!-- Growth state -->
          <div class="space-y-2" :class="fieldModifiedClass('growthState')">
            <div class="flex items-center justify-between text-sm">
              <Label>{{ t("field.growthState") }}</Label>
              <span class="font-mono text-muted-foreground">
                {{ field.growthState }} / {{ readyState ?? MAX_GROWTH_STATE }}
              </span>
            </div>
            <Slider
              :model-value="[field.growthState]"
              :max="MAX_GROWTH_STATE"
              :min="0"
              :step="1"
              @update:model-value="handleGrowthChange"
            />
            <Button variant="outline" size="sm" :disabled="readyState === undefined" @click="handleMaxGrowth">
              <Sparkles class="size-4" />
              {{ t("field.maxGrowth") }}
            </Button>
          </div>

          <!-- Ground type -->
          <div class="space-y-2" :class="fieldModifiedClass('groundType')">
            <Label>{{ t("field.groundType") }}</Label>
            <Select :model-value="field.groundType" @update:model-value="handleGroundTypeChange">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="gt in GROUND_TYPES" :key="gt" :value="gt">
                  {{ t(`groundTypes.${gt}`, gt) }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          </template>

          <!-- Farmland owner (always visible, not from density maps) -->
          <div v-if="farmland" class="space-y-2">
            <Label>{{ t("field.farmlandOwner") }}</Label>
            <div v-if="farmlandInfo" class="flex flex-wrap items-center justify-between gap-2 text-sm">
              <span class="font-mono text-muted-foreground">
                {{ farmlandInfo.areaHa.toFixed(2) }} ha · {{ formatMoney(farmlandInfo.price) }}
              </span>
              <Button
                v-if="farmland.farmId === 0"
                size="sm"
                variant="outline"
                :disabled="!!trade.buyBlock([farmland.id])"
                :title="trade.buyBlock([farmland.id]) ? t(`farmland.block.${trade.buyBlock([farmland.id])}`) : ''"
                @click="trade.buy([farmland.id])"
              >
                {{ t("farmland.buy") }}
              </Button>
              <Button
                v-else-if="farmland.farmId === PLAYER_FARM_ID"
                size="sm"
                variant="outline"
                :disabled="!!trade.sellBlock([farmland.id])"
                :title="trade.sellBlock([farmland.id]) ? t(`farmland.block.${trade.sellBlock([farmland.id])}`) : ''"
                @click="trade.sell([farmland.id])"
              >
                {{ t("farmland.sell") }}
              </Button>
            </div>
            <p v-if="farmlandInfo && settings.advancedMode" class="text-xs text-muted-foreground">
              {{ t("field.freeTransfer") }}
            </p>
            <Select
              v-if="!farmlandInfo || settings.advancedMode"
              :model-value="String(farmland.farmId)"
              @update:model-value="handleOwnerChange"
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="0">{{ t("field.free") }}</SelectItem>
                <SelectItem value="1">{{ t("field.myFarm") }}</SelectItem>
                <SelectItem v-if="settings.advancedMode" v-for="i in [2,3,4,5,6]" :key="i" :value="String(i)">
                  {{ t("field.farm", { id: i }) }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <!-- Advanced: Treatment levels -->
          <template v-if="settings.advancedMode">
            <Separator />
            <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
              {{ t("field.treatments") }}
            </p>

            <div class="space-y-4">
              <!-- Weed state -->
              <div class="space-y-2" :class="fieldModifiedClass('weedState')">
                <div class="flex items-center justify-between text-sm">
                  <Label>{{ t("field.weedState") }}</Label>
                  <span class="font-mono text-muted-foreground">{{ field.weedState }}</span>
                </div>
                <Slider
                  :model-value="[field.weedState]"
                  :max="9"
                  :min="0"
                  :step="1"
                  @update:model-value="handleSliderChange('weedState', $event)"
                />
              </div>

              <!-- Stone level -->
              <div class="space-y-2" :class="fieldModifiedClass('stoneLevel')">
                <div class="flex items-center justify-between text-sm">
                  <Label>{{ t("field.stoneLevel") }}</Label>
                  <span class="font-mono text-muted-foreground">{{ field.stoneLevel }}</span>
                </div>
                <Slider
                  :model-value="[field.stoneLevel]"
                  :max="5"
                  :min="0"
                  :step="1"
                  @update:model-value="handleSliderChange('stoneLevel', $event)"
                />
              </div>

              <!-- Spray level -->
              <div class="space-y-2" :class="fieldModifiedClass('sprayLevel')">
                <div class="flex items-center justify-between text-sm">
                  <Label>{{ t("field.sprayLevel") }}</Label>
                  <span class="font-mono text-muted-foreground">{{ field.sprayLevel }}</span>
                </div>
                <Slider
                  :model-value="[field.sprayLevel]"
                  :max="2"
                  :min="0"
                  :step="1"
                  @update:model-value="handleSliderChange('sprayLevel', $event)"
                />
              </div>

              <!-- Lime level -->
              <div class="space-y-2" :class="fieldModifiedClass('limeLevel')">
                <div class="flex items-center justify-between text-sm">
                  <Label>{{ t("field.limeLevel") }}</Label>
                  <span class="font-mono text-muted-foreground">{{ field.limeLevel }}</span>
                </div>
                <Slider
                  :model-value="[field.limeLevel]"
                  :max="3"
                  :min="0"
                  :step="1"
                  @update:model-value="handleSliderChange('limeLevel', $event)"
                />
              </div>

              <!-- Plow level -->
              <div class="space-y-2" :class="fieldModifiedClass('plowLevel')">
                <div class="flex items-center justify-between text-sm">
                  <Label>{{ t("field.plowLevel") }}</Label>
                  <span class="font-mono text-muted-foreground">{{ field.plowLevel }}</span>
                </div>
                <Slider
                  :model-value="[field.plowLevel]"
                  :max="1"
                  :min="0"
                  :step="1"
                  @update:model-value="handleSliderChange('plowLevel', $event)"
                />
              </div>

              <!-- Roller level -->
              <div class="space-y-2" :class="fieldModifiedClass('rollerLevel')">
                <div class="flex items-center justify-between text-sm">
                  <Label>{{ t("field.rollerLevel") }}</Label>
                  <span class="font-mono text-muted-foreground">{{ field.rollerLevel }}</span>
                </div>
                <Slider
                  :model-value="[field.rollerLevel]"
                  :max="1"
                  :min="0"
                  :step="1"
                  @update:model-value="handleSliderChange('rollerLevel', $event)"
                />
              </div>

              <!-- Stubble shred level -->
              <div class="space-y-2" :class="fieldModifiedClass('stubbleShredLevel')">
                <div class="flex items-center justify-between text-sm">
                  <Label>{{ t("field.stubbleShredLevel") }}</Label>
                  <span class="font-mono text-muted-foreground">{{ field.stubbleShredLevel }}</span>
                </div>
                <Slider
                  :model-value="[field.stubbleShredLevel]"
                  :max="1"
                  :min="0"
                  :step="1"
                  @update:model-value="handleSliderChange('stubbleShredLevel', $event)"
                />
              </div>

              <!-- Water level -->
              <div class="space-y-2" :class="fieldModifiedClass('waterLevel')">
                <div class="flex items-center justify-between text-sm">
                  <Label>{{ t("field.waterLevel") }}</Label>
                  <span class="font-mono text-muted-foreground">{{ field.waterLevel }}</span>
                </div>
                <Slider
                  :model-value="[field.waterLevel]"
                  :max="3"
                  :min="0"
                  :step="1"
                  @update:model-value="handleSliderChange('waterLevel', $event)"
                />
              </div>
            </div>
          </template>
        </div>
      </template>
    </SheetContent>
  </Sheet>
</template>
