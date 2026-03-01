<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useEconomyStore } from "@/stores/economy";
import { useSettingsStore } from "@/stores/settings";
import type { GreatDemand, GreatDemandAdditionPayload } from "@/lib/types";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { TrendingUp, Plus, Trash2 } from "lucide-vue-next";

const PERIODS = [
  "EARLY_SPRING",
  "MID_SPRING",
  "LATE_SPRING",
  "EARLY_SUMMER",
  "MID_SUMMER",
  "LATE_SUMMER",
  "EARLY_AUTUMN",
  "MID_AUTUMN",
  "LATE_AUTUMN",
  "EARLY_WINTER",
  "MID_WINTER",
  "LATE_WINTER",
];

const { t } = useI18n();
const store = useEconomyStore();
const settings = useSettingsStore();

// Great demand editor sheet
const editorOpen = ref(false);
const selectedDemand = ref<GreatDemand | null>(null);

// Add demand dialog
const addDialogOpen = ref(false);
const newDemand = ref<GreatDemandAdditionPayload>({
  uniqueId: "",
  fillTypeName: "",
  demandMultiplier: 1.0,
  demandStartDay: 1,
  demandStartHour: 0,
  demandDuration: 24,
});

function openEditor(demand: GreatDemand) {
  selectedDemand.value = demand;
  editorOpen.value = true;
}

function translateFillType(name: string): string {
  const key = `fillTypes.${name}`;
  const translated = t(key);
  return translated === key ? name : translated;
}

function periodShort(period: string): string {
  const key = `periods.${period}`;
  const translated = t(key);
  return translated === key ? period : translated;
}

function getPriceClass(price: number, prices: number[]): string {
  if (prices.length === 0) return "";
  const min = Math.min(...prices);
  const max = Math.max(...prices);
  if (min === max) return "";
  if (price === max) return "text-green-600 dark:text-green-400 font-semibold";
  if (price === min) return "text-red-600 dark:text-red-400";
  return "";
}

// Editor handlers
function handleMultiplierChange(event: Event) {
  const target = event.target as HTMLInputElement;
  const value = parseFloat(target.value);
  if (selectedDemand.value && !isNaN(value) && value > 0) {
    store.updateDemand(selectedDemand.value.index, { demandMultiplier: value });
  }
}

function handleStartDayChange(event: Event) {
  const target = event.target as HTMLInputElement;
  const value = parseInt(target.value);
  if (selectedDemand.value && !isNaN(value)) {
    store.updateDemand(selectedDemand.value.index, { demandStartDay: value });
  }
}

function handleStartHourChange(event: Event) {
  const target = event.target as HTMLInputElement;
  const value = parseInt(target.value);
  if (selectedDemand.value && !isNaN(value)) {
    store.updateDemand(selectedDemand.value.index, { demandStartHour: value });
  }
}

function handleDurationChange(event: Event) {
  const target = event.target as HTMLInputElement;
  const value = parseInt(target.value);
  if (selectedDemand.value && !isNaN(value) && value > 0) {
    store.updateDemand(selectedDemand.value.index, { demandDuration: value });
  }
}

function handleDelete(demand: GreatDemand) {
  store.deleteDemand(demand.index);
}

function openAddDialog() {
  const existingIds = store.greatDemands.map((d) => d.uniqueId);
  newDemand.value = {
    uniqueId: existingIds[0] || "",
    fillTypeName: "WHEAT",
    demandMultiplier: 1.0,
    demandStartDay: 1,
    demandStartHour: 0,
    demandDuration: 24,
  };
  addDialogOpen.value = true;
}

function handleAddDemand() {
  if (newDemand.value.uniqueId && newDemand.value.fillTypeName) {
    store.addDemand({ ...newDemand.value });
    addDialogOpen.value = false;
  }
}

