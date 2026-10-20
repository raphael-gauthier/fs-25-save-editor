<script setup lang="ts">
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

/** Shows `label` in a tooltip over the slotted control (an icon button, mostly). */
defineProps<{
  label: string;
  side?: "top" | "right" | "bottom" | "left";
  disabled?: boolean;
}>();
</script>

<template>
  <Tooltip :disabled="disabled">
    <TooltipTrigger as-child>
      <slot />
    </TooltipTrigger>
    <TooltipContent :side="side ?? 'bottom'">{{ label }}</TooltipContent>
  </Tooltip>
</template>
