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
import { AnimatePresence, motion } from "motion-v";
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
import { Sparkles, Tractor, Shovel, Wheat, ChevronUp, ChevronDown, Undo2 } from "@lucide/vue";
import { fruitColor } from "@/lib/fruitColors";
import WithTooltip from "@/components/WithTooltip.vue";

interface Props {
  field: Field | null;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  prev: [];
  next: [];
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

/** The crop the field shows: the density map's, else the XML's. */
const currentFruit = computed<string | null>(() => {
  if (!props.field) return null;
  if (density.value) return density.value.dominantFruit;
  return props.field.fruitType !== "UNKNOWN" ? props.field.fruitType : null;
});

const header = computed(() => {
  const fruit = currentFruit.value;
  if (!props.field || !fruit) return { crop: t("field.none"), stage: "", ready: false };
  const mapFruit = store.getMapFruitType(fruit);
  const dominant = density.value?.fruitDistribution[0];
  const state = dominant ? Math.round(dominant.avgGrowth) : props.field.growthState;
  const { key, params } = growthStageLabel(mapFruit, state);
  return { crop: formatFruitName(fruit), stage: t(key, params), ready: key === "growthStage.ready" };
});

interface SoilTile {
  key: string;
  label: string;
  value: number;
  ok: boolean;
  action?: { label: string; run: () => void };
}

/** Soil state from the density maps, each with the fix it offers. */
const soilTiles = computed<SoilTile[]>(() => {
  const d = density.value;
  if (!d) return [];
  return [
    {
      key: "lime",
      label: t("field.limeLevel"),
      value: 100 - d.limeStatus.pctAtZero,
      ok: d.limeStatus.pctAtZero <= 50,
      action: d.limeStatus.pctAtZero > 0 ? { label: t("field.maxLime"), run: handleDensityMaxLime } : undefined,
    },
    {
      key: "spray",
      label: t("field.sprayLevel"),
      value: 100 - d.sprayStatus.pctAtZero,
      ok: d.sprayStatus.pctAtZero <= 50,
      action: d.sprayStatus.pctAtZero > 0 ? { label: t("field.maxFertilizer"), run: handleDensityMaxSpray } : undefined,
    },
    {
      key: "plow",
      label: t("field.plowLevel"),
      value: 100 - d.plowStatus.pctAtZero,
      ok: d.plowStatus.pctAtZero <= 50,
      action: d.plowStatus.pctAtZero > 0 ? { label: t("field.maxPlow"), run: handleDensityMaxPlow } : undefined,
    },
    {
      key: "weeds",
      label: t("field.weedState"),
      value: d.weedCoverage,
      ok: d.weedCoverage <= 30,
      action: d.weedCoverage > 0 ? { label: t("field.removeWeeds"), run: handleDensityClearWeeds } : undefined,
    },
    {
      key: "stones",
      label: t("field.stoneLevel"),
      value: d.stoneCoverage,
      ok: d.stoneCoverage <= 30,
      action: d.stoneCoverage > 0 ? { label: t("field.removeStones"), run: handleDensityClearStones } : undefined,
    },
  ];
});

type LevelKey =
  | "weedState" | "stoneLevel" | "sprayLevel" | "limeLevel"
  | "plowLevel" | "rollerLevel" | "stubbleShredLevel" | "waterLevel";

/** XML treatment levels (advanced mode) with the game's maximum for each. */
const xmlLevels: { key: LevelKey; labelKey: string; max: number }[] = [
  { key: "weedState", labelKey: "field.weedState", max: 9 },
  { key: "stoneLevel", labelKey: "field.stoneLevel", max: 5 },
  { key: "sprayLevel", labelKey: "field.sprayLevel", max: 2 },
  { key: "limeLevel", labelKey: "field.limeLevel", max: 3 },
  { key: "plowLevel", labelKey: "field.plowLevel", max: 1 },
  { key: "rollerLevel", labelKey: "field.rollerLevel", max: 1 },
  { key: "stubbleShredLevel", labelKey: "field.stubbleShredLevel", max: 1 },
  { key: "waterLevel", labelKey: "field.waterLevel", max: 3 },
];

function formatFruitName(name: string): string {
  const unknownMatch = name.match(/^UNKNOWN_(\d+)$/);
  if (unknownMatch) {
    return t("field.unknownFruit", { index: unknownMatch[1] });
  }
  return t(`fillTypes.${name}`, name);
}
</script>

<template>
  <section
    v-if="field"
    :aria-labelledby="`field-title-${field.id}`"
    class="flex flex-col gap-5"
  >
    <!-- Header -->
    <div class="flex items-start gap-3.5">
      <span
        class="size-12 shrink-0 rounded-[13px] transition-colors duration-300"
        :style="{ background: fruitColor(currentFruit) }"
        aria-hidden="true"
      />
      <div class="flex min-w-0 flex-1 flex-col gap-0.5">
        <h3 :id="`field-title-${field.id}`" class="text-2xl font-semibold tracking-tight">
          {{ t("farmland.fieldNumber", { id: field.id }) }}
        </h3>
        <p class="truncate text-muted-foreground">
          {{ header.crop }}<template v-if="header.stage"> ·
            <span class="font-semibold" :class="header.ready ? 'text-ready' : 'text-primary-text'">{{ header.stage }}</span>
          </template>
        </p>
      </div>
      <div class="flex shrink-0 gap-0.5">
        <WithTooltip :label="t('fieldUi.prev')">
          <Button variant="ghost" size="icon" class="size-9" :aria-label="t('fieldUi.prev')" @click="emit('prev')">
            <ChevronUp class="size-4" />
          </Button>
        </WithTooltip>
        <WithTooltip :label="t('fieldUi.next')">
          <Button variant="ghost" size="icon" class="size-9" :aria-label="t('fieldUi.next')" @click="emit('next')">
            <ChevronDown class="size-4" />
          </Button>
        </WithTooltip>
      </div>
    </div>

