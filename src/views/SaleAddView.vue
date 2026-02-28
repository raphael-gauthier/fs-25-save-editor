<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { useSaleStore } from "@/stores/sale";
import { useSettingsStore } from "@/stores/settings";
import { useVehicleImages } from "@/composables/useVehicleImages";
import { useTauri } from "@/composables/useTauri";
import { formatMoney } from "@/lib/utils";
import type { CatalogVehicle } from "@/lib/types";
import VehicleImage from "@/components/vehicles/VehicleImage.vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ArrowLeft, Search, Plus, Loader2, AlertTriangle } from "lucide-vue-next";

const { t } = useI18n();
const router = useRouter();
const store = useSaleStore();
const settings = useSettingsStore();
const { invokeCommand } = useTauri();
const { loadBatch } = useVehicleImages();

const catalog = ref<CatalogVehicle[]>([]);
const isLoading = ref(false);
const loadError = ref<string | null>(null);

// Filters
const searchQuery = ref("");
const selectedBrand = ref("__all__");
const selectedCategory = ref("__all__");
const selectedSource = ref("__all__");

// Selection & config
const selectedVehicle = ref<CatalogVehicle | null>(null);
const configPrice = ref(0);
const configDiscount = ref(0);
const configWear = ref(0);
const configDamage = ref(0);
const configAge = ref(0);
const configOperatingHours = ref(0);
const configTimeLeft = ref(30);

// Bidirectional price ↔ discount sync
let updatingFrom: "price" | "discount" | null = null;

watch(configPrice, (price) => {
  if (updatingFrom === "discount") return;
  updatingFrom = "price";
  if (selectedVehicle.value && selectedVehicle.value.price > 0) {
    configDiscount.value = Math.round((1 - price / selectedVehicle.value.price) * 100);
  }
  updatingFrom = null;
});

watch(configDiscount, (discount) => {
  if (updatingFrom === "price") return;
  updatingFrom = "discount";
  if (selectedVehicle.value) {
    configPrice.value = Math.round(selectedVehicle.value.price * (1 - discount / 100));
  }
  updatingFrom = null;
});

const brands = computed(() => {
  const set = new Set(catalog.value.map((v) => v.brand));
  return [...set].sort();
});

const categories = computed(() => {
  const set = new Set(catalog.value.map((v) => v.category));
  return [...set].sort();
});

const filteredCatalog = computed(() => {
  let result = catalog.value;

  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase();
    result = result.filter(
      (v) =>
        v.name.toLowerCase().includes(q) ||
        v.brand.toLowerCase().includes(q),
    );
  }

  if (selectedBrand.value !== "__all__") {
    result = result.filter((v) => v.brand === selectedBrand.value);
  }

  if (selectedCategory.value !== "__all__") {
    result = result.filter((v) => v.category === selectedCategory.value);
  }

  if (selectedSource.value !== "__all__") {
    if (selectedSource.value === "baseGame") {
      result = result.filter((v) => v.source === "baseGame");
    } else {
      result = result.filter((v) => typeof v.source === "object");
    }
  }

  return result;
});

function selectVehicle(vehicle: CatalogVehicle) {
  selectedVehicle.value = vehicle;
  updatingFrom = "price";
  configPrice.value = vehicle.price;
  configDiscount.value = 0;
  updatingFrom = null;
  configWear.value = 0;
  configDamage.value = 0;
  configAge.value = 0;
  configOperatingHours.value = 0;
  configTimeLeft.value = 30;
}

function clearSelection() {
  selectedVehicle.value = null;
}

function addToMarket() {
  if (!selectedVehicle.value) return;

  store.addItem({
    xmlFilename: selectedVehicle.value.xmlFilename,
    price: configPrice.value,
    damage: configDamage.value / 100,
    wear: configWear.value / 100,
    age: configAge.value,
    operatingTime: configOperatingHours.value * 60, // hours → minutes
    timeLeft: configTimeLeft.value,
  });

  router.push({ name: "sales" });
}

function goBack() {
  router.push({ name: "sales" });
}