// Get unique station IDs from existing demands
function getStationIds(): string[] {
  const ids = new Set(store.greatDemands.map((d) => d.uniqueId));
  return [...ids];
}
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-2xl font-semibold">{{ t("economy.title") }}</h2>
        <p class="text-sm text-muted-foreground">{{ t("economy.subtitle") }}</p>
      </div>
    </div>

    <Tabs default-value="demands">
      <TabsList>
        <TabsTrigger value="demands">{{ t("economy.greatDemands") }}</TabsTrigger>
        <TabsTrigger value="prices">{{ t("economy.marketPrices") }}</TabsTrigger>
      </TabsList>

      <!-- Great Demands Tab -->
      <TabsContent value="demands" class="space-y-4">
        <div class="flex items-center justify-between">
          <span class="text-sm text-muted-foreground">
            {{ t("economy.found", { count: store.activeGreatDemands.length }) }}
          </span>
          <Button size="sm" @click="openAddDialog">
            <Plus class="mr-2 size-4" />
            {{ t("economy.addDemand") }}
          </Button>
        </div>

        <div v-if="store.activeGreatDemands.length > 0" class="rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{{ t("economy.station") }}</TableHead>
                <TableHead>{{ t("economy.fillType") }}</TableHead>
                <TableHead class="text-right">{{ t("economy.multiplier") }}</TableHead>
                <TableHead class="text-right">{{ t("economy.startDay") }}</TableHead>
                <TableHead class="text-right">{{ t("economy.startHour") }}</TableHead>
                <TableHead class="text-right">{{ t("economy.duration") }}</TableHead>
                <TableHead>{{ t("economy.running") }}</TableHead>
                <TableHead></TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow
                v-for="d in store.activeGreatDemands"
                :key="d.index"
                class="cursor-pointer"
                @click="openEditor(d)"
              >
                <TableCell class="font-mono text-xs">{{ d.uniqueId }}</TableCell>
                <TableCell>{{ translateFillType(d.fillTypeName) }}</TableCell>
                <TableCell class="text-right font-mono">
                  x{{ d.demandMultiplier.toFixed(2) }}
                </TableCell>
                <TableCell class="text-right font-mono">{{ d.demandStartDay }}</TableCell>
                <TableCell class="text-right font-mono">{{ d.demandStartHour }}h</TableCell>
                <TableCell class="text-right font-mono">{{ d.demandDuration }}h</TableCell>
                <TableCell>
                  <Badge :variant="d.isRunning ? 'default' : 'outline'" class="text-xs">
                    {{ d.isRunning ? t("economy.yes") : t("economy.no") }}
                  </Badge>
                </TableCell>
                <TableCell>
                  <AlertDialog>
                    <AlertDialogTrigger as-child>
                      <Button
                        variant="ghost"
                        size="sm"
                        class="size-8 p-0"
                        @click.stop
                      >
                        <Trash2 class="size-4 text-destructive" />
                      </Button>
                    </AlertDialogTrigger>
                    <AlertDialogContent @click.stop>
                      <AlertDialogHeader>
                        <AlertDialogTitle>{{ t("economy.deleteTitle") }}</AlertDialogTitle>
                        <AlertDialogDescription>
                          {{ t("economy.deleteDesc", { fillType: translateFillType(d.fillTypeName), station: d.uniqueId }) }}
                        </AlertDialogDescription>
                      </AlertDialogHeader>
                      <AlertDialogFooter>
                        <AlertDialogCancel>{{ t("common.cancel") }}</AlertDialogCancel>
                        <AlertDialogAction @click="handleDelete(d)">
                          {{ t("common.delete") }}
                        </AlertDialogAction>
                      </AlertDialogFooter>
                    </AlertDialogContent>
                  </AlertDialog>
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>

        <!-- Additions pending -->
        <div v-if="store.additions.length > 0" class="rounded-md border p-4">
          <p class="mb-2 text-sm font-medium text-muted-foreground">
            {{ t("economy.addDemand") }} ({{ store.additions.length }})
          </p>
          <div
            v-for="(a, i) in store.additions"
            :key="i"
            class="flex items-center gap-4 text-sm"
          >
            <span class="font-mono text-xs">{{ a.uniqueId }}</span>
            <span>{{ translateFillType(a.fillTypeName) }}</span>
            <span class="font-mono">x{{ a.demandMultiplier.toFixed(2) }}</span>
            <Badge variant="secondary" class="text-xs">{{ t("sale.new") }}</Badge>
          </div>
        </div>

        <!-- Empty state -->
        <div v-if="store.activeGreatDemands.length === 0 && store.additions.length === 0" class="py-12 text-center">
          <TrendingUp class="mx-auto size-12 text-muted-foreground/50" />
          <p class="mt-4 text-muted-foreground">{{ t("economy.emptyDemands") }}</p>
        </div>
      </TabsContent>

      <!-- Market Prices Tab -->
      <TabsContent value="prices" class="space-y-4">
        <div class="flex flex-wrap items-center gap-3">
          <Input
            v-model="store.priceSearchQuery"
            :placeholder="t('economy.searchPlaceholder')"
            class="max-w-xs"
          />
          <div class="flex items-center gap-2">
            <Switch
              :checked="store.showAllProducts"
              @update:checked="store.showAllProducts = $event"
            />
            <Label class="text-sm">{{ t("economy.showAll") }}</Label>
          </div>
          <span class="text-sm text-muted-foreground">
            {{ t("economy.found", { count: store.filteredFillTypes.length }) }}
          </span>
        </div>

        <div v-if="store.filteredFillTypes.length > 0" class="rounded-md border overflow-x-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead class="sticky left-0 bg-background z-10">{{ t("economy.product") }}</TableHead>
                <TableHead class="text-right">{{ t("economy.totalAmount") }}</TableHead>
                <TableHead
                  v-for="period in PERIODS"
                  :key="period"
                  class="text-right text-xs whitespace-nowrap"
                >
                  {{ periodShort(period) }}
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow v-for="ft in store.filteredFillTypes" :key="ft.fillType">
                <TableCell class="sticky left-0 bg-background z-10 font-medium">
                  {{ translateFillType(ft.fillType) }}
                </TableCell>
                <TableCell class="text-right font-mono">
                  {{ ft.totalAmount?.toLocaleString() ?? "-" }}
                </TableCell>
                <TableCell
                  v-for="period in PERIODS"
                  :key="period"
                  class="text-right font-mono text-xs"
                  :class="getPriceClass(
                    ft.priceHistory.find((p) => p.period === period)?.price ?? 0,
                    ft.priceHistory.map((p) => p.price)
                  )"
                >
                  {{ ft.priceHistory.find((p) => p.period === period)?.price ?? "-" }}
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>

        <!-- Empty state -->
        <div v-else-if="store.fillTypes.length === 0" class="py-12 text-center">
          <TrendingUp class="mx-auto size-12 text-muted-foreground/50" />
          <p class="mt-4 text-muted-foreground">{{ t("economy.emptyPrices") }}</p>
        </div>

        <!-- No match -->
        <div v-else class="py-12 text-center">
          <p class="text-muted-foreground">{{ t("economy.noMatchPrices") }}</p>
        </div>
      </TabsContent>
    </Tabs>

    <!-- Demand Editor Sheet -->
    <Sheet :open="editorOpen" @update:open="editorOpen = $event">
      <SheetContent class="overflow-y-auto sm:max-w-md">
        <template v-if="selectedDemand">
          <SheetHeader>
            <SheetTitle class="flex items-center gap-2">
              <TrendingUp class="size-5" />
              {{ t("economy.editDemand") }}
            </SheetTitle>
            <SheetDescription>
              {{ selectedDemand.uniqueId }} &middot; {{ translateFillType(selectedDemand.fillTypeName) }}
            </SheetDescription>
          </SheetHeader>

          <div class="mt-6 space-y-6">
            <div class="space-y-2">
              <Label>{{ t("economy.demandMultiplier") }}</Label>
              <Input
                type="number"
                step="0.01"
                min="0"
                :model-value="selectedDemand.demandMultiplier"
                @change="handleMultiplierChange"
              />
            </div>

            <div class="space-y-2">
              <Label>{{ t("economy.demandStartDay") }}</Label>
              <Input
                type="number"
                min="0"
                :model-value="selectedDemand.demandStartDay"
                @change="handleStartDayChange"
              />
            </div>

            <div class="space-y-2">
              <Label>{{ t("economy.demandStartHour") }}</Label>
              <Input
                type="number"
                min="0"
                max="23"
                :model-value="selectedDemand.demandStartHour"
                @change="handleStartHourChange"
              />
            </div>

            <div class="space-y-2">
              <Label>{{ t("economy.demandDuration") }}</Label>
              <Input
                type="number"
                min="1"
                :model-value="selectedDemand.demandDuration"
                @change="handleDurationChange"
              />
            </div>

            <template v-if="settings.advancedMode">
              <div class="flex items-center justify-between">
                <Label>{{ t("economy.running") }}</Label>
                <Badge :variant="selectedDemand.isRunning ? 'default' : 'outline'" class="text-xs">
                  {{ selectedDemand.isRunning ? t("economy.yes") : t("economy.no") }}
                </Badge>
              </div>
              <div class="flex items-center justify-between">
                <Label>{{ t("economy.valid") }}</Label>
                <Badge :variant="selectedDemand.isValid ? 'default' : 'outline'" class="text-xs">
                  {{ selectedDemand.isValid ? t("economy.yes") : t("economy.no") }}
                </Badge>
              </div>
            </template>
          </div>
        </template>
      </SheetContent>
    </Sheet>

    <!-- Add Demand Dialog -->
    <Dialog :open="addDialogOpen" @update:open="addDialogOpen = $event">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{{ t("economy.addDemand") }}</DialogTitle>
          <DialogDescription>
            {{ t("economy.subtitle") }}
          </DialogDescription>
        </DialogHeader>

        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <Label>{{ t("economy.stationId") }}</Label>
            <Input
              v-model="newDemand.uniqueId"
              :placeholder="t('economy.stationId')"
              :list="getStationIds().length > 0 ? 'station-ids' : undefined"
            />
            <datalist v-if="getStationIds().length > 0" id="station-ids">
              <option v-for="id in getStationIds()" :key="id" :value="id" />
            </datalist>
          </div>

          <div class="space-y-2">
            <Label>{{ t("economy.fillType") }}</Label>
            <Input v-model="newDemand.fillTypeName" placeholder="WHEAT" />
          </div>

          <div class="space-y-2">
            <Label>{{ t("economy.demandMultiplier") }}</Label>
            <Input v-model.number="newDemand.demandMultiplier" type="number" step="0.01" min="0" />
          </div>

          <div class="grid grid-cols-3 gap-3">
            <div class="space-y-2">
              <Label>{{ t("economy.demandStartDay") }}</Label>
              <Input v-model.number="newDemand.demandStartDay" type="number" min="0" />
            </div>
            <div class="space-y-2">
              <Label>{{ t("economy.demandStartHour") }}</Label>
              <Input v-model.number="newDemand.demandStartHour" type="number" min="0" max="23" />
            </div>
            <div class="space-y-2">
              <Label>{{ t("economy.demandDuration") }}</Label>
              <Input v-model.number="newDemand.demandDuration" type="number" min="1" />
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" @click="addDialogOpen = false">
            {{ t("common.cancel") }}
          </Button>
          <Button
            :disabled="!newDemand.uniqueId || !newDemand.fillTypeName"
            @click="handleAddDemand"
          >
            {{ t("economy.addDemand") }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