    <!-- Pending edits -->
    <AnimatePresence>
      <motion.div
        v-if="hasPendingDensityEdit"
        :initial="{ opacity: 0, scale: 0.96, height: 0 }"
        :animate="{ opacity: 1, scale: 1, height: 'auto' }"
        :exit="{ opacity: 0, scale: 0.96, height: 0 }"
        :transition="{ type: 'spring', stiffness: 420, damping: 36 }"
        class="overflow-hidden"
      >
        <div class="flex items-start gap-3 rounded-xl bg-ready-soft px-3.5 py-3 text-ready">
          <div class="min-w-0 flex-1">
            <p class="text-sm font-semibold">{{ t("fieldUi.pendingTitle") }}</p>
            <ul v-if="pendingItems.length > 0" class="mt-1 space-y-0.5 text-[13px]">
              <li v-for="item in pendingItems" :key="item">{{ item }}</li>
            </ul>
          </div>
          <Button
            variant="outline"
            size="sm"
            class="h-8 shrink-0 border-current bg-transparent text-current hover:bg-ready-soft hover:text-current"
            @click="store.discardFieldEdits(field.id)"
          >
            <Undo2 class="size-3.5" />
            {{ t("dirtyTracking.discardChanges") }}
          </Button>
        </div>
      </motion.div>
    </AnimatePresence>

    <!-- Density map data (actual field state with edit actions) -->
    <template v-if="density">
      <!-- Crop distribution -->
      <div class="space-y-2.5">
        <div class="flex items-center justify-between gap-2">
          <h4 class="text-[13px] font-semibold text-muted-foreground">{{ t("field.fruitDistribution") }}</h4>
          <Button
            v-if="density.dominantFruit && !header.ready"
            size="sm"
            class="h-8"
            @click="handleDensityMaxGrowth"
          >
            <Sparkles class="size-4" />
            {{ t("field.maxGrowth") }}
          </Button>
        </div>
        <ul v-if="density.fruitDistribution.length > 0" class="space-y-2">
          <li
            v-for="fc in density.fruitDistribution"
            :key="fc.fruitType"
            class="flex items-center gap-2.5 text-sm"
          >
            <span class="size-2.5 shrink-0 rounded-[3px]" :style="{ background: fruitColor(fc.fruitType) }" aria-hidden="true" />
            <span class="w-24 truncate font-medium">{{ formatFruitName(fc.fruitType) }}</span>
            <span class="h-1.5 flex-1 overflow-hidden rounded-full bg-border" aria-hidden="true">
              <motion.span
                class="block h-full rounded-full"
                :style="{ background: fruitColor(fc.fruitType) }"
                :initial="{ width: 0 }"
                :animate="{ width: fc.percentage + '%' }"
                :transition="{ type: 'spring', stiffness: 160, damping: 26 }"
              />
            </span>
            <span class="w-28 truncate text-xs text-muted-foreground">
              {{ stageLabel(store.getMapFruitType(fc.fruitType), Math.round(fc.avgGrowth)) }}
            </span>
            <span class="w-10 text-right text-xs text-muted-foreground tabular">{{ fc.percentage.toFixed(0) }}%</span>
          </li>
        </ul>
        <p v-else class="text-sm text-muted-foreground">{{ t("field.none") }}</p>
      </div>

