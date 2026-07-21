<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useAnimalStore } from "@/stores/animal";
import HusbandryCard from "@/components/animals/HusbandryCard.vue";
import { PiggyBank } from "@lucide/vue";

const { t } = useI18n();
const store = useAnimalStore();

const sorted = computed(() =>
  [...store.husbandries].sort((a, b) => a.placeableIndex - b.placeableIndex),
);
</script>

<template>
  <div class="space-y-6">
    <div>
      <h2 class="text-2xl font-semibold">{{ t("animal.title") }}</h2>
      <p class="text-sm text-muted-foreground">
        {{ t("animal.subtitle", { count: store.totalAnimals }) }}
      </p>
    </div>

    <div v-if="sorted.length === 0" class="py-12 text-center">
      <PiggyBank class="mx-auto mb-3 size-10 text-muted-foreground" />
      <p class="text-muted-foreground">{{ t("animal.none") }}</p>
    </div>

    <div v-else class="grid gap-4 lg:grid-cols-2">
      <HusbandryCard
        v-for="husbandry in sorted"
        :key="husbandry.placeableIndex"
        :husbandry="husbandry"
      />
    </div>
  </div>
</template>
