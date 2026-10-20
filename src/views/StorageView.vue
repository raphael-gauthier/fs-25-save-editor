<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { AnimatePresence, motion } from "motion-v";
import { useBuildingStore } from "@/stores/building";
import type { Placeable, SiloCapacity, SiloStorage } from "@/lib/types";
import { formatMoney } from "@/lib/utils";
import { groupObjects } from "@/lib/storageChanges";
import { fruitColor } from "@/lib/fruitColors";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Minus, Plus, Trash2, Wand2, Warehouse, Package } from "@lucide/vue";

const { t } = useI18n();
const store = useBuildingStore();

/** One entry of the list: a silo storage or a placeable's object storage. */
type Entry =
  | { key: string; type: "silo"; p: Placeable; storage: SiloStorage }
  | { key: string; type: "objects"; p: Placeable };

const entries = computed<Entry[]>(() => {
  const list: Entry[] = [];
  for (const p of store.placeables) {
    for (const storage of p.siloStorages) {
      list.push({ key: storageKey(p, storage), type: "silo", p, storage });
    }
  }
  for (const p of store.placeables) {
    if (
      p.storedObjects.length > 0 ||
      store.storageCapacities.objectStorages.some((o) => o.placeableIndex === p.index)
    ) {
      list.push({ key: `objects-${p.index}`, type: "objects", p });
    }
  }
  return list;
});

const focusKey = ref<string | null>(null);
watch(
  entries,
  (list) => {
    if (!list.some((e) => e.key === focusKey.value)) focusKey.value = list[0]?.key ?? null;
  },
  { immediate: true },
);
const focused = computed(() => entries.value.find((e) => e.key === focusKey.value));

function liters(amount: number): string {
  return `${formatMoney(amount)} L`;
}

function fillTypeLabel(fillType: string): string {
  return t(`fillTypes.${fillType}`, fillType);
}

function ownerLabel(farmId: number): string {
  if (farmId === 0) return t("field.free");
  if (farmId === 1) return t("field.myFarm");
  return t("field.farm", { id: farmId });
}

function capacityOf(p: Placeable, storage: SiloStorage): SiloCapacity | undefined {
  return store.storageCapacities.silos.find(
    (c) => c.placeableIndex === p.index && c.kind === storage.kind && c.index === storage.index,
  );
}

function used(storage: SiloStorage): number {
  return storage.stocks.reduce((sum, s) => sum + s.amount, 0);
}

/** Most a fill type may hold: its own capacity, or what the shared capacity leaves. */
function maxFor(storage: SiloStorage, capacity: SiloCapacity | undefined, fillType: string): number | undefined {
  if (!capacity) return undefined;
  const own = capacity.fillTypeCapacities.find((c) => c.fillType === fillType);
  if (own) return own.capacity;
  if (!capacity.shared) return capacity.capacity;
  const others = storage.stocks.filter((s) => s.fillType !== fillType).reduce((sum, s) => sum + s.amount, 0);
  return Math.max(0, capacity.capacity - others);
}

function setLevel(p: Placeable, storage: SiloStorage, fillType: string, value: number) {
  if (Number.isNaN(value)) return;
  const max = maxFor(storage, capacityOf(p, storage), fillType);
  const amount = Math.max(0, max === undefined ? value : Math.min(value, max));
  store.setSiloStock(p.index, storage.kind, storage.index, fillType, amount);
}

function storageKey(p: Placeable, storage: SiloStorage): string {
  return `${p.index}-${storage.kind}-${storage.index}`;
}

/** Fill type being added to each storage. */
const newFillType = ref<Record<string, string>>({});

function addableFillTypes(p: Placeable, storage: SiloStorage): string[] {
  const present = new Set(storage.stocks.filter((s) => s.amount > 0).map((s) => s.fillType));
  return (capacityOf(p, storage)?.fillTypes ?? []).filter((f) => !present.has(f));
}

function addFillType(p: Placeable, storage: SiloStorage) {
  const fillType = newFillType.value[storageKey(p, storage)];
  if (!fillType) return;
  // Fill what the storage has left for it
  const max = maxFor(storage, capacityOf(p, storage), fillType) ?? 0;
  store.setSiloStock(p.index, storage.kind, storage.index, fillType, max);
  newFillType.value = { ...newFillType.value, [storageKey(p, storage)]: "" };
}