      <!-- Crop picker: fruit and growth stage from the map's own crops -->
      <div v-if="store.mapFruitTypes.length > 0" class="space-y-2.5">
        <h4 class="text-[13px] font-semibold text-muted-foreground">{{ t("fieldUi.changeCrop") }}</h4>
        <Select :model-value="cropChoice" @update:model-value="handleCropChoice">
          <SelectTrigger class="w-full" :aria-label="t('field.cropFruit')">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="NONE">{{ t("field.noCrop") }}</SelectItem>
            <SelectItem v-for="fruit in sortedFruitTypes" :key="fruit.name" :value="fruit.name">
              {{ formatFruitName(fruit.name) }}
            </SelectItem>
          </SelectContent>
        </Select>
        <div
          v-if="chosenFruit"
          role="radiogroup"
          :aria-label="t('field.cropStage')"
          class="flex flex-wrap gap-1.5"
        >
          <button
            v-for="(_, i) in chosenFruit.states"
            :key="i"
            type="button"
            role="radio"
            :aria-checked="stageChoice === i + 1"
            class="relative h-8 rounded-lg border px-2.5 text-[13px] transition-colors"
            :class="stageChoice === i + 1 ? 'border-transparent font-semibold text-primary-text' : 'hover:bg-hover'"
            @click="handleStageChoice(i + 1)"
          >
            <motion.span
              v-if="stageChoice === i + 1"
              layout-id="field-stage-choice"
              class="absolute inset-0 rounded-lg bg-primary-soft ring-1 ring-primary/40"
              :transition="{ type: 'spring', stiffness: 520, damping: 40 }"
            />
            <span class="relative">{{ i + 1 }} · {{ stageLabel(chosenFruit, i + 1) }}</span>
          </button>
        </div>
        <Button variant="outline" size="sm" class="h-9" @click="handleApplyCrop">
          <Wheat class="size-4" />
          {{ t("field.applyCrop") }}
        </Button>
      </div>

      <!-- Soil state, each with its fix -->
      <div class="space-y-2.5">
        <h4 class="text-[13px] font-semibold text-muted-foreground">{{ t("fieldUi.soil") }}</h4>
        <ul class="grid grid-cols-[repeat(auto-fill,minmax(min(100%,150px),1fr))] gap-1.5">
          <li
            v-for="tile in soilTiles"
            :key="tile.key"
            class="flex min-h-12 flex-wrap items-center gap-x-2.5 gap-y-1 rounded-xl bg-card py-1.5 pl-3 pr-1.5"
          >
            <span
              class="size-[9px] shrink-0"
              :class="tile.ok ? 'rounded-full bg-primary' : 'rounded-[2px] bg-destructive'"
              aria-hidden="true"
            />
            <span class="flex min-w-[96px] flex-1 flex-col leading-tight">
              <span class="text-sm font-medium">{{ tile.label }}</span>
              <span class="text-xs tabular" :class="tile.ok ? 'text-muted-foreground' : 'text-destructive'">
                {{ tile.value.toFixed(0) }}%
                <span class="sr-only">{{ tile.ok ? t("fieldUi.ok") : t("fieldUi.toDo") }}</span>
              </span>
            </span>
            <button
              v-if="tile.action"
              type="button"
              class="ml-auto h-8 shrink-0 rounded-lg bg-primary-soft px-2.5 text-[13px] font-semibold text-primary-text transition-colors hover:bg-primary hover:text-primary-foreground"
              @click="tile.action.run"
            >
              {{ tile.action.label }}
            </button>
          </li>
        </ul>
      </div>

      <!-- Soil work presets -->
      <div class="space-y-2.5">
        <h4 class="text-[13px] font-semibold text-muted-foreground">{{ t("field.soilTitle") }}</h4>
        <div class="flex flex-wrap gap-1.5">
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
        <h4 class="text-[13px] font-semibold text-muted-foreground">{{ t("field.xmlEditTitle") }}</h4>
      </template>
    </template>

