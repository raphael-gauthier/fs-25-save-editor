<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useSaleStore } from "@/stores/sale";
import { useSettingsStore } from "@/stores/settings";
import { formatMoney } from "@/lib/utils";
import type { SaleItem } from "@/lib/types";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Separator } from "@/components/ui/separator";
import { Badge } from "@/components/ui/badge";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { RotateCcw, Tag, CalendarPlus, Trash2 } from "lucide-vue-next";

interface Props {
  item: SaleItem | null;
  open: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  "update:open": [value: boolean];
}>();

const { t } = useI18n();
const store = useSaleStore();
const settings = useSettingsStore();
const showDeleteDialog = ref(false);

const original = computed(() => props.item ? store.getOriginalItem(props.item.index) : undefined);

function saleFieldModified(field: keyof SaleItem): boolean {
  if (!props.item || !original.value) return false;
  const curr = props.item[field];
  const orig = original.value[field];
  if (typeof curr === "number" && typeof orig === "number") {
    return Math.abs(curr - orig) > 0.0001;
  }
  return curr !== orig;
}

function saleModifiedClass(field: keyof SaleItem): string {
  return saleFieldModified(field) ? "border-l-2 border-amber-500 pl-2" : "";
}

function handlePriceInput(event: Event) {
  const value = parseInt((event.target as HTMLInputElement).value);
  if (!isNaN(value) && props.item) {
    store.updateItem(props.item.index, { price: Math.max(0, value) });
  }
}

function handleWearChange(value: number[] | undefined) {
  if (value && props.item) {
    store.updateItem(props.item.index, { wear: value[0] / 100 });
  }
}

function handleDamageChange(value: number[] | undefined) {
  if (value && props.item) {
    store.updateItem(props.item.index, { damage: value[0] / 100 });
  }
}

function handleTimeLeftInput(event: Event) {
  const value = parseInt((event.target as HTMLInputElement).value);
  if (!isNaN(value) && props.item) {
    store.updateItem(props.item.index, { timeLeft: Math.max(0, value) });
  }
}

function handleAgeInput(event: Event) {
  const value = parseInt((event.target as HTMLInputElement).value);
  if (!isNaN(value) && props.item) {
    store.updateItem(props.item.index, { age: Math.max(0, value) });
  }
}

function handleOperatingTimeInput(event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  if (!isNaN(value) && props.item) {
    store.updateItem(props.item.index, { operatingTime: Math.max(0, value) });
  }
}

function handleResetToNew() {
  if (props.item) store.resetToNew(props.item.index);
}

function handleDiscountPrice() {
  if (props.item) store.setDiscountPrice(props.item.index);
}

function handleExtendSale() {
  if (props.item) store.extendSale(props.item.index, 30);
}

function handleDelete() {
  if (props.item) {
    store.deleteItem(props.item.index);
    showDeleteDialog.value = false;
    emit("update:open", false);
  }
}
</script>

