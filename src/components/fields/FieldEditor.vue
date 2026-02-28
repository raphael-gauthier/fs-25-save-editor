<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useFieldStore } from "@/stores/field";
import { useSettingsStore } from "@/stores/settings";
import type { Field } from "@/lib/types";
import { FRUIT_TYPES, GROUND_TYPES, MAX_GROWTH_STATE } from "@/lib/constants";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Separator } from "@/components/ui/separator";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Sprout, Sparkles } from "lucide-vue-next";

interface Props {
  field: Field | null;
  open: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  "update:open": [value: boolean];
}>();

const { t } = useI18n();
const store = useFieldStore();
const settings = useSettingsStore();

const original = computed(() =>
  props.field ? store.getOriginalFieldById(props.field.id) : undefined,
);

const farmland = computed(() =>
  props.field ? store.getFarmlandByFieldId(props.field.id) : undefined,
);

function fieldModifiedClass(key: keyof Field): string {
  if (!props.field || !original.value) return "";
  return props.field[key] !== original.value[key]
    ? "border-l-2 border-amber-500 pl-2"
    : "";
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleFruitTypeChange(value: any) {
  if (props.field) {
    store.updateField(props.field.id, { fruitType: String(value) });
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handlePlannedFruitChange(value: any) {
  if (props.field) {
    store.updateField(props.field.id, { plannedFruit: String(value) });
  }
}

function handleGrowthChange(value: number[] | undefined) {
  if (value && props.field) {
    store.updateField(props.field.id, { growthState: value[0] });
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleGroundTypeChange(value: any) {
  if (props.field) {
    store.updateField(props.field.id, { groundType: String(value) });
  }
}

function handleMaxGrowth() {
  if (props.field) {
    store.updateField(props.field.id, { growthState: MAX_GROWTH_STATE });
  }
}

function handleSliderChange(key: keyof Field, value: number[] | undefined) {
  if (value && props.field) {
    store.updateField(props.field.id, { [key]: value[0] } as Partial<Field>);
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleOwnerChange(value: any) {
  if (props.field) {
    store.updateFarmland(props.field.id, Number(value));
  }
}
</script>

<template>
  <Sheet :open="open" @update:open="emit('update:open', $event)">
    <SheetContent class="overflow-y-auto sm:max-w-md">
      <template v-if="field">
        <SheetHeader>
          <SheetTitle class="flex items-center gap-2">
            <Sprout class="size-5" />
            {{ t("field.title") }} #{{ field.id }}
          </SheetTitle>
          <SheetDescription>
            {{ field.fruitType !== "UNKNOWN" ? t(`fillTypes.${field.fruitType}`, field.fruitType) : t("field.none") }}
          </SheetDescription>
        </SheetHeader>

        <div class="mt-6 space-y-6">
          <!-- Fruit type -->
          <div class="space-y-2" :class="fieldModifiedClass('fruitType')">
            <Label>{{ t("field.fruitType") }}</Label>
            <Select :model-value="field.fruitType" @update:model-value="handleFruitTypeChange">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="fruit in FRUIT_TYPES" :key="fruit" :value="fruit">
                  {{ t(`fillTypes.${fruit}`, fruit) }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <!-- Planned fruit -->
          <div class="space-y-2" :class="fieldModifiedClass('plannedFruit')">
            <Label>{{ t("field.plannedFruit") }}</Label>
            <Select :model-value="field.plannedFruit" @update:model-value="handlePlannedFruitChange">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="fruit in FRUIT_TYPES" :key="fruit" :value="fruit">
                  {{ t(`fillTypes.${fruit}`, fruit) }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <!-- Growth state -->
          <div class="space-y-2" :class="fieldModifiedClass('growthState')">
            <div class="flex items-center justify-between text-sm">
              <Label>{{ t("field.growthState") }}</Label>
              <span class="font-mono text-muted-foreground">
                {{ field.growthState }} / {{ MAX_GROWTH_STATE }}
              </span>
            </div>
            <Slider
              :model-value="[field.growthState]"
              :max="MAX_GROWTH_STATE"
              :min="0"
              :step="1"
              @update:model-value="handleGrowthChange"
            />
            <Button variant="outline" size="sm" @click="handleMaxGrowth">
              <Sparkles class="size-4" />
              {{ t("field.maxGrowth") }}
            </Button>
          </div>

          <!-- Ground type -->
          <div class="space-y-2" :class="fieldModifiedClass('groundType')">
            <Label>{{ t("field.groundType") }}</Label>
            <Select :model-value="field.groundType" @update:model-value="handleGroundTypeChange">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="gt in GROUND_TYPES" :key="gt" :value="gt">
                  {{ t(`groundTypes.${gt}`, gt) }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <!-- Farmland owner -->
          <div v-if="farmland" class="space-y-2">
            <Label>{{ t("field.farmlandOwner") }}</Label>
            <Select :model-value="String(farmland.farmId)" @update:model-value="handleOwnerChange">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="0">{{ t("field.free") }}</SelectItem>
                <SelectItem value="1">{{ t("field.myFarm") }}</SelectItem>
                <SelectItem v-if="settings.advancedMode" v-for="i in [2,3,4,5,6]" :key="i" :value="String(i)">
                  {{ t("field.farm", { id: i }) }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <!-- Advanced: Treatment levels -->
          <template v-if="settings.advancedMode">
            <Separator />
            <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
              {{ t("field.treatments") }}
            </p>

            <div class="space-y-4">
              <!-- Weed state -->
              <div class="space-y-2" :class="fieldModifiedClass('weedState')">
                <div class="flex items-center justify-between text-sm">
                  <Label>{{ t("field.weedState") }}</Label>
                  <span class="font-mono text-muted-foreground">{{ field.weedState }}</span>
                </div>
                <Slider
                  :model-value="[field.weedState]"
                  :max="10"
                  :min="0"
                  :step="1"
                  @update:model-value="handleSliderChange('weedState', $event)"
                />
              </div>

              <!-- Stone level -->
              <div class="space-y-2" :class="fieldModifiedClass('stoneLevel')">
                <div class="flex items-center justify-between text-sm">
                  <Label>{{ t("field.stoneLevel") }}</Label>
                  <span class="font-mono text-muted-foreground">{{ field.stoneLevel }}</span>
                </div>
                <Slider
                  :model-value="[field.stoneLevel]"
                  :max="5"
                  :min="0"
                  :step="1"
                  @update:model-value="handleSliderChange('stoneLevel', $event)"
                />
              </div>

              <!-- Spray level -->
              <div class="space-y-2" :class="fieldModifiedClass('sprayLevel')">
                <div class="flex items-center justify-between text-sm">
                  <Label>{{ t("field.sprayLevel") }}</Label>
                  <span class="font-mono text-muted-foreground">{{ field.sprayLevel }}</span>
                </div>
                <Slider
                  :model-value="[field.sprayLevel]"
                  :max="3"
                  :min="0"
                  :step="1"
                  @update:model-value="handleSliderChange('sprayLevel', $event)"
                />
              </div>

              <!-- Lime level -->
              <div class="space-y-2" :class="fieldModifiedClass('limeLevel')">
                <div class="flex items-center justify-between text-sm">
                  <Label>{{ t("field.limeLevel") }}</Label>
                  <span class="font-mono text-muted-foreground">{{ field.limeLevel }}</span>
                </div>
                <Slider
                  :model-value="[field.limeLevel]"
                  :max="3"
                  :min="0"
                  :step="1"
                  @update:model-value="handleSliderChange('limeLevel', $event)"
                />
              </div>

              <!-- Plow level -->
              <div class="space-y-2" :class="fieldModifiedClass('plowLevel')">
                <div class="flex items-center justify-between text-sm">
                  <Label>{{ t("field.plowLevel") }}</Label>
                  <span class="font-mono text-muted-foreground">{{ field.plowLevel }}</span>
                </div>
                <Slider
                  :model-value="[field.plowLevel]"
                  :max="3"
                  :min="0"
                  :step="1"
                  @update:model-value="handleSliderChange('plowLevel', $event)"
                />
              </div>

              <!-- Roller level -->
              <div class="space-y-2" :class="fieldModifiedClass('rollerLevel')">
                <div class="flex items-center justify-between text-sm">
                  <Label>{{ t("field.rollerLevel") }}</Label>
                  <span class="font-mono text-muted-foreground">{{ field.rollerLevel }}</span>
                </div>
                <Slider
                  :model-value="[field.rollerLevel]"
                  :max="3"
                  :min="0"
                  :step="1"
                  @update:model-value="handleSliderChange('rollerLevel', $event)"
                />
              </div>

              <!-- Stubble shred level -->
              <div class="space-y-2" :class="fieldModifiedClass('stubbleShredLevel')">
                <div class="flex items-center justify-between text-sm">
                  <Label>{{ t("field.stubbleShredLevel") }}</Label>
                  <span class="font-mono text-muted-foreground">{{ field.stubbleShredLevel }}</span>
                </div>
                <Slider
                  :model-value="[field.stubbleShredLevel]"
                  :max="3"
                  :min="0"
                  :step="1"
                  @update:model-value="handleSliderChange('stubbleShredLevel', $event)"
                />
              </div>

              <!-- Water level -->
              <div class="space-y-2" :class="fieldModifiedClass('waterLevel')">
                <div class="flex items-center justify-between text-sm">
                  <Label>{{ t("field.waterLevel") }}</Label>
                  <span class="font-mono text-muted-foreground">{{ field.waterLevel }}</span>
                </div>
                <Slider
                  :model-value="[field.waterLevel]"
                  :max="3"
                  :min="0"
                  :step="1"
                  @update:model-value="handleSliderChange('waterLevel', $event)"
                />
              </div>
            </div>
          </template>
        </div>
      </template>
    </SheetContent>
  </Sheet>
</template>
