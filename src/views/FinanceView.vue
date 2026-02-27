<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useFinanceStore } from "@/stores/finance";
import { useSettingsStore } from "@/stores/settings";
import { formatMoney } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
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

const statKeys = [
  "traveledDistance", "fuelUsage", "seedUsage", "sprayUsage",
  "workedHectares", "cultivatedHectares", "sownHectares", "sprayedHectares",
  "threshedHectares", "plowedHectares", "baleCount", "missionCount",
  "playTime", "revenue", "expenses",
] as const;

const financeKeys = [
  "newVehiclesCost", "soldVehicles", "newAnimalsCost", "soldAnimals",
  "constructionCost", "soldBuildings", "fieldPurchase", "soldFields",
  "vehicleRunningCost", "vehicleLeasingCost", "propertyMaintenance", "propertyIncome",
  "productionCosts", "soldProducts", "harvestIncome", "missionIncome",
  "wagePayment", "loanInterest", "otherIncome", "otherExpenses",
] as const;

function formatStatValue(key: string, value: number): string {
  if (key === "playTime") return `${(value / 60).toFixed(1)}h`;
  if (key === "revenue" || key === "expenses") return `${formatMoney(value)} $`;
  if (key === "baleCount" || key === "missionCount") return value.toString();
  return value.toFixed(1);
}
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
      <TabsContent v-if="settings.advancedMode" value="stats">
        <Card>
          <CardHeader>
            <CardTitle class="text-base">{{ t("finance.farmStats") }}</CardTitle>
          </CardHeader>
          <CardContent>
            <div
              v-if="finance.statistics"
              class="grid grid-cols-1 gap-3 sm:grid-cols-2"
            >
              <div
                v-for="key in statKeys"
                :key="key"
                class="flex items-center justify-between rounded-md border px-3 py-2"
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
            <p v-else class="text-sm text-muted-foreground">
              {{ t("finance.noStats") }}
            </p>
          </CardContent>
        </Card>
      </TabsContent>

      <!-- Daily finances tab (advanced) -->
      <TabsContent v-if="settings.advancedMode" value="history">
        <Card>
          <CardHeader>
            <CardTitle class="text-base">{{ t("finance.dailyHistory") }}</CardTitle>
          </CardHeader>
          <CardContent>
            <div v-if="finance.dailyFinances.length > 0" class="overflow-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead class="w-16">{{ t("finance.day") }}</TableHead>
                    <TableHead
                      v-for="key in financeKeys"
                      :key="key"
                      class="min-w-24 text-right"
                    >
                      {{ t(`finance.categories.${key}`) }}
                    </TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow
                    v-for="df in finance.dailyFinances"
                    :key="df.day"
                  >
                    <TableCell class="font-mono">{{ df.day }}</TableCell>
                    <TableCell
                      v-for="key in financeKeys"
                      :key="key"
                      class="text-right font-mono"
                    >
                      {{
                        formatMoney(
                          (df as unknown as Record<string, number>)[key] ?? 0,
                        )
                      }}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </div>
            <p v-else class="text-sm text-muted-foreground">
              {{ t("finance.noHistory") }}
            </p>
          </CardContent>
        </Card>
      </TabsContent>
    </Tabs>
  </div>
</template>
