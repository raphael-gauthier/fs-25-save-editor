<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useFinanceStore } from "@/stores/finance";
import { useSettingsStore } from "@/stores/settings";
import { formatMoney } from "@/lib/utils";
import type { DailyFinance } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Switch } from "@/components/ui/switch";
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
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { Banknote, RotateCcw } from "lucide-vue-next";

const { t } = useI18n();
const finance = useFinanceStore();
const settings = useSettingsStore();

const isMoneyModified = computed(() => finance.money !== finance.originalMoney);
const isLoanModified = computed(() => finance.loan !== finance.originalLoan);

const showRepayDialog = ref(false);
const activeTab = ref("simple");

watch(() => settings.advancedMode, (advanced) => {
  if (!advanced) {
    activeTab.value = "simple";
  }
});

function handleMoneyInput(event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  if (!isNaN(value)) {
    finance.setMoney(value);
  }
}

function handleLoanInput(event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  if (!isNaN(value)) {
    finance.setLoan(value);
  }
}

const statGroups = [
  {
    key: "distances",
    fields: ["traveledDistance", "tractorDistance", "truckDistance", "carDistance", "horseDistance"],
  },
  {
    key: "consumption",
    fields: ["fuelUsage", "seedUsage", "sprayUsage"],
  },
  {
    key: "hectares",
    fields: ["workedHectares", "cultivatedHectares", "sownHectares", "sprayedHectares", "threshedHectares", "plowedHectares", "harvestedGrapes", "harvestedOlives"],
  },
  {
    key: "timeSpent",
    fields: ["workedTime", "cultivatedTime", "sownTime", "sprayedTime", "threshedTime", "plowedTime"],
  },
  {
    key: "counts",
    fields: ["baleCount", "wrappedBales", "soldCottonBales", "missionCount", "repairVehicleCount", "repaintVehicleCount"],
  },
  {
    key: "animals",
    fields: ["breedCowsCount", "breedSheepCount", "breedPigsCount", "breedChickenCount", "breedHorsesCount", "breedGoatsCount", "breedWaterBuffaloCount", "petDogCount", "horseJumpCount"],
  },
  {
    key: "trees",
    fields: ["plantedTreeCount", "cutTreeCount", "woodTonsSold"],
  },
  {
    key: "general",
    fields: ["playTime"],
  },
] as const;

const financeKeys = [
  "newVehiclesCost", "soldVehicles", "newAnimalsCost", "soldAnimals",
  "constructionCost", "soldBuildings", "fieldPurchase", "soldFields",
  "vehicleRunningCost", "vehicleLeasingCost", "propertyMaintenance", "propertyIncome",
  "productionCosts", "soldProducts", "harvestIncome", "missionIncome",
  "wagePayment", "loanInterest", "otherIncome", "otherExpenses",
] as const;

function formatStatValue(key: string, value: number): string {
  if (key === "playTime" || key.endsWith("Time")) return `${(value / 60).toFixed(1)}h`;
  if (key === "woodTonsSold") return `${value.toFixed(1)}t`;
  if (key.endsWith("Distance") || key.endsWith("Usage") || key.endsWith("Hectares") || key === "harvestedGrapes" || key === "harvestedOlives") return value.toFixed(1);
  if (Number.isInteger(value)) return value.toString();
  return value.toFixed(1);
}

const totalRevenue = computed(() => {
  return finance.dailyFinances.reduce((sum, df) => {
    const record = df as unknown as Record<string, number>;
    return sum + financeKeys.reduce((daySum, key) => {
      const val = record[key] ?? 0;
      return daySum + (val > 0 ? val : 0);
    }, 0);
  }, 0);
});

const totalExpenses = computed(() => {
  return finance.dailyFinances.reduce((sum, df) => {
    const record = df as unknown as Record<string, number>;
    return sum + financeKeys.reduce((daySum, key) => {
      const val = record[key] ?? 0;
      return daySum + (val < 0 ? val : 0);
    }, 0);
  }, 0);
});

function getDayTotal(df: DailyFinance): number {
  const record = df as unknown as Record<string, number>;
  return financeKeys.reduce((sum, key) => sum + (record[key] ?? 0), 0);
}

function getNonZeroEntries(df: DailyFinance): { key: string; value: number }[] {
  const record = df as unknown as Record<string, number>;
  return financeKeys
    .filter((key) => Math.abs(record[key] ?? 0) > 0.01)
    .map((key) => ({ key, value: record[key] ?? 0 }));
}

