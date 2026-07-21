<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useAnimalStore } from "@/stores/animal";
import type { FillStock, Husbandry, MeadowStock } from "@/lib/types";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Slider } from "@/components/ui/slider";
import { Separator } from "@/components/ui/separator";

const props = defineProps<{ husbandry: Husbandry }>();

const { t, te } = useI18n();
const store = useAnimalStore();

const totalAnimals = computed(() =>
  props.husbandry.clusters.reduce((sum, c) => sum + c.numAnimals, 0),
);

const isEmpty = computed(() => props.husbandry.clusters.length === 0);

/** Falls back to the raw code so DLC and modded species still display. */
function animalName(subType: string): string {
  const key = `animalTypes.${subType}`;
  return te(key) ? t(key) : subType;
}

function fillName(fillType: string): string {
  const key = `fillTypes.${fillType}`;
  return te(key) ? t(key) : fillType;
}

function ownerLabel(farmId: number): string {
  if (farmId === 0) return t("field.free");
  if (farmId === 1) return t("field.myFarm");
  return t("field.farm", { id: farmId });
}

/**
 * The savegame records no capacity for headcount, food or water, so no real maximum
 * can be enforced. Flag values far past the original instead of blocking them.
 *
 * A zero original carries no information — filling an empty trough or restocking an
 * emptied barn is ordinary — so it never warns. Warning there would be noise, and a
 * warning users learn to ignore is worse than none.
 */
function isSuspicious(current: number, original: number | undefined): boolean {
  if (original === undefined || original === 0) return false;
  return current > original * 2;
}

function warnClass(current: number, original: number | undefined): string {
  return isSuspicious(current, original)
    ? "border-amber-500 focus-visible:ring-amber-500"
    : "";
}

function onClusterInput(
  clusterIndex: number,
  field: "numAnimals" | "age",
  event: Event,
) {
  const value = Number((event.target as HTMLInputElement).value);
  if (Number.isFinite(value) && value >= 0) {
    store.updateCluster(props.husbandry.placeableIndex, clusterIndex, field, Math.round(value));
  }
}

function onClusterSlider(
  clusterIndex: number,
  field: "health" | "reproduction",
  value: number[] | undefined,
) {
  if (value?.length) {
    store.updateCluster(props.husbandry.placeableIndex, clusterIndex, field, value[0]);
  }
}

function onStockInput(kind: "food" | "water", fillType: string, event: Event) {
  const value = Number((event.target as HTMLInputElement).value);
  if (Number.isFinite(value) && value >= 0) {
    store.updateStock(props.husbandry.placeableIndex, kind, fillType, value);
  }
}

function onMeadowSlider(fillType: string, value: number[] | undefined) {
  if (value?.length) {
    store.updateStock(props.husbandry.placeableIndex, "meadow", fillType, value[0]);
  }
}

function originalCluster(
  clusterIndex: number,
  field: "numAnimals" | "age" | "health" | "reproduction",
) {
  return store.originalClusterValue(props.husbandry.placeableIndex, clusterIndex, field);
}

function originalStock(kind: "food" | "water", fillType: string) {
  return store.originalStockValue(props.husbandry.placeableIndex, kind, fillType);
}

function stockLabel(stock: FillStock | MeadowStock): string {
  return fillName(stock.fillType);
}
</script>

