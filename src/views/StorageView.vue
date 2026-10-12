<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useBuildingStore } from "@/stores/building";
import type { Placeable, SiloCapacity, SiloStorage } from "@/lib/types";
import { formatMoney } from "@/lib/utils";
import { groupObjects } from "@/lib/storageChanges";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
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
import { Minus, Plus, Trash2, Wand2, Warehouse } from "@lucide/vue";

const { t } = useI18n();
const store = useBuildingStore();

const siloPlaceables = computed(() => store.placeables.filter((p) => p.siloStorages.length > 0));
const objectPlaceables = computed(() =>
  store.placeables.filter(
    (p) =>
      p.storedObjects.length > 0 ||
      store.storageCapacities.objectStorages.some((o) => o.placeableIndex === p.index),
  ),
);

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
  const amount = max === undefined ? value : Math.min(value, max);
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
</script>

<template>
  <div class="space-y-6">
    <div>
      <h2 class="text-2xl font-semibold">{{ t("storage.title") }}</h2>
      <p class="text-sm text-muted-foreground">{{ t("storage.subtitle") }}</p>
    </div>

    <!-- Silos -->
    <section class="space-y-3">
      <h3 class="text-lg font-medium">{{ t("storage.silos") }}</h3>
      <p v-if="siloPlaceables.length === 0" class="text-sm text-muted-foreground">{{ t("storage.noSilos") }}</p>
      <div class="grid gap-4 lg:grid-cols-2">
        <template v-for="p in siloPlaceables" :key="p.index">
          <Card v-for="storage in p.siloStorages" :key="storageKey(p, storage)">
            <CardHeader>
              <CardTitle class="flex items-center gap-2 text-base">
                <Warehouse class="size-4" />
                {{ p.displayName }}
                <Badge v-if="storage.kind === 'extension'" variant="secondary">{{ t("storage.extension") }}</Badge>
              </CardTitle>
              <CardDescription>
                {{ ownerLabel(storage.farmId) }}
                <template v-if="capacityOf(p, storage)">
                  · {{ liters(used(storage)) }} / {{ liters(capacityOf(p, storage)!.capacity) }}
                </template>
              </CardDescription>
              <div v-if="capacityOf(p, storage)" class="h-2 rounded-full bg-muted">
                <div
                  class="h-2 rounded-full bg-primary"
                  :style="{ width: Math.min(100, (used(storage) / capacityOf(p, storage)!.capacity) * 100) + '%' }"
                />
              </div>
            </CardHeader>
            <CardContent class="space-y-2">
              <div
                v-for="stock in storage.stocks.filter((s) => s.amount > 0)"
                :key="stock.fillType"
                class="flex items-center gap-2"
              >
                <span class="w-32 truncate text-sm">{{ fillTypeLabel(stock.fillType) }}</span>
                <Input
                  type="number"
                  min="0"
                  step="1000"
                  class="h-8 flex-1 font-mono"
                  :aria-label="fillTypeLabel(stock.fillType)"
                  :model-value="Math.round(stock.amount)"
                  @change="setLevel(p, storage, stock.fillType, Number(($event.target as HTMLInputElement).value))"
                />
                <Button
                  v-if="maxFor(storage, capacityOf(p, storage), stock.fillType) !== undefined"
                  size="sm"
                  variant="outline"
                  @click="setLevel(p, storage, stock.fillType, maxFor(storage, capacityOf(p, storage), stock.fillType)!)"
                >
                  {{ t("storage.fill") }}
                </Button>
                <Button size="icon-sm" variant="ghost" :aria-label="t('storage.empty')" @click="setLevel(p, storage, stock.fillType, 0)">
                  <Trash2 class="size-4" />
                </Button>
              </div>
              <p v-if="storage.stocks.every((s) => s.amount <= 0)" class="text-sm text-muted-foreground">
                {{ t("storage.emptySilo") }}
              </p>
              <div v-if="addableFillTypes(p, storage).length > 0" class="flex items-center gap-2 pt-2">
                <Select
                  :model-value="newFillType[storageKey(p, storage)] ?? ''"
                  @update:model-value="newFillType = { ...newFillType, [storageKey(p, storage)]: String($event) }"
                >
                  <SelectTrigger class="h-8 flex-1" :aria-label="t('storage.addFillType')">
                    <SelectValue :placeholder="t('storage.addFillType')" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem v-for="fillType in addableFillTypes(p, storage)" :key="fillType" :value="fillType">
                      {{ fillTypeLabel(fillType) }}
                    </SelectItem>
                  </SelectContent>
                </Select>
                <Button size="sm" variant="outline" :disabled="!newFillType[storageKey(p, storage)]" @click="addFillType(p, storage)">
                  <Plus class="size-4" />
                  {{ t("storage.add") }}
                </Button>
              </div>
            </CardContent>
          </Card>
        </template>
      </div>
    </section>

    <!-- Object storages -->
    <section class="space-y-3">
      <h3 class="text-lg font-medium">{{ t("storage.objectStorages") }}</h3>
      <p v-if="objectPlaceables.length === 0" class="text-sm text-muted-foreground">{{ t("storage.noObjectStorages") }}</p>
      <Card v-for="p in objectPlaceables" :key="p.index">
        <CardHeader>
          <CardTitle class="flex flex-wrap items-center justify-between gap-2 text-base">
            <span class="flex items-center gap-2">
              <Warehouse class="size-4" />
              {{ p.displayName }}
            </span>
            <Button v-if="hasFermenting(p)" size="sm" variant="outline" @click="store.finishFermentation(p.index)">
              <Wand2 class="size-4" />
              {{ t("storage.finishFermentation") }}
            </Button>
          </CardTitle>
          <CardDescription>
            {{ t("storage.objectCount", { count: p.storedObjects.length }) }}
            <template v-if="objectCapacity(p) !== undefined"> / {{ objectCapacity(p) }}</template>
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Table v-if="p.storedObjects.length > 0">
            <TableHeader>
              <TableRow>
                <TableHead>{{ t("storage.kind") }}</TableHead>
                <TableHead>{{ t("storage.content") }}</TableHead>
                <TableHead>{{ t("storage.model") }}</TableHead>
                <TableHead class="text-right">{{ t("storage.fillLevel") }}</TableHead>
                <TableHead class="w-48 text-right">{{ t("storage.count") }}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow v-for="group in groupObjects(p.storedObjects)" :key="group.key">
                <TableCell>
                  {{ group.object.className === "Bale" ? t("storage.bale") : t("storage.pallet") }}
                  <Badge v-if="group.object.wrappingState > 0" variant="secondary" class="ml-1">{{ t("storage.wrapped") }}</Badge>
                  <Badge
                    v-if="group.object.isFermenting && !store.fermentationToFinish.has(p.index)"
                    variant="outline"
                    class="ml-1"
                  >
                    {{ t("storage.fermenting") }}
                  </Badge>
                </TableCell>
                <TableCell>{{ fillTypeLabel(group.object.fillType) }}</TableCell>
                <TableCell class="font-mono text-xs text-muted-foreground">{{ modelName(group.object.filename) }}</TableCell>
                <TableCell class="text-right font-mono">{{ liters(group.object.fillLevel) }}</TableCell>
                <TableCell class="text-right">
                  <div class="flex items-center justify-end gap-1">
                    <Button size="icon-sm" variant="outline" :aria-label="t('storage.remove')" @click="store.changeObjectCount(p.index, group.key, -1)">
                      <Minus class="size-4" />
                    </Button>
                    <span class="w-10 text-center font-mono">{{ group.indices.length }}</span>
                    <Button
                      size="icon-sm"
                      variant="outline"
                      :aria-label="t('storage.add')"
                      :disabled="!canAdd(p, 1)"
                      @click="store.changeObjectCount(p.index, group.key, 1)"
                    >
                      <Plus class="size-4" />
                    </Button>
                    <Button size="sm" variant="ghost" :disabled="!canAdd(p, 10)" @click="store.changeObjectCount(p.index, group.key, 10)">
                      +10
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
          <p v-else class="text-sm text-muted-foreground">{{ t("storage.emptyObjectStorage") }}</p>
        </CardContent>
      </Card>
    </section>
  </div>
</template>