function objectCapacity(p: Placeable): number | undefined {
  return store.storageCapacities.objectStorages.find((o) => o.placeableIndex === p.index)?.capacity;
}

function canAdd(p: Placeable, count: number): boolean {
  const capacity = objectCapacity(p);
  return capacity === undefined || p.storedObjects.length + count <= capacity;
}

function modelName(filename: string): string {
  return filename.split("/").pop()?.replace(/\.xml$/, "") ?? filename;
}

function hasFermenting(p: Placeable): boolean {
  return p.storedObjects.some((o) => o.isFermenting) && !store.fermentationToFinish.has(p.index);
}

/** Fill share of an entry in percent, when its capacity is known. */
function percent(entry: Entry): number | null {
  if (entry.type === "silo") {
    const capacity = capacityOf(entry.p, entry.storage)?.capacity;
    return capacity ? Math.min(100, (used(entry.storage) / capacity) * 100) : null;
  }
  const capacity = objectCapacity(entry.p);
  return capacity ? Math.min(100, (entry.p.storedObjects.length / capacity) * 100) : null;
}

/** Bar segments, one per fill type, as shares of the capacity. */
function segments(entry: Entry): { key: string; width: number; color: string }[] {
  if (entry.type === "objects") {
    return [{ key: "objects", width: percent(entry) ?? 0, color: "var(--primary)" }];
  }
  const capacity = capacityOf(entry.p, entry.storage)?.capacity;
  if (!capacity) return [];
  return entry.storage.stocks
    .filter((s) => s.amount > 0)
    .map((s) => ({ key: s.fillType, width: (s.amount / capacity) * 100, color: fruitColor(s.fillType) }));
}

function subtitle(entry: Entry): string {
  if (entry.type === "objects") {
    const capacity = objectCapacity(entry.p);
    const count = t("storage.objectCount", { count: entry.p.storedObjects.length });
    return capacity === undefined ? count : `${count} / ${capacity}`;
  }
  const capacity = capacityOf(entry.p, entry.storage)?.capacity;
  return capacity === undefined
    ? liters(used(entry.storage))
    : `${liters(used(entry.storage))} / ${liters(capacity)}`;
}

// Ring gauge for the focused silo
const RADIUS = 70;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;
const ringArcs = computed(() => {
  const entry = focused.value;
  if (!entry || entry.type !== "silo") return [];
  const capacity = capacityOf(entry.p, entry.storage)?.capacity;
  if (!capacity) return [];
  let start = 0;
  return entry.storage.stocks
    .filter((s) => s.amount > 0)
    .map((s) => {
      const len = Math.min(1, s.amount / capacity) * CIRCUMFERENCE;
      const arc = { key: s.fillType, color: fruitColor(s.fillType), dash: `${len} ${CIRCUMFERENCE - len}`, offset: -start };
      start += len;
      return arc;
    });
});

const spring = { type: "spring", stiffness: 480, damping: 40 } as const;
</script>

