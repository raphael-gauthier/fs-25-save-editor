<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { Slider } from "@/components/ui/slider";
import { formatMoney } from "@/lib/utils";

const { t } = useI18n();

interface Props {
  fillType: string;
  fillLevel: number;
  capacity: number | null;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  "update:fillLevel": [value: number];
}>();

const label = computed(() => {
  const key = `fillTypes.${props.fillType}`;
  const translated = t(key);
  return translated === key ? props.fillType : translated;
});
const maxValue = computed(() => props.capacity ?? props.fillLevel);
const percentage = computed(() => {
  if (maxValue.value <= 0) return 0;
  return Math.round((props.fillLevel / maxValue.value) * 100);
});

function handleSliderChange(value: number[] | undefined) {
  if (value) emit("update:fillLevel", value[0]);
}
</script>

<template>
  <div class="space-y-2">
    <div class="flex items-center justify-between text-sm">
      <span class="font-medium">{{ label }}</span>
      <span class="font-mono text-muted-foreground">
        {{ formatMoney(fillLevel) }} / {{ capacity !== null ? formatMoney(capacity) : "?" }} L
        <span class="ml-1 text-xs">({{ percentage }}%)</span>
      </span>
    </div>
    <Slider
      :model-value="[fillLevel]"
      :max="maxValue"
      :min="0"
      :step="1"
      @update:model-value="handleSliderChange"
    />
  </div>
</template>
