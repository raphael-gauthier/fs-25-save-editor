<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { useSaleStore } from "@/stores/sale";
import { useSettingsStore } from "@/stores/settings";
import { useTauri } from "@/composables/useTauri";
import { formatMoney, formatOperatingTime } from "@/lib/utils";
import { useVehicleImages } from "@/composables/useVehicleImages";
import type { SaleItem, CatalogVehicle } from "@/lib/types";
import SaleItemEditor from "@/components/sales/SaleItemEditor.vue";
import VehicleImage from "@/components/vehicles/VehicleImage.vue";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Plus } from "lucide-vue-next";

const { t } = useI18n();
const router = useRouter();
const store = useSaleStore();
const settings = useSettingsStore();
const { invokeCommand } = useTauri();
const { loadBatch } = useVehicleImages();

// Catalog prices map (normalized xmlFilename → price)
const catalogPrices = ref<Map<string, number>>(new Map());

/** Normalize xmlFilename for reliable matching between savegame and catalog. */
function normalizePath(filename: string): string {
  let p = filename.replace(/\\/g, "/").toLowerCase();
  // Strip $moddir$ModName/ prefix → just the relative path inside the mod
  const moddirMatch = p.match(/^\$moddir\$[^/]+\/(.+)$/);
  if (moddirMatch) p = moddirMatch[1];
  // Strip absolute mod paths: keep everything after /mods/ModName/
  const modsMatch = p.match(/\/mods\/[^/]+\/(.+)$/);
  if (modsMatch) p = modsMatch[1];
  return p;
}

onMounted(async () => {
  if (!settings.gamePath) return;
  try {
    const catalog = await invokeCommand<CatalogVehicle[]>(
      "get_vehicle_catalog",
      { gamePath: settings.gamePath },
    );
    const map = new Map<string, number>();
    for (const v of catalog) {
      map.set(normalizePath(v.xmlFilename), v.price);
    }
    catalogPrices.value = map;
  } catch {
    // Catalog unavailable — columns will show "—"
  }
});

function getCatalogPrice(xmlFilename: string): number | null {
  return catalogPrices.value.get(normalizePath(xmlFilename)) ?? null;
}

function getDiscount(salePrice: number, catalogPrice: number | null): string {
  if (catalogPrice == null || catalogPrice <= 0) return "—";
  return `-${Math.round((1 - salePrice / catalogPrice) * 100)}%`;
}

watch(
  () => store.items,
  (items) => {
    if (settings.gamePath && items.length > 0) {
      loadBatch(
        settings.gamePath,
        items.map((s) => s.xmlFilename),
      );
    }
  },
  { immediate: true },
);

const selectedItem = ref<SaleItem | null>(null);
const editorOpen = ref(false);

const itemCount = computed(() => store.items.length);

function openEditor(item: SaleItem) {
  selectedItem.value = item;
  editorOpen.value = true;
}

function formatPercent(value: number): string {
  return `${Math.round(value * 100)}%`;
}
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-start justify-between">
      <div>
        <h2 class="text-2xl font-semibold">
          {{ t("sale.title") }}
          <span class="text-muted-foreground font-normal">({{ t("sale.count", { count: itemCount }) }})</span>
        </h2>
        <p class="text-sm text-muted-foreground">
          {{ t("sale.subtitle") }}
        </p>
      </div>
      <TooltipProvider v-if="!settings.gamePath">
        <Tooltip>
          <TooltipTrigger as-child>
            <Button variant="outline" disabled>
              <Plus class="size-4" />
              {{ t("sale.addVehicle") }}
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            <p>{{ t("sale.noGamePath") }}</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
      <Button v-else variant="outline" @click="router.push({ name: 'sale-add' })">
        <Plus class="size-4" />
        {{ t("sale.addVehicle") }}
      </Button>
    </div>

    <!-- Table -->
    <div class="rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead v-if="settings.gamePath" class="w-14" />
            <TableHead>{{ t("sale.name") }}</TableHead>
            <TableHead v-if="catalogPrices.size > 0" class="text-right">{{ t("sale.originalPrice") }}</TableHead>
            <TableHead class="text-right">{{ t("sale.price") }}</TableHead>
            <TableHead v-if="catalogPrices.size > 0" class="text-right">{{ t("sale.discount") }}</TableHead>
            <TableHead class="text-right">{{ t("sale.wear") }}</TableHead>
            <TableHead class="text-right">{{ t("sale.damage") }}</TableHead>
            <TableHead class="text-right">{{ t("sale.days") }}</TableHead>
            <TableHead class="text-right">{{ t("sale.hours") }}</TableHead>
            <TableHead>{{ t("sale.source") }}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-if="itemCount === 0">
            <TableCell :colspan="(settings.gamePath ? 8 : 7) + (catalogPrices.size > 0 ? 2 : 0)" class="py-8 text-center text-muted-foreground">
              {{ t("sale.empty") }}
            </TableCell>
          </TableRow>
          <TableRow
            v-for="item in store.items"
            :key="item.index"
            class="cursor-pointer"
            @click="openEditor(item)"
          >
            <TableCell v-if="settings.gamePath">
              <VehicleImage :filename="item.xmlFilename" size="sm" />
            </TableCell>
            <TableCell class="font-medium">
              {{ item.displayName }}
            </TableCell>
            <TableCell v-if="catalogPrices.size > 0" class="text-right font-mono text-muted-foreground">
              {{ getCatalogPrice(item.xmlFilename) != null ? `${formatMoney(getCatalogPrice(item.xmlFilename)!)} $` : '—' }}
            </TableCell>
            <TableCell class="text-right font-mono">
              {{ formatMoney(item.price) }} $
            </TableCell>
            <TableCell v-if="catalogPrices.size > 0" class="text-right font-mono">
              {{ getDiscount(item.price, getCatalogPrice(item.xmlFilename)) }}
            </TableCell>
            <TableCell class="text-right font-mono">
              {{ formatPercent(item.wear) }}
            </TableCell>
            <TableCell class="text-right font-mono">
              {{ formatPercent(item.damage) }}
            </TableCell>
            <TableCell class="text-right font-mono">
              {{ item.timeLeft }}j
            </TableCell>
            <TableCell class="text-right font-mono">
              {{ formatOperatingTime(item.operatingTime) }}
            </TableCell>
            <TableCell>
              <Badge v-if="item.index < 0" variant="default">{{ t("sale.new") }}</Badge>
              <Badge v-else-if="item.isGenerated" variant="secondary">{{ t("sale.generated") }}</Badge>
              <Badge v-else variant="outline">{{ t("sale.player") }}</Badge>
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

    <!-- Editor Sheet -->
    <SaleItemEditor
      :item="selectedItem"
      :catalog-price="selectedItem ? getCatalogPrice(selectedItem.xmlFilename) : null"
      :open="editorOpen"
      @update:open="editorOpen = $event"
    />
  </div>
</template>
