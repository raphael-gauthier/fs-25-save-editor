<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { AnimatePresence, motion } from "motion-v";
import { useFieldStore } from "@/stores/field";
import { useFinanceStore } from "@/stores/finance";
import { useFarmlandTrade, PLAYER_FARM_ID, type TradeBlock } from "@/composables/useFarmlandTrade";
import type { FarmlandInfo } from "@/lib/types";
import { formatMoney } from "@/lib/utils";
import { fruitColor } from "@/lib/fruitColors";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Info, Lock, Search, X } from "@lucide/vue";
import WithTooltip from "@/components/WithTooltip.vue";

type OwnerFilter = "all" | "free" | "mine" | "others";

const { t, locale } = useI18n();
const store = useFieldStore();
const finance = useFinanceStore();
const trade = useFarmlandTrade();

const search = ref("");
const ownerFilter = ref<OwnerFilter>("all");
const selected = ref(new Set<number>());
const focusId = ref<number | null>(null);

const ownerFilters: { value: OwnerFilter; labelKey: string }[] = [
  { value: "all", labelKey: "fieldUi.all" },
  { value: "free", labelKey: "farmland.free" },
  { value: "mine", labelKey: "fieldUi.mine" },
  { value: "others", labelKey: "farmland.others" },
];

/** Farmlands the in-game farmland screen lists. */
const listed = computed(() => store.farmlandInfo.filter((info) => info.showOnFarmlandsScreen));

/** The listed farmlands passing the search and owner filters. */
const rows = computed<FarmlandInfo[]>(() => {
  const query = search.value.trim();
  return listed.value.filter((info) => {
    if (query && !String(info.id).includes(query)) return false;
    const owner = trade.ownerOf(info.id) ?? 0;
    switch (ownerFilter.value) {
      case "free":
        return owner === 0;
      case "mine":
        return owner === PLAYER_FARM_ID;
      case "others":
        return owner !== 0 && owner !== PLAYER_FARM_ID;
      default:
        return true;
    }
  });
});

watch(
  rows,
  (list) => {
    if (!list.some((r) => r.id === focusId.value)) focusId.value = list[0]?.id ?? null;
  },
  { immediate: true },
);

const focused = computed(() => (focusId.value === null ? undefined : store.getFarmlandInfo(focusId.value)));

const selectedIds = computed(() => rows.value.map((r) => r.id).filter((id) => selected.value.has(id)));
const allSelected = computed(() => rows.value.length > 0 && selectedIds.value.length === rows.value.length);