onMounted(async () => {
  if (!settings.gamePath) return;

  isLoading.value = true;
  loadError.value = null;
  try {
    catalog.value = await invokeCommand<CatalogVehicle[]>(
      "get_vehicle_catalog",
      { gamePath: settings.gamePath },
    );
    // Load images for visible vehicles
    if (catalog.value.length > 0) {
      loadBatch(
        settings.gamePath,
        catalog.value.map((v) => v.xmlFilename),
      );
    }
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e);
  } finally {
    isLoading.value = false;
  }
});
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center gap-4">
      <Button variant="ghost" size="icon" @click="goBack">
        <ArrowLeft class="size-5" />
      </Button>
      <div>
        <h2 class="text-2xl font-semibold">{{ t("sale.catalog") }}</h2>
        <p class="text-sm text-muted-foreground">
          {{ t("sale.catalogSubtitle") }}
        </p>
      </div>
    </div>

    <!-- No game path -->
    <div
      v-if="!settings.gamePath"
      class="flex flex-col items-center gap-4 rounded-md border border-dashed p-8 text-center"
    >
      <AlertTriangle class="size-10 text-muted-foreground" />
      <p class="text-sm text-muted-foreground">{{ t("sale.noGamePath") }}</p>
      <Button variant="outline" @click="router.push({ name: 'settings' })">
        {{ t("sale.goToSettings") }}
      </Button>
    </div>

    <!-- Loading -->
    <div
      v-else-if="isLoading"
      class="flex flex-col items-center gap-4 py-16"
    >
      <Loader2 class="size-8 animate-spin text-muted-foreground" />
      <p class="text-sm text-muted-foreground">{{ t("sale.catalogLoading") }}</p>
    </div>

    <!-- Error -->
    <div
      v-else-if="loadError"
      class="flex flex-col items-center gap-4 rounded-md border border-destructive/50 p-8 text-center"
    >
      <AlertTriangle class="size-10 text-destructive" />
      <p class="text-sm text-destructive">{{ loadError }}</p>
    </div>

    <!-- Content: Vehicle selected → config form -->
    <template v-else-if="selectedVehicle">
      <Card>
        <CardContent class="pt-6">
          <div class="flex items-start gap-6">
            <!-- Vehicle info -->
            <VehicleImage :filename="selectedVehicle.xmlFilename" size="lg" />
            <div class="flex-1 space-y-1">
              <div class="flex items-center gap-2">
                <h3 class="text-lg font-semibold">{{ selectedVehicle.name }}</h3>
                <Badge variant="secondary">{{ selectedVehicle.brand }}</Badge>
                <Badge v-if="typeof selectedVehicle.source === 'object'" variant="outline">
                  {{ t("sale.sourceMod") }}
                </Badge>
              </div>
              <p class="text-sm text-muted-foreground">
                {{ t("sale.catalogPrice") }}: {{ formatMoney(selectedVehicle.price) }} $
              </p>
              <Button variant="ghost" size="sm" class="mt-2" @click="clearSelection">
                {{ t("sale.cancel") }}
              </Button>
            </div>
          </div>

          <Separator class="my-6" />

          <h4 class="mb-4 text-sm font-medium uppercase tracking-wide text-muted-foreground">
            {{ t("sale.configureVehicle") }}
          </h4>

          <div class="space-y-6">
            <!-- Price & Discount -->
            <div class="grid grid-cols-2 gap-4">
              <div class="space-y-2">
                <Label>{{ t("sale.price") }}</Label>
                <div class="flex items-center gap-2">
                  <Input
                    v-model.number="configPrice"
                    type="number"
                    class="font-mono"
                    min="0"
                    step="1000"
                  />
                  <span class="shrink-0 text-sm text-muted-foreground">$</span>
                </div>
                <p class="text-xs text-muted-foreground">
                  {{ formatMoney(configPrice) }} $
                </p>
              </div>
              <div class="space-y-2">
                <Label>{{ t("sale.discount") }}</Label>
                <div class="flex items-center gap-2">
                  <Input
                    v-model.number="configDiscount"
                    type="number"
                    class="font-mono"
                    min="0"
                    max="100"
                    step="5"
                  />
                  <span class="shrink-0 text-sm text-muted-foreground">%</span>
                </div>
              </div>
            </div>

            <!-- Wear -->
            <div class="space-y-2">
              <div class="flex items-center justify-between text-sm">
                <Label>{{ t("sale.wear") }}</Label>
                <span class="font-mono text-muted-foreground">{{ configWear }}%</span>
              </div>
              <Slider
                :model-value="[configWear]"
                :max="100"
                :min="0"
                :step="1"
                @update:model-value="(v: number[] | undefined) => { if (v) configWear = v[0] }"
              />
            </div>

            <!-- Damage -->
            <div class="space-y-2">
              <div class="flex items-center justify-between text-sm">
                <Label>{{ t("sale.damage") }}</Label>
                <span class="font-mono text-muted-foreground">{{ configDamage }}%</span>
              </div>
              <Slider
                :model-value="[configDamage]"
                :max="100"
                :min="0"
                :step="1"
                @update:model-value="(v: number[] | undefined) => { if (v) configDamage = v[0] }"
              />
            </div>

            <!-- Age -->
            <div class="space-y-2">
              <Label>{{ t("sale.ageDays") }}</Label>
              <Input
                v-model.number="configAge"
                type="number"
                class="font-mono"
                min="0"
                step="1"
              />
            </div>

            <!-- Operating hours -->
            <div class="space-y-2">
              <Label>{{ t("sale.operatingHours") }}</Label>
              <Input
                v-model.number="configOperatingHours"
                type="number"
                class="font-mono"
                min="0"
                step="1"
              />
            </div>

            <!-- Time left -->
            <div class="space-y-2">
              <Label>{{ t("sale.timeLeftDays") }}</Label>
              <Input
                v-model.number="configTimeLeft"
                type="number"
                class="font-mono"
                min="1"
                step="1"
              />
            </div>
          </div>

          <Separator class="my-6" />

          <div class="flex gap-3">
            <Button @click="addToMarket">
              <Plus class="size-4" />
              {{ t("sale.addToMarket") }}
            </Button>
            <Button variant="outline" @click="clearSelection">
              {{ t("sale.cancel") }}
            </Button>
          </div>
        </CardContent>
      </Card>
    </template>

    <!-- Content: Catalog browser -->
    <template v-else>
      <!-- Filters -->
      <div class="flex flex-wrap items-center gap-3">
        <div class="relative flex-1 min-w-[200px]">
          <Search class="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            v-model="searchQuery"
            :placeholder="t('sale.searchPlaceholder')"
            class="pl-9"
          />
        </div>

        <Select v-model="selectedBrand">
          <SelectTrigger class="w-48">
            <SelectValue :placeholder="t('sale.filterBrand')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="__all__">{{ t("sale.allBrands") }}</SelectItem>
            <SelectItem v-for="brand in brands" :key="brand" :value="brand">
              {{ brand }}
            </SelectItem>
          </SelectContent>
        </Select>

        <Select v-model="selectedCategory">
          <SelectTrigger class="w-48">
            <SelectValue :placeholder="t('sale.filterCategory')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="__all__">{{ t("sale.allCategories") }}</SelectItem>
            <SelectItem v-for="cat in categories" :key="cat" :value="cat">
              {{ cat }}
            </SelectItem>
          </SelectContent>
        </Select>

        <Select v-model="selectedSource">
          <SelectTrigger class="w-40">
            <SelectValue :placeholder="t('sale.filterSource')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="__all__">{{ t("sale.filterSource") }}</SelectItem>
            <SelectItem value="baseGame">{{ t("sale.sourceBaseGame") }}</SelectItem>
            <SelectItem value="mod">{{ t("sale.sourceMod") }}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <!-- Result count -->
      <p class="text-sm text-muted-foreground">
        {{ t("sale.vehiclesFound", { count: filteredCatalog.length }) }}
      </p>

      <!-- Vehicle grid -->
      <div
        v-if="filteredCatalog.length > 0"
        class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5"
      >
        <Card
          v-for="vehicle in filteredCatalog"
          :key="vehicle.xmlFilename"
          class="cursor-pointer transition-colors hover:bg-muted/50"
          @click="selectVehicle(vehicle)"
        >
          <CardContent class="flex flex-col items-center gap-2 p-3">
            <VehicleImage :filename="vehicle.xmlFilename" size="md" />
            <div class="w-full text-center">
              <p class="text-xs font-medium leading-tight truncate" :title="vehicle.name">
                {{ vehicle.name }}
              </p>
              <p class="text-xs text-muted-foreground truncate">{{ vehicle.brand }}</p>
              <p class="text-xs font-mono text-muted-foreground">
                {{ formatMoney(vehicle.price) }} $
              </p>
            </div>
            <Badge
              v-if="typeof vehicle.source === 'object'"
              variant="outline"
              class="text-[10px]"
            >
              {{ t("sale.sourceMod") }}
            </Badge>
          </CardContent>
        </Card>
      </div>

      <!-- Empty state -->
      <div
        v-else
        class="flex flex-col items-center gap-2 py-12 text-center text-muted-foreground"
      >
        <Search class="size-10" />
        <p class="text-sm">{{ t("sale.catalogEmpty") }}</p>
      </div>
    </template>
  </div>
</template>