<template>
  <Card :class="isEmpty ? 'opacity-60' : ''">
    <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-3">
      <div>
        <p class="font-medium">{{ husbandry.displayName }}</p>
        <p class="text-sm text-muted-foreground">
          <template v-if="isEmpty">{{ t("animal.empty") }}</template>
          <template v-else>
            {{ t("animal.headcount", { count: totalAnimals }) }}
            <span v-if="husbandry.clusters.length > 1">
              · {{ t("animal.clusterCount", { count: husbandry.clusters.length }) }}
            </span>
          </template>
        </p>
      </div>
      <Badge variant="outline">{{ ownerLabel(husbandry.farmId) }}</Badge>
    </CardHeader>

    <CardContent v-if="!isEmpty" class="space-y-5">
      <!-- Clusters -->
      <div
        v-for="cluster in husbandry.clusters"
        :key="cluster.index"
        class="space-y-3 rounded-md border p-3"
      >
        <p class="text-sm font-medium">
          {{ animalName(cluster.subType) }}
          <span class="text-muted-foreground">
            · {{ t("animal.ageMonths", { count: cluster.age }) }}
          </span>
        </p>

        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-1">
            <Label class="text-xs">{{ t("animal.numAnimals") }}</Label>
            <Input
              type="number"
              min="0"
              step="1"
              :model-value="cluster.numAnimals"
              :class="warnClass(cluster.numAnimals, originalCluster(cluster.index, 'numAnimals'))"
              @change="onClusterInput(cluster.index, 'numAnimals', $event)"
            />
          </div>
          <div class="space-y-1">
            <Label class="text-xs">{{ t("animal.age") }}</Label>
            <Input
              type="number"
              min="0"
              step="1"
              :model-value="cluster.age"
              :class="warnClass(cluster.age, originalCluster(cluster.index, 'age'))"
              @change="onClusterInput(cluster.index, 'age', $event)"
            />
          </div>
        </div>

        <div class="space-y-1">
          <div class="flex items-center justify-between text-xs">
            <Label>{{ t("animal.health") }}</Label>
            <span class="font-mono text-muted-foreground">{{ Math.round(cluster.health) }}</span>
          </div>
          <Slider
            :model-value="[cluster.health]"
            :min="0"
            :max="100"
            :step="1"
            @update:model-value="onClusterSlider(cluster.index, 'health', $event)"
          />
        </div>

        <div class="space-y-1">
          <div class="flex items-center justify-between text-xs">
            <Label>{{ t("animal.reproduction") }}</Label>
            <span class="font-mono text-muted-foreground">
              {{ Math.round(cluster.reproduction) }}
            </span>
          </div>
          <Slider
            :model-value="[cluster.reproduction]"
            :min="0"
            :max="100"
            :step="1"
            @update:model-value="onClusterSlider(cluster.index, 'reproduction', $event)"
          />
        </div>
      </div>

      <!-- Stocks -->
      <template v-if="husbandry.food.length || husbandry.water.length || husbandry.meadow.length">
        <Separator />
        <div class="space-y-3">
          <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            {{ t("animal.stocks") }}
          </p>

          <div
            v-for="stock in husbandry.food"
            :key="`food-${stock.fillType}`"
            class="grid grid-cols-2 items-center gap-2"
          >
            <Label class="text-sm">{{ stockLabel(stock) }}</Label>
            <Input
              type="number"
              min="0"
              step="1"
              :model-value="Math.round(stock.amount)"
              :class="warnClass(stock.amount, originalStock('food', stock.fillType))"
              @change="onStockInput('food', stock.fillType, $event)"
            />
          </div>

          <div
            v-for="stock in husbandry.water"
            :key="`water-${stock.fillType}`"
            class="grid grid-cols-2 items-center gap-2"
          >
            <Label class="text-sm">{{ stockLabel(stock) }}</Label>
            <Input
              type="number"
              min="0"
              step="1"
              :model-value="Math.round(stock.amount)"
              :class="warnClass(stock.amount, originalStock('water', stock.fillType))"
              @change="onStockInput('water', stock.fillType, $event)"
            />
          </div>

          <!-- Pasture is the only stock with a real capacity, so it gets a true slider. -->
          <div v-for="stock in husbandry.meadow" :key="`meadow-${stock.fillType}`" class="space-y-1">
            <div class="flex items-center justify-between text-sm">
              <Label>{{ stockLabel(stock) }}</Label>
              <span class="font-mono text-xs text-muted-foreground">
                {{ Math.round(stock.amount) }} / {{ Math.round(stock.capacity) }}
              </span>
            </div>
            <Slider
              :model-value="[stock.amount]"
              :min="0"
              :max="stock.capacity"
              :step="1"
              @update:model-value="onMeadowSlider(stock.fillType, $event)"
            />
          </div>
        </div>
      </template>
    </CardContent>
  </Card>
</template>