function formatDayLabel(day: number): string {
  if (day === 0) return t("finance.today");
  return `J-${day}`;
}

const sortedDailyFinances = computed(() =>
  [...finance.dailyFinances].sort((a, b) => a.day - b.day),
);
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-2xl font-semibold">{{ t("finance.title") }}</h2>
        <p class="text-sm text-muted-foreground">
          {{ t("finance.subtitle") }}
        </p>
      </div>
      <div class="flex items-center gap-2">
        <Label for="advanced-mode" class="text-sm">{{ t("common.advancedMode") }}</Label>
        <Switch id="advanced-mode" :model-value="settings.advancedMode" @update:model-value="settings.setAdvancedMode($event)" />
      </div>
    </div>

    <Tabs v-model="activeTab">
      <TabsList v-if="settings.advancedMode">
        <TabsTrigger value="simple">{{ t("finance.tabFinance") }}</TabsTrigger>
        <TabsTrigger value="stats">{{ t("finance.tabStats") }}</TabsTrigger>
        <TabsTrigger value="history">{{ t("finance.tabHistory") }}</TabsTrigger>
      </TabsList>

      <!-- Simple mode -->
      <TabsContent value="simple" class="space-y-6">
        <!-- Money -->
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center gap-2 text-base">
              <Banknote class="size-5" />
              {{ t("finance.availableMoney") }}
            </CardTitle>
          </CardHeader>
          <CardContent class="space-y-3">
            <div class="flex items-center gap-3" :class="isMoneyModified ? 'border-l-2 border-amber-500 pl-2' : ''">
              <Input
                type="number"
                :model-value="finance.money"
                @input="handleMoneyInput"
                class="max-w-xs font-mono"
                min="0"
                step="1000"
              />
              <span class="text-sm text-muted-foreground">
                {{ formatMoney(finance.money) }} $
              </span>
              <TooltipProvider v-if="isMoneyModified">
                <Tooltip>
                  <TooltipTrigger as-child>
                    <span class="size-2 rounded-full bg-amber-500" />
                  </TooltipTrigger>
                  <TooltipContent>
                    {{ t("dirtyTracking.originalValue", { value: formatMoney(finance.originalMoney) + " $" }) }}
                  </TooltipContent>
                </Tooltip>
              </TooltipProvider>
            </div>
            <div class="flex flex-wrap gap-2">
              <Button variant="outline" size="sm" @click="finance.addMoney(10000)">
                +10 000
              </Button>
              <Button variant="outline" size="sm" @click="finance.addMoney(100000)">
                +100 000
              </Button>
              <Button variant="outline" size="sm" @click="finance.addMoney(1000000)">
                +1 000 000
              </Button>
            </div>
          </CardContent>
        </Card>

        <!-- Loan -->
        <Card>
          <CardHeader>
            <CardTitle class="text-base">{{ t("finance.currentLoan") }}</CardTitle>
          </CardHeader>
          <CardContent class="space-y-3">
            <div class="flex items-center gap-3" :class="isLoanModified ? 'border-l-2 border-amber-500 pl-2' : ''">
              <Input
                type="number"
                :model-value="finance.loan"
                @input="handleLoanInput"
                class="max-w-xs font-mono"
                min="0"
                step="1000"
              />
              <span class="text-sm text-muted-foreground">
                {{ formatMoney(finance.loan) }} $
              </span>
              <TooltipProvider v-if="isLoanModified">
                <Tooltip>
                  <TooltipTrigger as-child>
                    <span class="size-2 rounded-full bg-amber-500" />
                  </TooltipTrigger>
                  <TooltipContent>
                    {{ t("dirtyTracking.originalValue", { value: formatMoney(finance.originalLoan) + " $" }) }}
                  </TooltipContent>
                </Tooltip>
              </TooltipProvider>
            </div>
            <AlertDialog v-model:open="showRepayDialog">
              <AlertDialogTrigger as-child>
                <Button
                  variant="outline"
                  size="sm"
                  :disabled="finance.loan === 0"
                >
                  <RotateCcw class="size-4" />
                  {{ t("finance.repayLoan") }}
                </Button>
              </AlertDialogTrigger>
              <AlertDialogContent>
                <AlertDialogHeader>
                  <AlertDialogTitle>{{ t("finance.repayLoan") }}</AlertDialogTitle>
                  <AlertDialogDescription>
                    {{ t("finance.repayLoanDesc", { amount: formatMoney(finance.loan) }) }}
                  </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                  <AlertDialogCancel>{{ t("common.cancel") }}</AlertDialogCancel>
                  <AlertDialogAction
                    @click="finance.repayLoan(false)"
                  >
                    {{ t("finance.clearOnly") }}
                  </AlertDialogAction>
                  <AlertDialogAction
                    @click="finance.repayLoan(true)"
                  >
                    {{ t("finance.deductFromMoney") }}
                  </AlertDialogAction>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
          </CardContent>
        </Card>

        <!-- Dirty indicator -->
        <div v-if="finance.isDirty" class="flex items-center gap-3">
          <span class="text-sm text-amber-600">{{ t("common.unsavedChanges") }}</span>
          <Button variant="ghost" size="sm" @click="finance.resetChanges()">
            {{ t("common.cancelChanges") }}
          </Button>
        </div>
      </TabsContent>

      <!-- Statistics tab (advanced) -->
      <TabsContent v-if="settings.advancedMode" value="stats" class="space-y-4">
        <!-- Revenue/Expenses summary from daily finances -->
        <div v-if="finance.dailyFinances.length > 0" class="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <Card>
            <CardContent class="flex items-center justify-between pt-6">
              <span class="text-sm font-medium">{{ t("finance.stats.totalRevenue") }}</span>
              <span class="font-mono text-lg text-green-600 dark:text-green-400">
                +{{ formatMoney(totalRevenue) }} $
              </span>
            </CardContent>
          </Card>
          <Card>
            <CardContent class="flex items-center justify-between pt-6">
              <span class="text-sm font-medium">{{ t("finance.stats.totalExpenses") }}</span>
              <span class="font-mono text-lg text-red-600 dark:text-red-400">
                {{ formatMoney(totalExpenses) }} $
              </span>
            </CardContent>
          </Card>
        </div>

        <!-- Stats by group -->
        <template v-if="finance.statistics">
          <Card v-for="group in statGroups" :key="group.key">
            <CardHeader class="pb-3">
              <CardTitle class="text-base">{{ t(`finance.statGroups.${group.key}`) }}</CardTitle>
            </CardHeader>
            <CardContent>
              <div class="grid grid-cols-1 gap-2 sm:grid-cols-2">
                <div
                  v-for="key in group.fields"
                  :key="key"
                  class="flex items-center justify-between rounded-md border px-3 py-1.5"
                >
                  <span class="text-sm text-muted-foreground">{{ t(`finance.stats.${key}`) }}</span>
                  <span class="font-mono text-sm">
                    {{
                      formatStatValue(
                        key,
                        (finance.statistics as Record<string, number>)[key] ?? 0,
                      )
                    }}
                  </span>
                </div>
              </div>
            </CardContent>
          </Card>
        </template>
        <Card v-else>
          <CardContent class="pt-6">
            <p class="text-sm text-muted-foreground">
              {{ t("finance.noStats") }}
            </p>
          </CardContent>
        </Card>
      </TabsContent>

      <!-- Daily finances tab (advanced) -->
      <TabsContent v-if="settings.advancedMode" value="history">
        <div v-if="finance.dailyFinances.length > 0" class="space-y-4">
          <Card
            v-for="df in sortedDailyFinances"
            :key="df.day"
          >
            <CardHeader class="pb-3">
              <CardTitle class="flex items-center justify-between text-base">
                <span>{{ formatDayLabel(df.day) }}</span>
                <span
                  class="font-mono text-sm"
                  :class="getDayTotal(df) >= 0 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'"
                >
                  {{ getDayTotal(df) >= 0 ? '+' : '' }}{{ formatMoney(getDayTotal(df)) }} $
                </span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div class="grid grid-cols-1 gap-2 sm:grid-cols-2">
                <div
                  v-for="entry in getNonZeroEntries(df)"
                  :key="entry.key"
                  class="flex items-center justify-between rounded-md border px-3 py-1.5"
                >
                  <span class="text-sm text-muted-foreground">{{ t(`finance.categories.${entry.key}`) }}</span>
                  <span
                    class="font-mono text-sm"
                    :class="entry.value >= 0 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'"
                  >
                    {{ entry.value >= 0 ? '+' : '' }}{{ formatMoney(entry.value) }} $
                  </span>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
        <Card v-else>
          <CardContent class="pt-6">
            <p class="text-sm text-muted-foreground">
              {{ t("finance.noHistory") }}
            </p>
          </CardContent>
        </Card>
      </TabsContent>
    </Tabs>
  </div>
</template>