function toggle(id: number) {
  const next = new Set(selected.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  selected.value = next;
}

function toggleAll() {
  selected.value = allSelected.value ? new Set() : new Set(rows.value.map((r) => r.id));
}

function ownerLabel(id: number): string {
  const owner = trade.ownerOf(id) ?? 0;
  if (owner === 0) return t("field.free");
  if (owner === PLAYER_FARM_ID) return t("field.myFarm");
  return t("field.farm", { id: owner });
}

function ownerChanged(id: number): boolean {
  return trade.ownerOf(id) !== store.getOriginalFarmlandOwner(id);
}

/** The field on a farmland (a field's id is its farmland's id) and its main crop. */
function fieldLabel(id: number): string {
  if (!store.getFieldById(id)) return t("farmlandUi.noField");
  const crop = store.getFieldDensity(id)?.dominantFruit;
  const name = t("farmland.fieldNumber", { id });
  return crop ? `${name} · ${t(`fillTypes.${crop}`, crop)}` : name;
}

function cropOf(id: number): string | null {
  return store.getFieldDensity(id)?.dominantFruit ?? null;
}

function blockLabel(block: TradeBlock | null): string {
  return block ? t(`farmland.block.${block}`) : "";
}

// The operation: checked free farmlands are bought, checked own ones sold
const toBuy = computed(() => selectedIds.value.filter((id) => trade.ownerOf(id) === 0));
const toSell = computed(() => selectedIds.value.filter((id) => trade.ownerOf(id) === PLAYER_FARM_ID));
const ignored = computed(() => selectedIds.value.length - toBuy.value.length - toSell.value.length);
const buyTotal = computed(() => trade.totalPrice(toBuy.value));
const sellTotal = computed(() => trade.totalPrice(toSell.value));
const moneyAfter = computed(() => finance.money - buyTotal.value + sellTotal.value);

/** Why the operation cannot go through, selling first so its money counts. */
const operationBlock = computed<string | null>(() => {
  if (toBuy.value.length === 0 && toSell.value.length === 0) return null;
  const sellBlock = toSell.value.length ? trade.sellBlock(toSell.value) : null;
  if (sellBlock) return blockLabel(sellBlock);
  for (const id of toBuy.value) {
    const block = trade.buyBlock([id]);
    if (block && block !== "money") return blockLabel(block);
  }
  return moneyAfter.value < 0 ? t("farmlandUi.missing", { amount: formatMoney(-moneyAfter.value) }) : null;
});

function confirmOperation() {
  if (operationBlock.value) return;
  if (toSell.value.length) trade.sell(toSell.value);
  if (toBuy.value.length) trade.buy(toBuy.value);
  selected.value = new Set();
}

function quickTrade(id: number) {
  if (trade.ownerOf(id) === 0) trade.buy([id]);
  else if (trade.ownerOf(id) === PLAYER_FARM_ID) trade.sell([id]);
}

/** The player's land once the operation is done, for the proportional bar. */
const landAfter = computed(() =>
  listed.value
    .filter((info) => {
      const owner = trade.ownerOf(info.id);
      if (owner === PLAYER_FARM_ID) return !toSell.value.includes(info.id);
      return toBuy.value.includes(info.id);
    })
    .map((info) => ({ id: info.id, ha: info.areaHa, bought: toBuy.value.includes(info.id) })),
);
const landAfterHa = computed(() => landAfter.value.reduce((sum, l) => sum + l.ha, 0));
const landBeforeHa = computed(() =>
  listed.value
    .filter((info) => trade.ownerOf(info.id) === PLAYER_FARM_ID)
    .reduce((sum, info) => sum + info.areaHa, 0),
);
const landDelta = computed(() => landAfterHa.value - landBeforeHa.value);

/** Hectares the way the user's language writes them (23,08 in French). */
function ha(value: number): string {
  return new Intl.NumberFormat(locale.value, { minimumFractionDigits: 2, maximumFractionDigits: 2 }).format(value);
}

const spring = { type: "spring", stiffness: 480, damping: 40 } as const;
</script>

<template>
  <div class="grid h-full lg:grid-cols-[minmax(0,1fr)_minmax(340px,400px)]">
    <!-- List -->
    <section
      aria-labelledby="farmlands-title"
      class="relative flex min-h-0 min-w-0 flex-col gap-4 overflow-y-auto p-6"
    >
      <div class="flex flex-col gap-1">
        <h2 id="farmlands-title" class="text-2xl font-semibold tracking-tight">{{ t("farmland.title") }}</h2>
        <p class="text-[13px] text-muted-foreground">{{ t("farmland.priceNote") }}</p>
      </div>

      <Alert v-if="store.farmlandInfo.length === 0">
        <Info class="size-4" />
        <AlertDescription>{{ t("farmland.noInfo") }}</AlertDescription>
      </Alert>

      <template v-else>
        <div class="flex flex-wrap items-center gap-2">
          <ToggleGroup
            type="single"
            :spacing="1"
            :aria-label="t('farmland.owner')"
            :model-value="ownerFilter"
            class="rounded-[11px] bg-hover p-[3px]"
            @update:model-value="(v) => v && (ownerFilter = v as OwnerFilter)"
          >
            <ToggleGroupItem
              v-for="filter in ownerFilters"
              :key="filter.value"
              :value="filter.value"
              class="relative h-8 rounded-lg px-3 text-[13px] text-muted-foreground hover:bg-transparent hover:text-foreground data-[state=on]:bg-transparent data-[state=on]:font-semibold data-[state=on]:text-foreground"
            >
              <motion.span
                v-if="ownerFilter === filter.value"
                layout-id="farmland-owner-segment"
                class="absolute inset-0 rounded-lg bg-background shadow-sm"
                :transition="spring"
              />
              <span class="relative">{{ t(filter.labelKey) }}</span>
            </ToggleGroupItem>
          </ToggleGroup>
          <div class="relative min-w-[160px] flex-1">
            <Search class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
            <Input
              v-model="search"
              type="search"
              :aria-label="t('farmland.searchPlaceholder')"
              :placeholder="t('farmland.searchPlaceholder')"
              class="h-[38px] pl-9"
            />
          </div>
        </div>

        <div class="flex items-center gap-3 px-1 text-[13px] text-muted-foreground">
          <Checkbox :model-value="allSelected" :aria-label="t('farmlandUi.checkAll')" @update:model-value="toggleAll" />
          <span aria-live="polite">{{ t("farmland.found", { count: rows.length }) }}</span>
        </div>

        <ul :aria-label="t('farmlandUi.list')" class="flex flex-col">
          <li v-for="info in rows" :key="info.id" class="relative flex h-[60px] items-center gap-1 pl-2">
            <motion.span
              v-if="info.id === focusId"
              layout-id="farmland-row-active"
              class="absolute inset-0 rounded-xl bg-primary-soft"
              :transition="spring"
            />
            <Checkbox
              class="relative"
              :model-value="selected.has(info.id)"
              :aria-label="t('farmlandUi.check', { id: info.id })"
              @update:model-value="toggle(info.id)"
            />
            <button
              type="button"
              :aria-current="info.id === focusId ? 'true' : undefined"
              class="relative flex h-14 min-w-0 flex-1 items-center gap-3 rounded-[10px] px-2.5 text-left transition-colors hover:bg-hover"
              @click="focusId = info.id"
            >
              <span
                class="inline-flex size-[34px] shrink-0 items-center justify-center rounded-[9px] text-[13px] font-bold tabular"
                :class="trade.ownerOf(info.id) === PLAYER_FARM_ID ? 'bg-primary-soft text-primary-text' : 'bg-card'"
              >
                {{ info.id }}
              </span>
              <span class="flex min-w-0 flex-1 flex-col leading-snug">
                <span class="flex items-center gap-2 truncate" :class="info.id === focusId ? 'font-semibold' : 'font-medium'">
                  <span
                    v-if="cropOf(info.id)"
                    class="size-2 shrink-0 rounded-[3px]"
                    :style="{ background: fruitColor(cropOf(info.id)) }"
                    aria-hidden="true"
                  />
                  {{ fieldLabel(info.id) }}
                </span>
                <span class="truncate text-[13px] text-muted-foreground">
                  {{ ha(info.areaHa) }} ha ·
                  <span :class="ownerChanged(info.id) ? 'font-semibold text-ready' : ''">{{ ownerLabel(info.id) }}</span>
                </span>
              </span>
              <Lock
                v-if="info.builtPlaceables > 0 && trade.ownerOf(info.id) === PLAYER_FARM_ID"
                class="size-3.5 shrink-0 text-muted-foreground"
                :aria-label="t('farmland.block.buildings')"
              />
              <span class="font-semibold tabular">{{ formatMoney(info.price) }}</span>
            </button>
          </li>
        </ul>
      </template>
    </section>

    <!-- Inspector: the focused parcel and the operation -->
    <aside
      :aria-label="t('farmlandUi.details')"
      class="flex min-h-0 flex-col gap-5 overflow-y-auto border-t bg-hover p-5 lg:border-l lg:border-t-0"
    >
      <AnimatePresence mode="wait" :initial="false">
        <motion.div
          v-if="focused"
          :key="focused.id"
          class="flex flex-col gap-4"
          :initial="{ opacity: 0, x: 12 }"
          :animate="{ opacity: 1, x: 0 }"
          :exit="{ opacity: 0, x: -12 }"
          :transition="{ duration: 0.2, ease: [0.32, 0.72, 0, 1] }"
        >
          <div class="flex flex-col gap-1">
            <Badge :variant="trade.ownerOf(focused.id) === PLAYER_FARM_ID ? 'soft' : 'muted'" class="self-start">
              {{ ownerLabel(focused.id) }}
            </Badge>
            <h3 class="text-[28px] font-semibold tracking-tight">{{ t("farmland.id") }} {{ focused.id }}</h3>
            <span class="text-muted-foreground">{{ fieldLabel(focused.id) }}</span>
          </div>
          <dl class="grid grid-cols-2 gap-2">
            <div class="rounded-xl bg-card px-3.5 py-3">
              <dt class="text-xs text-muted-foreground">{{ t("farmland.area") }}</dt>
              <dd class="text-xl font-semibold tabular">{{ ha(focused.areaHa) }} ha</dd>
            </div>
            <div class="rounded-xl bg-card px-3.5 py-3">
              <dt class="text-xs text-muted-foreground">{{ t("farmland.price") }}</dt>
              <dd class="text-xl font-semibold tabular">{{ formatMoney(focused.price) }}</dd>
            </div>
            <div v-if="focused.builtPlaceables > 0" class="col-span-2 rounded-xl bg-card px-3.5 py-3">
              <dt class="text-xs text-muted-foreground">{{ t("farmland.buildings") }}</dt>
              <dd class="font-semibold tabular">{{ focused.builtPlaceables }}</dd>
            </div>
          </dl>
          <div v-if="trade.ownerOf(focused.id) === 0 || trade.ownerOf(focused.id) === PLAYER_FARM_ID" class="flex flex-col gap-1.5">
            <Button
              :variant="trade.ownerOf(focused.id) === 0 ? 'default' : 'outline'"
              class="h-10"
              :disabled="!!(trade.ownerOf(focused.id) === 0 ? trade.buyBlock([focused.id]) : trade.sellBlock([focused.id]))"
              @click="quickTrade(focused.id)"
            >
              {{ trade.ownerOf(focused.id) === 0 ? t("farmland.buy") : t("farmland.sell") }}
              · {{ formatMoney(focused.price) }}
            </Button>
            <p class="text-xs text-muted-foreground">
              {{ blockLabel(trade.ownerOf(focused.id) === 0 ? trade.buyBlock([focused.id]) : trade.sellBlock([focused.id])) }}
            </p>
          </div>
        </motion.div>
      </AnimatePresence>

      <span class="flex-1" />

      <!-- The operation on the checked parcels -->
      <section aria-labelledby="operation-title" class="glass-strong flex flex-col gap-3 rounded-2xl p-4">
        <h3 id="operation-title" class="font-semibold">{{ t("farmlandUi.operation") }}</h3>

        <div class="flex flex-col gap-2">
          <span class="text-xs text-muted-foreground">{{ t("farmlandUi.landAfter") }}</span>
          <div class="flex flex-wrap items-baseline gap-x-2 gap-y-1">
            <span class="whitespace-nowrap text-lg font-semibold tracking-tight tabular">{{ ha(landAfterHa) }} ha</span>
            <span class="whitespace-nowrap text-[13px] text-muted-foreground">
              {{ t("farmlandUi.parcels", { count: landAfter.length }, landAfter.length) }}
            </span>
            <Badge
              v-if="Math.abs(landDelta) >= 0.005"
              :variant="landDelta > 0 ? 'soft' : 'warning'"
              class="ml-auto tabular"
            >
              {{ landDelta > 0 ? "+" : "−" }}{{ ha(Math.abs(landDelta)) }} ha
            </Badge>
          </div>
          <div
            role="img"
            :aria-label="t('farmlandUi.landAfterAria', { ha: ha(landAfterHa), count: landAfter.length })"
            class="flex h-2.5 gap-[2px] overflow-hidden rounded-full"
          >
            <motion.span
              v-for="land in landAfter"
              :key="land.id"
              layout
              class="h-full min-w-[3px]"
              :class="land.bought ? 'bg-primary' : 'bg-primary/45'"
              :style="{ flex: `${Math.round(land.ha * 100)} 1 0` }"
              :title="`${t('farmland.id')} ${land.id} · ${ha(land.ha)} ha`"
              :transition="spring"
            />
          </div>
        </div>

        <p v-if="selectedIds.length === 0" class="text-[13px] text-muted-foreground">{{ t("farmlandUi.hint") }}</p>
        <ul v-else class="flex flex-col gap-1.5">
          <li v-for="id in toSell" :key="'s' + id" class="flex items-center gap-2 text-sm">
            <span class="flex-1">{{ t("farmlandUi.sellLine", { id }) }}</span>
            <span class="font-semibold text-primary-text tabular">+{{ formatMoney(trade.totalPrice([id])) }}</span>
          </li>
          <li v-for="id in toBuy" :key="'b' + id" class="flex items-center gap-2 text-sm">
            <span class="flex-1">{{ t("farmlandUi.buyLine", { id }) }}</span>
            <span class="font-semibold tabular">−{{ formatMoney(trade.totalPrice([id])) }}</span>
          </li>
          <li v-if="ignored > 0" class="text-xs text-muted-foreground">
            {{ t("farmlandUi.ignored", { count: ignored }, ignored) }}
          </li>
        </ul>

        <div class="flex items-baseline gap-2 border-t pt-2.5">
          <span class="flex-1 text-muted-foreground">{{ t("farmlandUi.moneyAfter") }}</span>
          <span
            aria-live="polite"
            class="text-[22px] font-semibold tracking-tight tabular"
            :class="moneyAfter < 0 ? 'text-destructive' : ''"
          >
            {{ formatMoney(moneyAfter) }}
          </span>
        </div>
        <p
          v-if="operationBlock"
          role="alert"
          class="rounded-lg bg-destructive-soft px-3 py-2 text-sm font-semibold text-destructive"
        >
          {{ operationBlock }}
        </p>
        <div v-if="selectedIds.length > 0" class="flex gap-2">
          <Button
            class="h-10 flex-1"
            :disabled="(toBuy.length === 0 && toSell.length === 0) || !!operationBlock"
            @click="confirmOperation"
          >
            {{ t("farmlandUi.confirm") }}
          </Button>
          <WithTooltip :label="t('common.deselectAll')" side="top">
            <Button
              variant="ghost"
              size="icon"
              class="size-10"
              :aria-label="t('common.deselectAll')"
              @click="selected = new Set()"
            >
              <X class="size-4" />
            </Button>
          </WithTooltip>
        </div>
        <p v-if="selectedIds.length > 0" class="text-xs text-muted-foreground">{{ t("farmlandUi.note") }}</p>
      </section>
    </aside>
  </div>
</template>
