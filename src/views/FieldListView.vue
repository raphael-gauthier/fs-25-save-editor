<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";
import { AnimatePresence, motion } from "motion-v";
import { useFieldStore } from "@/stores/field";
import { useSettingsStore } from "@/stores/settings";
import type { Field } from "@/lib/types";
import { HARVEST_READY_STATES, MAX_GROWTH_STATE } from "@/lib/constants";
import { growthStageLabel } from "@/lib/utils";
import { fruitColor } from "@/lib/fruitColors";
import { soilWorkEdit, type SoilWork } from "@/lib/fieldWork";
import FieldEditor from "@/components/fields/FieldEditor.vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import { Spinner } from "@/components/ui/spinner";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { Alert, AlertDescription } from "@/components/ui/alert";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Sprout,
  Sparkles,
  Bug,
  Gem,
  Droplets,
  FlaskConical,
  Info,
  Tractor,
  Shovel,
  Search,
  Ellipsis,
  X,
} from "@lucide/vue";
import WithTooltip from "@/components/WithTooltip.vue";

const { t } = useI18n();
const store = useFieldStore();
const settings = useSettingsStore();

const route = useRoute();
const selectedId = ref<number | null>(null);

const selectedField = computed<Field | null>(() =>
  selectedId.value === null ? null : store.getFieldById(selectedId.value) ?? null,
);

const farmlandMap = computed(() =>
  new Map(store.farmlands.map((fl) => [fl.id, fl.farmId])),
);

// A field opened from the command palette: clear filters that would hide it
watch(
  () => route.query.field,
  (value) => {
    const id = Number(value);
    if (!value || Number.isNaN(id)) return;
    store.searchQuery = "";
    store.fruitFilter = null;
    store.ownerFilter = null;
    selectedId.value = id;
    nextTick(() => document.getElementById(`field-row-${id}`)?.scrollIntoView({ block: "center" }));
  },
  { immediate: true },
);

// Keep a field selected while the filters change
watch(
  () => store.filteredFields,
  (list) => {
    if (!list.some((f) => f.id === selectedId.value)) selectedId.value = list[0]?.id ?? null;
  },
  { immediate: true },
);

function step(delta: number, focus = false) {
  const list = store.filteredFields;
  if (list.length === 0) return;
  const i = list.findIndex((f) => f.id === selectedId.value);
  const next = list[(i + delta + list.length) % list.length];
  selectedId.value = next.id;
  nextTick(() => {
    const row = document.getElementById(`field-row-${next.id}`);
    row?.scrollIntoView({ block: "nearest" });
    if (focus) row?.focus();
  });
}

function onListKeydown(e: KeyboardEvent) {
  if (e.key === "ArrowDown" || e.key === "ArrowUp") {
    e.preventDefault();
    step(e.key === "ArrowDown" ? 1 : -1, true);
  }
}

const ownerSegments = computed<{ value: number | null; label: string }[]>(() => [
  { value: null, label: t("fieldUi.all") },
  { value: 1, label: t("fieldUi.mine") },
  { value: 0, label: t("field.free") },
]);

const myFieldCount = computed(
  () => store.fields.filter((f) => farmlandMap.value.get(f.id) === 1).length,
);

function formatFruitName(name: string): string {
  const unknownMatch = name.match(/^UNKNOWN_(\d+)$/);
  if (unknownMatch) {
    return t("field.unknownFruit", { index: unknownMatch[1] });
  }
  return t(`fillTypes.${name}`, name);
}

/** Growth as a share of the fruit's harvest-ready state (100% = ready). */
function growthPercent(state: number, readyState: number | null | undefined): number {
  return Math.round((state / (readyState ?? MAX_GROWTH_STATE)) * 100);
}

function densityFruit(fieldId: number): string {
  const d = store.getFieldDensity(fieldId);
  if (!d || !d.dominantFruit) return t("field.none");
  return formatFruitName(d.dominantFruit);
}

