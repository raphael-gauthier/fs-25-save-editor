<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useSaleStore } from "@/stores/sale";
import { useSettingsStore } from "@/stores/settings";
import { formatMoney, formatOperatingTime } from "@/lib/utils";
import { useVehicleImages } from "@/composables/useVehicleImages";
import type { SaleItem } from "@/lib/types";
import SaleItemEditor from "@/components/sales/SaleItemEditor.vue";
import VehicleImage from "@/components/vehicles/VehicleImage.vue";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

const { t } = useI18n();
const store = useSaleStore();
const settings = useSettingsStore();
const { loadBatch } = useVehicleImages();

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
    <div>
      <h2 class="text-2xl font-semibold">
        {{ t("sale.title") }}
        <span class="text-muted-foreground font-normal">({{ t("sale.count", { count: itemCount }) }})</span>
      </h2>
      <p class="text-sm text-muted-foreground">
        {{ t("sale.subtitle") }}
      </p>
    </div>

    <!-- Table -->
    <div class="rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead v-if="settings.gamePath" class="w-14" />
            <TableHead>{{ t("sale.name") }}</TableHead>
            <TableHead class="text-right">{{ t("sale.price") }}</TableHead>
            <TableHead class="text-right">{{ t("sale.wear") }}</TableHead>
            <TableHead class="text-right">{{ t("sale.damage") }}</TableHead>
            <TableHead class="text-right">{{ t("sale.days") }}</TableHead>
            <TableHead class="text-right">{{ t("sale.hours") }}</TableHead>
            <TableHead>{{ t("sale.source") }}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-if="itemCount === 0">
            <TableCell :colspan="settings.gamePath ? 8 : 7" class="py-8 text-center text-muted-foreground">
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
            <TableCell class="text-right font-mono">
              {{ formatMoney(item.price) }} $
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
              <Badge v-if="item.isGenerated" variant="secondary">{{ t("sale.generated") }}</Badge>
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
      :open="editorOpen"
      @update:open="editorOpen = $event"
    />
  </div>
</template>
