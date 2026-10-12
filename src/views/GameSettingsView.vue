<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { useGameSettingsStore } from "@/stores/gameSettings";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface Choice {
  value: string;
  labelKey: string;
}

type Control =
  | { key: string; type: "bool" }
  | { key: string; type: "choice"; choices: Choice[] }
  | { key: string; type: "number"; min: number; max: number; step: number }
  | { key: string; type: "text" };

const { t } = useI18n();
const store = useGameSettingsStore();

/** Helper slurry/manure sources (Sprayer.lua): 1 none, 2 buy, 3 and up a map station. */
const helperSources: Choice[] = [
  { value: "1", labelKey: "gameSettings.helperSource.none" },
  { value: "2", labelKey: "gameSettings.helperSource.buy" },
];

const sections: { titleKey: string; controls: Control[] }[] = [
  {
    titleKey: "gameSettings.sections.savegame",
    controls: [
      { key: "savegameName", type: "text" },
      {
        key: "economicDifficulty",
        type: "choice",
        choices: ["EASY", "NORMAL", "HARD"].map((d) => ({ value: d, labelKey: `difficulty.${d}` })),
      },
    ],
  },
  {
    titleKey: "gameSettings.sections.driving",
    controls: [
      {
        key: "fuelUsage",
        type: "choice",
        choices: ["1", "2", "3"].map((v) => ({ value: v, labelKey: `gameSettings.fuelUsageLevel.${v}` })),
      },
      { key: "trafficEnabled", type: "bool" },
      { key: "stopAndGoBraking", type: "bool" },
      { key: "automaticMotorStartEnabled", type: "bool" },
      { key: "trailerFillLimit", type: "bool" },
    ],
  },
  {
    titleKey: "gameSettings.sections.farming",
    controls: [
      { key: "plowingRequiredEnabled", type: "bool" },
      { key: "limeRequired", type: "bool" },
      { key: "weedsEnabled", type: "bool" },
      { key: "stonesEnabled", type: "bool" },
      { key: "fruitDestruction", type: "bool" },
      { key: "isSnowEnabled", type: "bool" },
      {
        key: "dirtInterval",
        type: "choice",
        choices: ["1", "2", "3", "4"].map((v) => ({ value: v, labelKey: `gameSettings.dirtLevel.${v}` })),
      },
    ],
  },
  {
    titleKey: "gameSettings.sections.helpers",
    controls: [
      { key: "helperBuyFuel", type: "bool" },
      { key: "helperBuySeeds", type: "bool" },
      { key: "helperBuyFertilizer", type: "bool" },
      { key: "helperSlurrySource", type: "choice", choices: helperSources },
      { key: "helperManureSource", type: "choice", choices: helperSources },
    ],
  },
  {
    titleKey: "gameSettings.sections.time",
    controls: [
      { key: "timeScale", type: "number", min: 0.5, max: 360, step: 0.5 },
      { key: "autoSaveInterval", type: "number", min: 0, max: 120, step: 5 },
    ],
  },
];

function has(key: string): boolean {
  return store.get(key) !== undefined;
}

/** A choice list plus the saved value when it is not one of them (a map's loading station). */
function choicesFor(control: Control & { type: "choice" }): Choice[] {
  const value = store.get(control.key);
  if (value === undefined || control.choices.some((c) => c.value === value)) return control.choices;
  return [...control.choices, { value, labelKey: "gameSettings.helperSource.station" }];
}

function setNumber(control: Control & { type: "number" }, raw: string) {
  const value = Number(raw);
  if (raw.trim() === "" || Number.isNaN(value)) return;
  const clamped = Math.min(control.max, Math.max(control.min, value));
  // Saved with 6 decimals, as the game does
  store.set(control.key, clamped.toFixed(6));
}

function setText(key: string, raw: string) {
  if (raw.trim() !== "") store.set(key, raw.trim());
}
</script>

<template>
  <div class="space-y-6">
    <div>
      <h2 class="text-2xl font-semibold">{{ t("gameSettings.title") }}</h2>
      <p class="text-sm text-muted-foreground">{{ t("gameSettings.subtitle") }}</p>
    </div>

    <div class="grid gap-4 lg:grid-cols-2">
      <Card v-for="section in sections" :key="section.titleKey">
        <CardHeader>
          <CardTitle class="text-base">{{ t(section.titleKey) }}</CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          <template v-for="control in section.controls" :key="control.key">
            <div v-if="has(control.key)" class="flex items-center justify-between gap-4">
              <Label :for="control.key" class="text-sm">{{ t(`gameSettings.keys.${control.key}`) }}</Label>

              <Switch
                v-if="control.type === 'bool'"
                :id="control.key"
                :model-value="store.get(control.key) === 'true'"
                @update:model-value="store.set(control.key, $event ? 'true' : 'false')"
              />

              <Select
                v-else-if="control.type === 'choice'"
                :model-value="store.get(control.key)"
                @update:model-value="store.set(control.key, String($event))"
              >
                <SelectTrigger :id="control.key" class="w-48">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem v-for="choice in choicesFor(control)" :key="choice.value" :value="choice.value">
                    {{ t(choice.labelKey, { value: choice.value }) }}
                  </SelectItem>
                </SelectContent>
              </Select>

              <Input
                v-else-if="control.type === 'number'"
                :id="control.key"
                type="number"
                class="w-32 font-mono"
                :min="control.min"
                :max="control.max"
                :step="control.step"
                :model-value="Number(store.get(control.key))"
                @change="setNumber(control, ($event.target as HTMLInputElement).value)"
              />

              <Input
                v-else
                :id="control.key"
                class="w-64"
                :model-value="store.get(control.key)"
                @change="setText(control.key, ($event.target as HTMLInputElement).value)"
              />
            </div>
          </template>
        </CardContent>
      </Card>
    </div>

    <p class="text-xs text-muted-foreground">{{ t("gameSettings.note") }}</p>
  </div>
</template>
