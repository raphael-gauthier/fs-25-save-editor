<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useBuildingStore } from "@/stores/building";
import { useSettingsStore } from "@/stores/settings";
import type { Placeable } from "@/lib/types";
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
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
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
import { Building2, Hammer } from "lucide-vue-next";

interface Props {
  placeable: Placeable | null;
  open: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  "update:open": [value: boolean];
}>();

const { t } = useI18n();
const store = useBuildingStore();
const settings = useSettingsStore();

const original = computed(() =>
  props.placeable ? store.getOriginalByIndex(props.placeable.index) : undefined,
);

function fieldModifiedClass(key: keyof Placeable): string {
  if (!props.placeable || !original.value) return "";
  return props.placeable[key] !== original.value[key]
    ? "border-l-2 border-amber-500 pl-2"
    : "";
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleOwnerChange(value: any) {
  if (props.placeable) {
    store.updatePlaceable(props.placeable.index, { farmId: Number(value) });
  }
}

function handlePriceChange(event: Event) {
  const target = event.target as HTMLInputElement;
  const value = parseFloat(target.value);
  if (props.placeable && !isNaN(value)) {
    store.updatePlaceable(props.placeable.index, { price: value });
  }
}

function handleCompleteConstruction() {
  if (props.placeable) {
    store.completeConstruction(props.placeable.index);
  }
}

function handleInputStockChange(fillType: string, value: number[] | undefined) {
  if (value && props.placeable) {
    store.updateProductionStock(props.placeable.index, "input", fillType, value[0]);
  }
}

function handleOutputStockChange(fillType: string, value: number[] | undefined) {
  if (value && props.placeable) {
    store.updateProductionStock(props.placeable.index, "output", fillType, value[0]);
  }
}
</script>

<template>
  <Sheet :open="open" @update:open="emit('update:open', $event)">
    <SheetContent class="overflow-y-auto sm:max-w-md">
      <template v-if="placeable">
        <SheetHeader>
          <SheetTitle class="flex items-center gap-2">
            <Building2 class="size-5" />
            {{ placeable.displayName }}
          </SheetTitle>
          <SheetDescription>
            {{ placeable.filename }}
          </SheetDescription>
        </SheetHeader>

        <div class="mt-6 space-y-6">
          <!-- Owner -->
          <div class="space-y-2" :class="fieldModifiedClass('farmId')">
            <Label>{{ t("building.owner") }}</Label>
            <Select :model-value="String(placeable.farmId)" @update:model-value="handleOwnerChange">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="0">{{ t("field.free") }}</SelectItem>
                <SelectItem value="1">{{ t("field.myFarm") }}</SelectItem>
                <SelectItem v-for="i in [2, 3, 4, 5, 6]" :key="i" :value="String(i)">
                  {{ t("field.farm", { id: i }) }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <!-- Price -->
          <div class="space-y-2" :class="fieldModifiedClass('price')">
            <Label>{{ t("building.price") }}</Label>
            <Input
              type="number"
              :model-value="placeable.price"
              @change="handlePriceChange"
            />
          </div>

          <!-- Construction section -->
          <template v-if="placeable.constructionSteps.length > 0">
            <Separator />
            <div class="space-y-4">
              <div class="flex items-center justify-between">
                <p class="text-sm font-medium">
                  <Hammer class="mr-1 inline size-4" />
                  {{ placeable.isUnderConstruction ? t("building.underConstruction") : t("building.ready") }}
                </p>
                <Badge v-if="placeable.isUnderConstruction" variant="secondary">
                  {{ t("building.underConstruction") }}
                </Badge>
                <Badge v-else variant="default">
                  {{ t("building.ready") }}
                </Badge>
              </div>

              <!-- Construction steps detail -->
              <div v-for="step in placeable.constructionSteps" :key="step.stepIndex" class="space-y-2">
                <p class="text-xs font-medium text-muted-foreground">
                  {{ t("building.constructionStep", { index: step.stepIndex + 1 }) }}
                </p>
                <div
                  v-for="mat in step.materials"
                  :key="mat.fillType"
                  class="flex items-center justify-between text-sm"
                >
                  <span>{{ mat.fillType }}</span>
                  <span class="font-mono text-muted-foreground">
                    {{ t("building.materialRemaining", { remaining: Math.round(mat.amountRemaining), total: Math.round(mat.amountTotal) }) }}
                  </span>
                </div>
              </div>

              <!-- Complete construction button -->
              <AlertDialog v-if="placeable.isUnderConstruction">
                <AlertDialogTrigger as-child>
                  <Button variant="default" class="w-full">
                    <Hammer class="size-4" />
                    {{ t("building.completeConstruction") }}
                  </Button>
                </AlertDialogTrigger>
                <AlertDialogContent>
                  <AlertDialogHeader>
                    <AlertDialogTitle>{{ t("building.completeConstruction") }}</AlertDialogTitle>
                    <AlertDialogDescription>
                      {{ t("building.completeConstructionDesc") }}
                    </AlertDialogDescription>
                  </AlertDialogHeader>
                  <AlertDialogFooter>
                    <AlertDialogCancel>{{ t("common.cancel") }}</AlertDialogCancel>
                    <AlertDialogAction @click="handleCompleteConstruction">
                      {{ t("common.confirm") }}
                    </AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
            </div>
          </template>

          <!-- Advanced mode: production stocks -->
          <template v-if="settings.advancedMode">
            <!-- Production inputs -->
            <template v-if="placeable.productionInputs.length > 0">
              <Separator />
              <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                {{ t("building.productionInputs") }}
              </p>
              <div class="space-y-4">
                <div v-for="stock in placeable.productionInputs" :key="stock.fillType" class="space-y-2">
                  <div class="flex items-center justify-between text-sm">
                    <Label>{{ stock.fillType }}</Label>
                    <span class="font-mono text-muted-foreground">
                      {{ Math.round(stock.amount) }} / {{ Math.round(stock.capacity) }}
                    </span>
                  </div>
                  <Slider
                    :model-value="[stock.amount]"
                    :max="stock.capacity"
                    :min="0"
                    :step="1"
                    @update:model-value="handleInputStockChange(stock.fillType, $event)"
                  />
                </div>
              </div>
            </template>

            <!-- Production outputs -->
            <template v-if="placeable.productionOutputs.length > 0">
              <Separator />
              <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                {{ t("building.productionOutputs") }}
              </p>
              <div class="space-y-4">
                <div v-for="stock in placeable.productionOutputs" :key="stock.fillType" class="space-y-2">
                  <div class="flex items-center justify-between text-sm">
                    <Label>{{ stock.fillType }}</Label>
                    <span class="font-mono text-muted-foreground">
                      {{ Math.round(stock.amount) }} / {{ Math.round(stock.capacity) }}
                    </span>
                  </div>
                  <Slider
                    :model-value="[stock.amount]"
                    :max="stock.capacity"
                    :min="0"
                    :step="1"
                    @update:model-value="handleOutputStockChange(stock.fillType, $event)"
                  />
                </div>
              </div>
            </template>

            <!-- Position (read-only) -->
            <template v-if="placeable.position">
              <Separator />
              <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                {{ t("building.position") }}
              </p>
              <div class="grid grid-cols-3 gap-2 text-sm">
                <div>
                  <Label class="text-muted-foreground">X</Label>
                  <p class="font-mono">{{ placeable.position.x.toFixed(1) }}</p>
                </div>
                <div>
                  <Label class="text-muted-foreground">Y</Label>
                  <p class="font-mono">{{ placeable.position.y.toFixed(1) }}</p>
                </div>
                <div>
                  <Label class="text-muted-foreground">Z</Label>
                  <p class="font-mono">{{ placeable.position.z.toFixed(1) }}</p>
                </div>
              </div>
            </template>

            <!-- Age (read-only) -->
            <div class="space-y-2">
              <Label class="text-muted-foreground">{{ t("building.age") }}</Label>
              <p class="font-mono text-sm">{{ Math.round(placeable.age) }}</p>
            </div>
          </template>
        </div>
      </template>
    </SheetContent>
  </Sheet>
</template>
