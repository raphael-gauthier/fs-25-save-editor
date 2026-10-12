<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useFieldStore } from "@/stores/field";
import { useFinanceStore } from "@/stores/finance";
import { useFarmlandTrade, PLAYER_FARM_ID, type TradeBlock } from "@/composables/useFarmlandTrade";
import type { FarmlandInfo } from "@/lib/types";
import { formatMoney } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
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
import { Info, Landmark, HandCoins } from "@lucide/vue";

type OwnerFilter = "all" | "free" | "mine" | "others";

const { t } = useI18n();
const store = useFieldStore();
const finance = useFinanceStore();
const trade = useFarmlandTrade();

const search = ref("");
const ownerFilter = ref<OwnerFilter>("all");
const selected = ref(new Set<number>());

const ownerFilters: OwnerFilter[] = ["all", "free", "mine", "others"];

/** Farmlands the in-game farmland screen lists, with the search and owner filters. */
const rows = computed<FarmlandInfo[]>(() => {
  const query = search.value.trim();
  return store.farmlandInfo.filter((info) => {
    if (!info.showOnFarmlandsScreen) return false;
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
  if (!store.getFieldById(id)) return "—";
  const crop = store.getFieldDensity(id)?.dominantFruit;
  const name = t("farmland.fieldNumber", { id });
  return crop ? `${name} · ${t(`fillTypes.${crop}`, crop)}` : name;
}

function blockLabel(block: TradeBlock | null): string {
  return block ? t(`farmland.block.${block}`) : "";
}

function handleBuy(ids: number[]) {
  if (trade.buy(ids)) selected.value = new Set();
}

function handleSell(ids: number[]) {
  if (trade.sell(ids)) selected.value = new Set();
}

const selectedBuyBlock = computed(() => trade.buyBlock(selectedIds.value));
const selectedSellBlock = computed(() => trade.sellBlock(selectedIds.value));
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h2 class="text-2xl font-semibold">{{ t("farmland.title") }}</h2>
        <p class="text-sm text-muted-foreground">{{ t("farmland.subtitle") }}</p>
      </div>
      <p class="font-mono text-sm">{{ t("farmland.money", { amount: formatMoney(finance.money) }) }}</p>
    </div>

    <Alert v-if="store.farmlandInfo.length === 0">
      <Info class="size-4" />
      <AlertDescription>{{ t("farmland.noInfo") }}</AlertDescription>
    </Alert>

    <template v-else>
      <p class="text-xs text-muted-foreground">{{ t("farmland.priceNote") }}</p>

      <!-- Filters -->
      <div class="flex flex-wrap items-center gap-3">
        <Input v-model="search" :placeholder="t('farmland.searchPlaceholder')" class="max-w-xs" />
        <Select :model-value="ownerFilter" @update:model-value="ownerFilter = $event as OwnerFilter">
          <SelectTrigger class="w-48" :aria-label="t('farmland.owner')">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="filter in ownerFilters" :key="filter" :value="filter">
              {{ t(filter === "all" ? "farmland.allOwners" : `farmland.${filter}`) }}
            </SelectItem>
          </SelectContent>
        </Select>
        <span class="text-sm text-muted-foreground">{{ t("farmland.found", { count: rows.length }) }}</span>
      </div>

      <!-- Batch actions -->
      <div v-if="selectedIds.length > 0" class="flex flex-wrap items-center gap-2">
        <span class="text-sm font-medium">{{ t("farmland.selected", { count: selectedIds.length }) }}</span>
        <span :title="blockLabel(selectedBuyBlock)">
          <Button variant="outline" size="sm" :disabled="!!selectedBuyBlock" @click="handleBuy(selectedIds)">
            <Landmark class="size-4" />
            {{ t("farmland.buySelected", { amount: formatMoney(trade.totalPrice(selectedIds)) }) }}
          </Button>
        </span>
        <span :title="blockLabel(selectedSellBlock)">
          <Button variant="outline" size="sm" :disabled="!!selectedSellBlock" @click="handleSell(selectedIds)">
            <HandCoins class="size-4" />
            {{ t("farmland.sellSelected", { amount: formatMoney(trade.totalPrice(selectedIds)) }) }}
          </Button>
        </span>
        <Button variant="ghost" size="sm" @click="selected = new Set()">
          {{ t("common.deselectAll") }}
        </Button>
      </div>

      <!-- Table -->
      <div class="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead class="w-10">
                <Checkbox :model-value="allSelected" :aria-label="t('farmland.selected', { count: rows.length })" @update:model-value="toggleAll" />
              </TableHead>
              <TableHead class="w-20">{{ t("farmland.id") }}</TableHead>
              <TableHead>{{ t("farmland.field") }}</TableHead>
              <TableHead class="text-right">{{ t("farmland.area") }}</TableHead>
              <TableHead class="text-right">{{ t("farmland.price") }}</TableHead>
              <TableHead>{{ t("farmland.owner") }}</TableHead>
              <TableHead class="text-right">{{ t("farmland.buildings") }}</TableHead>
              <TableHead class="w-28" />
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="info in rows" :key="info.id">
              <TableCell>
                <Checkbox
                  :model-value="selected.has(info.id)"
                  :aria-label="String(info.id)"
                  @update:model-value="toggle(info.id)"
                />
              </TableCell>
              <TableCell class="font-mono">{{ info.id }}</TableCell>
              <TableCell>{{ fieldLabel(info.id) }}</TableCell>
              <TableCell class="text-right font-mono">{{ info.areaHa.toFixed(2) }} ha</TableCell>
              <TableCell class="text-right font-mono">{{ formatMoney(info.price) }}</TableCell>
              <TableCell :class="ownerChanged(info.id) ? 'font-medium text-amber-600 dark:text-amber-400' : ''">
                {{ ownerLabel(info.id) }}
              </TableCell>
              <TableCell class="text-right font-mono">{{ info.builtPlaceables || "—" }}</TableCell>
              <TableCell class="text-right">
                <span v-if="trade.ownerOf(info.id) === 0" :title="blockLabel(trade.buyBlock([info.id]))">
                  <Button size="sm" variant="outline" :disabled="!!trade.buyBlock([info.id])" @click="handleBuy([info.id])">
                    {{ t("farmland.buy") }}
                  </Button>
                </span>
                <span v-else-if="trade.ownerOf(info.id) === PLAYER_FARM_ID" :title="blockLabel(trade.sellBlock([info.id]))">
                  <Button size="sm" variant="outline" :disabled="!!trade.sellBlock([info.id])" @click="handleSell([info.id])">
                    {{ t("farmland.sell") }}
                  </Button>
                </span>
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>
    </template>
  </div>
</template>