    <!-- XML fields: always without density, behind advanced mode with it -->
    <template v-if="!density || settings.advancedMode">
      <div class="space-y-2" :class="fieldModifiedClass('fruitType')">
        <Label>{{ t("field.fruitType") }}</Label>
        <Select :model-value="field.fruitType" @update:model-value="handleFruitTypeChange">
          <SelectTrigger class="w-full">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="fruit in fruitOptions" :key="fruit" :value="fruit">
              {{ t(`fillTypes.${fruit}`, fruit) }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div class="space-y-2" :class="fieldModifiedClass('plannedFruit')">
        <Label>{{ t("field.plannedFruit") }}</Label>
        <Select :model-value="field.plannedFruit" @update:model-value="handlePlannedFruitChange">
          <SelectTrigger class="w-full">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="fruit in fruitOptions" :key="fruit" :value="fruit">
              {{ t(`fillTypes.${fruit}`, fruit) }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div class="space-y-2" :class="fieldModifiedClass('growthState')">
        <div class="flex items-center justify-between text-sm">
          <Label>{{ t("field.growthState") }}</Label>
          <span class="text-muted-foreground tabular">
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

      <div class="space-y-2" :class="fieldModifiedClass('groundType')">
        <Label>{{ t("field.groundType") }}</Label>
        <Select :model-value="field.groundType" @update:model-value="handleGroundTypeChange">
          <SelectTrigger class="w-full">
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

    <!-- Advanced: treatment levels -->
    <template v-if="settings.advancedMode">
      <Separator />
      <h4 class="text-[13px] font-semibold text-muted-foreground">{{ t("field.treatments") }}</h4>
      <div class="space-y-4">
        <div
          v-for="lvl in xmlLevels"
          :key="lvl.key"
          class="space-y-2"
          :class="fieldModifiedClass(lvl.key)"
        >
          <div class="flex items-center justify-between text-sm">
            <Label>{{ t(lvl.labelKey) }}</Label>
            <span class="text-muted-foreground tabular">{{ field[lvl.key] }}</span>
          </div>
          <Slider
            :model-value="[field[lvl.key] as number]"
            :max="lvl.max"
            :min="0"
            :step="1"
            @update:model-value="handleSliderChange(lvl.key, $event)"
          />
        </div>
      </div>
    </template>

    <!-- Farmland: area, price, buy or sell -->
    <div v-if="farmland" class="mt-auto space-y-2.5 border-t pt-4">
      <div class="flex flex-wrap items-center gap-3">
        <div class="flex min-w-0 flex-1 flex-col leading-tight">
          <span class="font-semibold">
            {{ t("farmland.id") }} {{ farmland.id }}<template v-if="farmlandInfo"> · {{ farmlandInfo.areaHa.toFixed(2) }} ha</template>
          </span>
          <span class="text-[13px] text-muted-foreground">
            {{ farmland.farmId === PLAYER_FARM_ID ? t("field.myFarm") : farmland.farmId === 0 ? t("field.free") : t("field.farm", { id: farmland.farmId }) }}<template v-if="farmlandInfo"> · {{ formatMoney(farmlandInfo.price) }}</template>
          </span>
        </div>
        <template v-if="farmlandInfo">
          <Button
            v-if="farmland.farmId === 0"
            size="sm"
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
        </template>
      </div>
      <p v-if="farmlandInfo && settings.advancedMode" class="text-xs text-muted-foreground">
        {{ t("field.freeTransfer") }}
      </p>
      <Select
        v-if="!farmlandInfo || settings.advancedMode"
        :model-value="String(farmland.farmId)"
        @update:model-value="handleOwnerChange"
      >
        <SelectTrigger class="w-full" :aria-label="t('field.farmlandOwner')">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="0">{{ t("field.free") }}</SelectItem>
          <SelectItem value="1">{{ t("field.myFarm") }}</SelectItem>
          <template v-if="settings.advancedMode">
            <SelectItem v-for="i in [2, 3, 4, 5, 6]" :key="i" :value="String(i)">
              {{ t("field.farm", { id: i }) }}
            </SelectItem>
          </template>
        </SelectContent>
      </Select>
    </div>
  </section>

  <div v-else class="flex h-full items-center justify-center p-8 text-center text-muted-foreground">
    {{ t("fieldUi.pick") }}
  </div>
</template>