function densityGrowthPercent(fieldId: number): number {
  const d = store.getFieldDensity(fieldId);
  const dominant = d?.fruitDistribution[0];
  if (!d || !dominant) return 0;
  return growthPercent(
    d.avgGrowthState,
    dominant.harvestReadyState ?? HARVEST_READY_STATES[dominant.fruitType],
  );
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

interface FieldRow {
  id: number;
  crop: string;
  color: string;
  stage: string;
  ready: boolean;
  pct: number;
  issues: string | null;
  mine: boolean;
  pending: boolean;
}

/** What each row shows: density data when there is some, else the XML. */
function fieldRow(field: Field): FieldRow {
  const d = store.getFieldDensity(field.id);
  const mine = farmlandMap.value.get(field.id) === 1;
  const pending = store.densityEdits.has(field.id);
  if (d) {
    const dominant = d.fruitDistribution[0];
    const fruit = d.dominantFruit;
    const label = fruit && dominant
      ? growthStageLabel(store.getMapFruitType(fruit), Math.round(dominant.avgGrowth))
      : null;
    return {
      id: field.id,
      crop: densityFruit(field.id),
      color: fruitColor(fruit),
      stage: label ? t(label.key, label.params) : "",
      ready: label?.key === "growthStage.ready",
      pct: fruit ? Math.min(densityGrowthPercent(field.id), 100) : 0,
      issues: treatmentBadge(field.id)?.label ?? null,
      mine,
      pending,
    };
  }
  const fruit = field.fruitType !== "UNKNOWN" ? field.fruitType : null;
  const pct = fruit ? Math.min(growthPercent(field.growthState, HARVEST_READY_STATES[fruit]), 100) : 0;
  return {
    id: field.id,
    crop: fruit ? t(`fillTypes.${fruit}`, fruit) : t("field.none"),
    color: fruitColor(fruit),
    stage: fruit ? `${field.growthState}/${HARVEST_READY_STATES[fruit] ?? MAX_GROWTH_STATE}` : "",
    ready: pct >= 100,
    pct,
    issues: null,
    mine,
    pending,
  };
}

const rows = computed(() => store.filteredFields.map(fieldRow));

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleFruitFilter(value: any) {
  store.fruitFilter = value === "__all__" ? null : String(value);
}


function handleBatchMaxGrowth() {
  const ids = Array.from(store.selectedFieldIds);
  store.batchMaxGrowth(ids);
  if (store.hasDensityData) {
    for (const id of ids) {
      const d = store.getFieldDensity(id);
      if (d?.dominantFruit) {
        store.addDensityEdit(id, { setHarvestReady: true });
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

function handleBatchMaxPlow() {
  const ids = Array.from(store.selectedFieldIds);
  store.batchDensityEdit(ids, { setPlowLevel: 1 });
}

function handleBatchSoilWork(work: SoilWork) {
  store.batchDensityEdit(Array.from(store.selectedFieldIds), soilWorkEdit(work));
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
  <div class="grid h-full lg:grid-cols-[minmax(0,1fr)_minmax(340px,420px)]">
    <!-- List -->
    <section
      aria-labelledby="fields-title"
      class="relative flex min-h-0 min-w-0 flex-col gap-4 overflow-y-auto p-6 pb-0"
    >
      <div class="flex flex-wrap items-baseline justify-between gap-2">
        <h2 id="fields-title" class="text-2xl font-semibold tracking-tight">{{ t("field.title") }}</h2>
        <span class="text-[13px] text-muted-foreground">
          {{ t("fieldUi.summary", { count: store.fields.length, mine: myFieldCount }) }}
        </span>
      </div>

      <div
        v-if="store.densityLoading && !store.densityFromCache"
        role="status"
        class="flex items-center gap-2 text-sm text-muted-foreground"
      >
        <Spinner />
        {{ t("field.densityLoading") }}
      </div>

      <!-- Density data info/error banners -->
      <Alert v-if="store.densityError && !store.densityFromCache" variant="destructive">
        <Info class="size-4" />
        <AlertDescription>{{ t("field.densityError", { details: store.densityError }) }}</AlertDescription>
      </Alert>
      <Alert v-if="store.densityFromCache && store.densityLoading">
        <Spinner />
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
      <div class="flex flex-wrap items-center gap-2">
        <ToggleGroup
          type="single"
          :spacing="1"
          :aria-label="t('field.owner')"
          :model-value="String(store.ownerFilter)"
          class="rounded-[11px] bg-hover p-[3px]"
          @update:model-value="(v) => v && (store.ownerFilter = v === 'null' ? null : Number(v))"
        >
          <ToggleGroupItem
            v-for="seg in ownerSegments"
            :key="String(seg.value)"
            :value="String(seg.value)"
            class="relative h-8 rounded-lg px-3 text-[13px] text-muted-foreground hover:bg-transparent hover:text-foreground data-[state=on]:bg-transparent data-[state=on]:font-semibold data-[state=on]:text-foreground"
          >
            <motion.span
              v-if="store.ownerFilter === seg.value"
              layout-id="field-owner-segment"
              class="absolute inset-0 rounded-lg bg-background shadow-sm"
              :transition="{ type: 'spring', stiffness: 520, damping: 40 }"
            />
            <span class="relative">{{ seg.label }}</span>
          </ToggleGroupItem>
        </ToggleGroup>
        <Select
          :model-value="store.fruitFilter ?? '__all__'"
          @update:model-value="handleFruitFilter"
        >
          <SelectTrigger class="h-[38px] w-44" :aria-label="t('field.fruitPlaceholder')">
            <SelectValue :placeholder="t('field.fruitPlaceholder')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="__all__">{{ t("field.allFruits") }}</SelectItem>
            <SelectItem v-for="fruit in store.availableFruits" :key="fruit" :value="fruit">
              {{ formatFruitName(fruit) }}
            </SelectItem>
          </SelectContent>
        </Select>
        <div class="relative min-w-[160px] flex-1">
          <Search class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
          <Input
            v-model="store.searchQuery"
            type="search"
            :aria-label="t('field.searchPlaceholder')"
            :placeholder="t('field.searchPlaceholder')"
            class="h-[38px] pl-9"
          />
        </div>
      </div>

      <div class="flex items-center gap-3 px-1 text-[13px] text-muted-foreground">
        <Checkbox
          :model-value="allSelected"
          :aria-label="t('fieldUi.checkAll')"
          @update:model-value="toggleSelectAll"
        />
        <span aria-live="polite">{{ t("field.found", { count: store.filteredFields.length }) }}</span>
      </div>

      <!-- Rows -->
      <ul
        v-if="rows.length > 0"
        :aria-label="t('fieldUi.list')"
        class="flex flex-col pb-20"
        @keydown="onListKeydown"
      >
        <li
          v-for="row in rows"
          :key="row.id"
          class="relative flex h-14 items-center gap-1 pl-2"
        >
          <motion.span
            v-if="row.id === selectedId"
            layout-id="field-row-active"
            class="absolute inset-0 rounded-xl bg-primary-soft"
            :transition="{ type: 'spring', stiffness: 480, damping: 40 }"
          />
          <Checkbox
            class="relative"
            :model-value="store.selectedFieldIds.has(row.id)"
            :aria-label="t('fieldUi.check', { id: row.id })"
            @update:model-value="store.toggleSelection(row.id)"
          />
          <button
            :id="`field-row-${row.id}`"
            type="button"
            :aria-current="row.id === selectedId ? 'true' : undefined"
            class="relative flex h-[52px] min-w-0 flex-1 items-center gap-3 rounded-[10px] px-2.5 text-left transition-colors hover:bg-hover"
            @click="selectedId = row.id"
          >
            <span class="size-[30px] shrink-0 rounded-lg" :style="{ background: row.color }" aria-hidden="true" />
            <span class="flex min-w-0 flex-1 flex-col leading-snug">
              <span class="flex items-center gap-1.5" :class="row.id === selectedId ? 'font-semibold' : 'font-medium'">
                {{ t("farmland.fieldNumber", { id: row.id }) }}
                <span
                  v-if="row.pending"
                  class="size-[7px] rounded-[2px] bg-ready"
                  :title="t('fieldUi.modified')"
                />
                <span v-if="row.pending" class="sr-only">{{ t("fieldUi.modified") }}</span>
              </span>
              <span class="truncate text-[13px] text-muted-foreground">
                {{ row.crop }}<template v-if="row.stage"> · <span :class="row.ready ? 'text-ready' : 'text-primary-text'">{{ row.stage }}</span></template>
              </span>
            </span>
            <Badge
              v-if="row.issues"
              variant="danger"
              class="hidden max-w-[150px] truncate xl:inline-flex"
              :title="row.issues"
            >
              {{ row.issues }}
            </Badge>
            <Badge v-if="row.mine" variant="soft">{{ t("fieldUi.mine") }}</Badge>
            <span class="h-[5px] w-14 shrink-0 overflow-hidden rounded-full bg-border" aria-hidden="true">
              <span
                class="block h-full rounded-full transition-[width] duration-500 ease-spring"
                :class="row.ready ? 'bg-ready' : 'bg-primary'"
                :style="{ width: row.pct + '%' }"
              />
            </span>
          </button>
        </li>
      </ul>

      <!-- Empty state -->
      <div v-else-if="store.fields.length === 0" class="py-12 text-center">
        <Sprout class="mx-auto size-12 text-muted-foreground/50" />
        <p class="mt-4 text-muted-foreground">{{ t("field.empty") }}</p>
      </div>

      <!-- No matches -->
      <div v-else class="py-12 text-center">
        <p class="text-muted-foreground">{{ t("field.noMatch") }}</p>
      </div>

      <!-- Batch actions dock -->
      <AnimatePresence>
        <motion.div
          v-if="store.selectedFieldIds.size > 0"
          role="toolbar"
          :aria-label="t('field.selected', { count: store.selectedFieldIds.size })"
          class="glass-strong sticky bottom-4 z-10 mx-auto flex max-w-full flex-wrap items-center gap-1 rounded-2xl py-1.5 pl-4 pr-1.5"
          :initial="{ opacity: 0, y: 24, scale: 0.96 }"
          :animate="{ opacity: 1, y: 0, scale: 1 }"
          :exit="{ opacity: 0, y: 24, scale: 0.96 }"
          :transition="{ type: 'spring', stiffness: 420, damping: 34 }"
        >
          <span class="mr-1.5 whitespace-nowrap font-semibold">
            {{ t("field.selected", { count: store.selectedFieldIds.size }) }}
          </span>
          <Button variant="ghost" size="sm" class="h-9" @click="handleBatchMaxGrowth">
            <Sparkles class="size-4" />
            {{ t("field.maxGrowth") }}
          </Button>
          <Button variant="ghost" size="sm" class="h-9" @click="handleBatchMaxLime">
            <Droplets class="size-4" />
            {{ t("field.maxLime") }}
          </Button>
          <Button variant="ghost" size="sm" class="h-9" @click="handleBatchMaxFertilizer">
            <FlaskConical class="size-4" />
            {{ t("field.maxFertilizer") }}
          </Button>
          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <Button variant="ghost" size="sm" class="h-9">
                <Ellipsis class="size-4" />
                {{ t("fieldUi.more") }}
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" class="w-56">
              <DropdownMenuItem @select="handleBatchRemoveWeeds">
                <Bug class="size-4" />
                {{ t("field.removeWeeds") }}
              </DropdownMenuItem>
              <DropdownMenuItem @select="handleBatchRemoveStones">
                <Gem class="size-4" />
                {{ t("field.removeStones") }}
              </DropdownMenuItem>
              <template v-if="store.hasDensityData">
                <DropdownMenuItem @select="handleBatchMaxPlow">
                  <Tractor class="size-4" />
                  {{ t("field.maxPlow") }}
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem @select="handleBatchSoilWork('plow')">
                  <Tractor class="size-4" />
                  {{ t("field.soilPlow") }}
                </DropdownMenuItem>
                <DropdownMenuItem @select="handleBatchSoilWork('cultivate')">
                  <Shovel class="size-4" />
                  {{ t("field.soilCultivate") }}
                </DropdownMenuItem>
                <DropdownMenuItem @select="handleBatchSoilWork('roll')">
                  <Tractor class="size-4" />
                  {{ t("field.soilRoll") }}
                </DropdownMenuItem>
                <DropdownMenuItem @select="handleBatchSoilWork('mulch')">
                  <Shovel class="size-4" />
                  {{ t("field.soilMulch") }}
                </DropdownMenuItem>
              </template>
            </DropdownMenuContent>
          </DropdownMenu>
          <WithTooltip :label="t('common.deselectAll')" side="top">
            <Button
              variant="ghost"
              size="icon"
              :aria-label="t('common.deselectAll')"
              @click="store.deselectAll()"
            >
              <X class="size-4" />
            </Button>
          </WithTooltip>
        </motion.div>
      </AnimatePresence>
    </section>

    <!-- Inspector -->
    <aside
      :aria-label="t('fieldUi.details')"
      class="min-h-0 overflow-y-auto border-t bg-hover p-5 lg:border-l lg:border-t-0"
    >
      <AnimatePresence mode="wait" :initial="false">
        <motion.div
          :key="selectedField?.id ?? 'none'"
          :initial="{ opacity: 0, x: 12 }"
          :animate="{ opacity: 1, x: 0 }"
          :exit="{ opacity: 0, x: -12 }"
          :transition="{ duration: 0.2, ease: [0.32, 0.72, 0, 1] }"
        >
          <FieldEditor :field="selectedField" @prev="step(-1)" @next="step(1)" />
        </motion.div>
      </AnimatePresence>
    </aside>
  </div>
</template>