<template>
  <div class="grid h-full lg:grid-cols-[minmax(0,1fr)_minmax(360px,440px)]">
    <!-- List -->
    <section aria-labelledby="storage-title" class="flex min-h-0 min-w-0 flex-col gap-4 overflow-y-auto p-6">
      <div class="flex flex-col gap-1">
        <h2 id="storage-title" class="text-2xl font-semibold tracking-tight">{{ t("storage.title") }}</h2>
        <p class="text-[13px] text-muted-foreground">{{ t("storage.subtitle") }}</p>
      </div>

      <p v-if="entries.length === 0" class="text-sm text-muted-foreground">{{ t("storage.noSilos") }}</p>

      <ul :aria-label="t('storageUi.list')" class="flex flex-col">
        <li v-for="entry in entries" :key="entry.key" class="relative">
          <motion.span
            v-if="entry.key === focusKey"
            layout-id="storage-row-active"
            class="absolute inset-0 rounded-xl bg-primary-soft"
            :transition="spring"
          />
          <button
            type="button"
            :aria-current="entry.key === focusKey ? 'true' : undefined"
            class="relative flex min-h-[72px] w-full items-center gap-3 rounded-xl px-3 text-left transition-colors hover:bg-hover"
            @click="focusKey = entry.key"
          >
            <span class="inline-flex size-9 shrink-0 items-center justify-center rounded-[10px] bg-card" aria-hidden="true">
              <Package v-if="entry.type === 'objects'" class="size-[18px]" />
              <Warehouse v-else class="size-[18px]" />
            </span>
            <span class="flex min-w-0 flex-1 flex-col gap-1.5">
              <span class="flex items-center justify-between gap-2">
                <span class="flex min-w-0 items-center gap-2 truncate" :class="entry.key === focusKey ? 'font-semibold' : 'font-medium'">
                  {{ entry.p.displayName }}
                  <Badge v-if="entry.type === 'silo' && entry.storage.kind === 'extension'" variant="muted">
                    {{ t("storage.extension") }}
                  </Badge>
                </span>
                <span v-if="percent(entry) !== null" class="text-[13px] text-muted-foreground tabular">
                  {{ Math.round(percent(entry)!) }} %
                </span>
              </span>
              <span class="flex h-1.5 overflow-hidden rounded-full bg-border" aria-hidden="true">
                <span
                  v-for="seg in segments(entry)"
                  :key="seg.key"
                  class="h-full transition-[width] duration-500 ease-spring"
                  :style="{ width: seg.width + '%', background: seg.color }"
                />
              </span>
              <span class="text-xs text-muted-foreground tabular">{{ subtitle(entry) }}</span>
            </span>
          </button>
        </li>
      </ul>
      <p class="px-1 text-[13px] text-muted-foreground">{{ t("storageUi.bunkerNote") }}</p>
    </section>

    <!-- Inspector -->
    <aside
      :aria-label="t('storageUi.details')"
      class="min-h-0 overflow-y-auto border-t bg-hover p-5 lg:border-l lg:border-t-0"
    >
      <AnimatePresence mode="wait" :initial="false">
        <motion.div
          v-if="focused"
          :key="focused.key"
          class="flex flex-col gap-5"
          :initial="{ opacity: 0, x: 12 }"
          :animate="{ opacity: 1, x: 0 }"
          :exit="{ opacity: 0, x: -12 }"
          :transition="{ duration: 0.2, ease: [0.32, 0.72, 0, 1] }"
        >
          <div class="flex flex-col gap-0.5">
            <h3 class="text-[26px] font-semibold tracking-tight">{{ focused.p.displayName }}</h3>
            <span class="text-muted-foreground">
              <template v-if="focused.type === 'silo'">
                {{ focused.storage.kind === "extension" ? t("storage.extension") : t("storageUi.silo") }}
                · {{ ownerLabel(focused.storage.farmId) }}
              </template>
              <template v-else>{{ t("storage.objectStorages") }}</template>
            </span>
          </div>

          <!-- Silo -->
          <template v-if="focused.type === 'silo'">
            <div v-if="capacityOf(focused.p, focused.storage)" class="flex flex-wrap items-center gap-6">
              <div
                role="img"
                :aria-label="t('storageUi.gauge', { used: liters(used(focused.storage)), capacity: liters(capacityOf(focused.p, focused.storage)!.capacity) })"
                class="relative size-[168px] shrink-0"
              >
                <svg width="168" height="168" viewBox="0 0 168 168" class="-rotate-90" aria-hidden="true">
                  <circle cx="84" cy="84" :r="RADIUS" fill="none" stroke="var(--border)" stroke-width="14" />
                  <circle
                    v-for="arc in ringArcs"
                    :key="arc.key"
                    cx="84"
                    cy="84"
                    :r="RADIUS"
                    fill="none"
                    :stroke="arc.color"
                    stroke-width="14"
                    :stroke-dasharray="arc.dash"
                    :stroke-dashoffset="arc.offset"
                    class="transition-[stroke-dasharray,stroke-dashoffset] duration-700 ease-spring"
                  />
                </svg>
                <span class="absolute inset-0 flex flex-col items-center justify-center leading-tight">
                  <span class="text-[28px] font-semibold tracking-tight tabular">{{ Math.round(percent(focused) ?? 0) }} %</span>
                  <span class="text-xs text-muted-foreground">{{ t("storageUi.filled") }}</span>
                </span>
              </div>
              <dl class="flex min-w-[150px] flex-1 flex-col gap-2.5">
                <div>
                  <dt class="text-xs text-muted-foreground">{{ t("storageUi.content") }}</dt>
                  <dd class="text-xl font-semibold tabular">{{ liters(used(focused.storage)) }}</dd>
                </div>
                <div>
                  <dt class="text-xs text-muted-foreground">{{ t("storageUi.capacity") }}</dt>
                  <dd class="text-xl font-semibold tabular">{{ liters(capacityOf(focused.p, focused.storage)!.capacity) }}</dd>
                </div>
                <div>
                  <dt class="text-xs text-muted-foreground">{{ t("storageUi.free") }}</dt>
                  <dd class="text-xl font-semibold tabular">
                    {{ liters(Math.max(0, capacityOf(focused.p, focused.storage)!.capacity - used(focused.storage))) }}
                  </dd>
                </div>
              </dl>
            </div>

            <div class="flex flex-col gap-1.5">
              <h4 class="text-[13px] font-semibold text-muted-foreground">{{ t("storageUi.products") }}</h4>
              <div
                v-for="stock in focused.storage.stocks.filter((s) => s.amount > 0)"
                :key="stock.fillType"
                class="flex flex-wrap items-center gap-2 rounded-xl bg-card py-2 pl-3.5 pr-2"
              >
                <span class="size-2.5 shrink-0 rounded-[3px]" :style="{ background: fruitColor(stock.fillType) }" aria-hidden="true" />
                <label :for="`stock-${focused.key}-${stock.fillType}`" class="min-w-[90px] flex-1 font-medium">
                  {{ fillTypeLabel(stock.fillType) }}
                </label>
                <span class="flex items-center gap-1.5">
                  <Input
                    :id="`stock-${focused.key}-${stock.fillType}`"
                    type="number"
                    min="0"
                    step="1000"
                    class="h-9 w-28 text-right tabular"
                    :model-value="Math.round(stock.amount)"
                    @change="setLevel(focused.p, focused.storage, stock.fillType, Number(($event.target as HTMLInputElement).value))"
                  />
                  <span class="text-muted-foreground">L</span>
                </span>
                <Button
                  v-if="maxFor(focused.storage, capacityOf(focused.p, focused.storage), stock.fillType) !== undefined"
                  size="sm"
                  variant="outline"
                  class="h-9"
                  :aria-label="t('storageUi.fillProduct', { product: fillTypeLabel(stock.fillType) })"
                  @click="setLevel(focused.p, focused.storage, stock.fillType, maxFor(focused.storage, capacityOf(focused.p, focused.storage), stock.fillType)!)"
                >
                  {{ t("storage.fill") }}
                </Button>
                <Button
                  size="icon"
                  variant="ghost"
                  class="size-9"
                  :aria-label="t('storageUi.emptyProduct', { product: fillTypeLabel(stock.fillType) })"
                  :title="t('storage.empty')"
                  @click="setLevel(focused.p, focused.storage, stock.fillType, 0)"
                >
                  <Trash2 class="size-4" />
                </Button>
              </div>
              <p v-if="focused.storage.stocks.every((s) => s.amount <= 0)" class="text-sm text-muted-foreground">
                {{ t("storage.emptySilo") }}
              </p>
              <div v-if="addableFillTypes(focused.p, focused.storage).length > 0" class="flex items-center gap-2 pt-1.5">
                <Select
                  :model-value="newFillType[focused.key] ?? ''"
                  @update:model-value="newFillType = { ...newFillType, [focused.key]: String($event) }"
                >
                  <SelectTrigger class="h-9 flex-1" :aria-label="t('storage.addFillType')">
                    <SelectValue :placeholder="t('storage.addFillType')" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem v-for="fillType in addableFillTypes(focused.p, focused.storage)" :key="fillType" :value="fillType">
                      {{ fillTypeLabel(fillType) }}
                    </SelectItem>
                  </SelectContent>
                </Select>
                <Button size="sm" variant="outline" class="h-9" :disabled="!newFillType[focused.key]" @click="addFillType(focused.p, focused.storage)">
                  <Plus class="size-4" />
                  {{ t("storage.add") }}
                </Button>
              </div>
            </div>
          </template>

          <!-- Object storage -->
          <template v-else>
            <div class="flex flex-col gap-2">
              <div class="flex items-baseline justify-between">
                <span class="text-[13px] font-semibold text-muted-foreground">{{ t("storageUi.stored") }}</span>
                <span aria-live="polite" class="text-xl font-semibold tabular">
                  {{ focused.p.storedObjects.length }}<template v-if="objectCapacity(focused.p) !== undefined"> / {{ objectCapacity(focused.p) }}</template>
                </span>
              </div>
              <div v-if="objectCapacity(focused.p)" class="h-2 overflow-hidden rounded-full bg-border" aria-hidden="true">
                <div
                  class="h-full rounded-full transition-[width] duration-500 ease-spring"
                  :class="canAdd(focused.p, 1) ? 'bg-primary' : 'bg-destructive'"
                  :style="{ width: (percent(focused) ?? 0) + '%' }"
                />
              </div>
              <Button v-if="hasFermenting(focused.p)" size="sm" variant="outline" class="h-9 self-start" @click="store.finishFermentation(focused.p.index)">
                <Wand2 class="size-4" />
                {{ t("storage.finishFermentation") }}
              </Button>
            </div>

            <ul v-if="focused.p.storedObjects.length > 0" class="flex flex-col gap-1.5">
              <li
                v-for="group in groupObjects(focused.p.storedObjects)"
                :key="group.key"
                class="flex min-h-12 items-center gap-2.5 rounded-xl bg-card py-1.5 pl-3.5 pr-1.5"
              >
                <span class="size-2.5 shrink-0 rounded-[3px]" :style="{ background: fruitColor(group.object.fillType) }" aria-hidden="true" />
                <span class="flex min-w-0 flex-1 flex-col leading-tight">
                  <span class="truncate font-semibold">
                    {{ fillTypeLabel(group.object.fillType) }}
                    <span class="font-normal text-muted-foreground">
                      · {{ group.object.className === "Bale" ? t("storage.bale") : t("storage.pallet") }}
                    </span>
                  </span>
                  <span class="truncate text-xs text-muted-foreground">
                    {{ modelName(group.object.filename) }} · {{ liters(group.object.fillLevel) }}
                    <template v-if="group.object.wrappingState > 0"> · {{ t("storage.wrapped") }}</template>
                    <template v-if="group.object.isFermenting && !store.fermentationToFinish.has(focused.p.index)"> · {{ t("storage.fermenting") }}</template>
                  </span>
                </span>
                <div role="group" :aria-label="fillTypeLabel(group.object.fillType)" class="flex items-center gap-1">
                  <Button
                    size="icon"
                    variant="outline"
                    class="size-8"
                    :aria-label="t('storage.remove')"
                    @click="store.changeObjectCount(focused.p.index, group.key, -1)"
                  >
                    <Minus class="size-4" />
                  </Button>
                  <span class="w-9 text-center font-semibold tabular">{{ group.indices.length }}</span>
                  <Button
                    size="icon"
                    variant="outline"
                    class="size-8"
                    :aria-label="t('storage.add')"
                    :disabled="!canAdd(focused.p, 1)"
                    @click="store.changeObjectCount(focused.p.index, group.key, 1)"
                  >
                    <Plus class="size-4" />
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    class="h-8 px-2"
                    :disabled="!canAdd(focused.p, 10)"
                    @click="store.changeObjectCount(focused.p.index, group.key, 10)"
                  >
                    +10
                  </Button>
                </div>
              </li>
            </ul>
            <p v-else class="text-sm text-muted-foreground">{{ t("storage.emptyObjectStorage") }}</p>
          </template>
        </motion.div>
      </AnimatePresence>
    </aside>
  </div>
</template>
