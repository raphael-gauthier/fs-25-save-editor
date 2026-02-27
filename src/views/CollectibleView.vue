<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { useMissionStore } from "@/stores/mission";
import { useSettingsStore } from "@/stores/settings";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Separator } from "@/components/ui/separator";
import { Trophy } from "lucide-vue-next";

const { t } = useI18n();
const store = useMissionStore();
const settings = useSettingsStore();
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-2xl font-semibold">{{ t("collectible.title") }}</h2>
        <p class="text-sm text-muted-foreground">{{ t("collectible.subtitle") }}</p>
      </div>
      <div class="flex items-center gap-2">
        <Label for="advanced-mode" class="text-sm">{{ t("common.advancedMode") }}</Label>
        <Switch
          id="advanced-mode"
          :model-value="settings.advancedMode"
          @update:model-value="settings.setAdvancedMode($event)"
        />
      </div>
    </div>

    <!-- Collectibles section -->
    <div v-if="store.collectibles.length > 0" class="space-y-4">
      <!-- Counter and actions -->
      <div class="flex flex-wrap items-center gap-3">
        <span class="text-sm font-medium">
          {{ t("collectible.counter", { collected: store.collectedCount, total: store.collectibles.length }) }}
        </span>
        <div class="flex gap-2">
          <Button size="sm" variant="outline" @click="store.collectAll()">
            {{ t("collectible.collectAll") }}
          </Button>
          <Button size="sm" variant="outline" @click="store.resetAllCollectibles()">
            {{ t("collectible.resetAll") }}
          </Button>
        </div>
      </div>

      <!-- Checkbox grid -->
      <div class="grid grid-cols-7 gap-3 sm:grid-cols-10">
        <label
          v-for="c in store.collectibles"
          :key="c.index"
          class="flex cursor-pointer items-center gap-1.5 rounded-md border p-2 transition-colors hover:bg-muted"
          :class="c.collected ? 'border-primary bg-primary/5' : 'border-border'"
        >
          <Checkbox
            :model-value="c.collected"
            @update:model-value="store.toggleCollectible(c.index)"
          />
          <span class="text-sm font-mono">{{ c.index + 1 }}</span>
        </label>
      </div>
    </div>

    <!-- Empty state -->
    <div v-else class="py-12 text-center">
      <Trophy class="mx-auto size-12 text-muted-foreground/50" />
      <p class="mt-4 text-muted-foreground">{{ t("collectible.empty") }}</p>
    </div>

    <!-- Contract settings (advanced mode) -->
    <template v-if="settings.advancedMode && store.contractSettings">
      <Separator />
      <div class="space-y-4">
        <h3 class="text-lg font-semibold">{{ t("collectible.contractSettings") }}</h3>
        <p class="text-sm text-muted-foreground">{{ t("collectible.contractSettingsDesc") }}</p>

        <div class="grid gap-4 sm:grid-cols-3">
          <div class="space-y-2">
            <Label>{{ t("collectible.leaseVehicle") }}</Label>
            <Input
              type="number"
              :model-value="store.contractSettings.leaseVehicle"
              @change="(e: Event) => store.updateContractSettings({ leaseVehicle: parseFloat((e.target as HTMLInputElement).value) })"
            />
          </div>
          <div class="space-y-2">
            <Label>{{ t("collectible.missionPerFarm") }}</Label>
            <Input
              type="number"
              :model-value="store.contractSettings.missionPerFarm"
              @change="(e: Event) => store.updateContractSettings({ missionPerFarm: parseFloat((e.target as HTMLInputElement).value) })"
            />
          </div>
          <div class="space-y-2">
            <Label>{{ t("collectible.allowClearAdd") }}</Label>
            <Input
              type="number"
              :model-value="store.contractSettings.allowClearAdd"
              @change="(e: Event) => store.updateContractSettings({ allowClearAdd: parseFloat((e.target as HTMLInputElement).value) })"
            />
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
