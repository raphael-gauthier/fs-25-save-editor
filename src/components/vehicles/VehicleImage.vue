<script setup lang="ts">
import { computed, ref } from "vue";
import { useVehicleImages } from "@/composables/useVehicleImages";
import { Truck, Loader2 } from "lucide-vue-next";

const props = withDefaults(
  defineProps<{
    filename: string;
    size?: "sm" | "md" | "lg";
  }>(),
  { size: "sm" },
);

const { imageCache, pendingBatch, getImageUrl } = useVehicleImages();
const hasError = ref(false);

const imageUrl = computed(() => {
  // Access imageCache.value to register reactive dependency
  imageCache.value;
  return getImageUrl(props.filename);
});

const loading = computed(() => {
  // Show skeleton if image not yet in cache and a batch is in progress
  imageCache.value;
  return !imageCache.value.has(props.filename) && pendingBatch.value;
});

const sizeClass = computed(() => {
  switch (props.size) {
    case "sm":
      return "size-10";
    case "md":
      return "size-16";
    case "lg":
      return "size-32";
  }
});

const iconSize = computed(() => {
  switch (props.size) {
    case "sm":
      return "size-5";
    case "md":
      return "size-8";
    case "lg":
      return "size-16";
  }
});

function handleError() {
  hasError.value = true;
}
</script>

<template>
  <div
    :class="[
      sizeClass,
      'shrink-0 flex items-center justify-center rounded-md bg-muted overflow-hidden',
    ]"
  >
    <img
      v-if="imageUrl && !hasError"
      :src="imageUrl"
      :alt="filename"
      class="size-full object-contain"
      @error="handleError"
    />
    <Loader2 v-else-if="loading" :class="['text-muted-foreground animate-spin', iconSize]" />
    <Truck v-else :class="['text-muted-foreground', iconSize]" />
  </div>
</template>