<template>
  <Sheet :open="open" @update:open="emit('update:open', $event)">
    <SheetContent class="overflow-y-auto sm:max-w-md">
      <template v-if="item">
        <SheetHeader>
          <SheetTitle>{{ item.displayName }}</SheetTitle>
          <SheetDescription class="flex items-center gap-2">
            <Badge v-if="item.isGenerated" variant="secondary">{{ t("sale.generated") }}</Badge>
            <Badge v-else variant="outline">{{ t("sale.player") }}</Badge>
          </SheetDescription>
        </SheetHeader>

        <div class="mt-6 space-y-6">
          <!-- Price -->
          <div class="space-y-2" :class="saleModifiedClass('price')">
            <Label>{{ t("sale.price") }}</Label>
            <div class="flex items-center gap-2">
              <Input
                type="number"
                :model-value="item.price"
                @input="handlePriceInput"
                class="font-mono"
                min="0"
                step="1000"
              />
              <span class="shrink-0 text-sm text-muted-foreground">
                {{ formatMoney(item.price) }} $
              </span>
            </div>
          </div>

          <!-- Wear -->
          <div class="space-y-2" :class="saleModifiedClass('wear')">
            <div class="flex items-center justify-between text-sm">
              <Label>{{ t("sale.wear") }}</Label>
              <span class="font-mono text-muted-foreground">
                {{ Math.round(item.wear * 100) }}%
              </span>
            </div>
            <Slider
              :model-value="[Math.round(item.wear * 100)]"
              :max="100"
              :min="0"
              :step="1"
              @update:model-value="handleWearChange"
            />
          </div>

          <!-- Damage -->
          <div class="space-y-2" :class="saleModifiedClass('damage')">
            <div class="flex items-center justify-between text-sm">
              <Label>{{ t("sale.damage") }}</Label>
              <span class="font-mono text-muted-foreground">
                {{ Math.round(item.damage * 100) }}%
              </span>
            </div>
            <Slider
              :model-value="[Math.round(item.damage * 100)]"
              :max="100"
              :min="0"
              :step="1"
              @update:model-value="handleDamageChange"
            />
          </div>

          <!-- Days left -->
          <div class="space-y-2" :class="saleModifiedClass('timeLeft')">
            <Label>{{ t("sale.daysLeft") }}</Label>
            <Input
              type="number"
              :model-value="item.timeLeft"
              @input="handleTimeLeftInput"
              class="font-mono"
              min="0"
              step="1"
            />
          </div>

          <!-- Advanced mode -->
          <template v-if="settings.advancedMode">
            <Separator />
            <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
              {{ t("sale.advancedMode") }}
            </p>

            <div class="space-y-4">
              <div class="space-y-2" :class="saleModifiedClass('age')">
                <Label>{{ t("sale.ageDays") }}</Label>
                <Input
                  type="number"
                  :model-value="item.age"
                  @input="handleAgeInput"
                  class="font-mono"
                  min="0"
                  step="1"
                />
              </div>

              <div class="space-y-2" :class="saleModifiedClass('operatingTime')">
                <Label>{{ t("sale.operatingTime") }}</Label>
                <Input
                  type="number"
                  :model-value="item.operatingTime"
                  @input="handleOperatingTimeInput"
                  class="font-mono"
                  min="0"
                  step="0.1"
                />
              </div>

              <div v-if="item.boughtConfigurations.length > 0" class="space-y-2">
                <Label>{{ t("sale.configuration") }}</Label>
                <div class="flex flex-wrap gap-1">
                  <Badge
                    v-for="config in item.boughtConfigurations"
                    :key="config.name + config.id"
                    variant="outline"
                  >
                    {{ config.name }} #{{ config.id }}
                  </Badge>
                </div>
              </div>
            </div>
          </template>

          <!-- Quick actions -->
          <Separator />
          <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            {{ t("sale.quickActions") }}
          </p>

          <div class="flex flex-col gap-2">
            <Button variant="outline" size="sm" @click="handleResetToNew">
              <RotateCcw class="size-4" />
              {{ t("sale.resetToNew") }}
            </Button>
            <Button variant="outline" size="sm" @click="handleDiscountPrice">
              <Tag class="size-4" />
              {{ t("sale.discountPrice") }}
            </Button>
            <Button variant="outline" size="sm" @click="handleExtendSale">
              <CalendarPlus class="size-4" />
              {{ t("sale.extendSale") }}
            </Button>
          </div>

          <!-- Remove from market -->
          <Separator />
          <AlertDialog v-model:open="showDeleteDialog">
            <Button variant="destructive" class="w-full" @click="showDeleteDialog = true">
              <Trash2 class="size-4" />
              {{ t("sale.removeFromMarket") }}
            </Button>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>{{ t("sale.removeTitle") }}</AlertDialogTitle>
                <AlertDialogDescription>
                  {{ t("sale.removeDesc", { name: item.displayName }) }}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>{{ t("common.cancel") }}</AlertDialogCancel>
                <AlertDialogAction @click="handleDelete">{{ t("sale.removeAction") }}</AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </div>
      </template>
    </SheetContent>
  </Sheet>
</template>
