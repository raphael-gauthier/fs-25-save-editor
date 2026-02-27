<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { useWorldStore } from "@/stores/world";
import { useSettingsStore } from "@/stores/settings";
import { WEATHER_TYPES } from "@/lib/constants";
import { dayTimeToHHMM, hhmmToDayTime, formatDuration } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import { Slider } from "@/components/ui/slider";
import { Separator } from "@/components/ui/separator";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
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
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Sun, CloudRain, Cloud, Snowflake, Tornado, Trash2 } from "lucide-vue-next";

const { t } = useI18n();
const store = useWorldStore();
const settings = useSettingsStore();

const weatherIcons: Record<string, typeof Sun> = {
  SUN: Sun,
  RAIN: CloudRain,
  CLOUDY: Cloud,
  SNOW: Snowflake,
  TWISTER: Tornado,
};

function handleDayChange(e: Event) {
  const val = Number((e.target as HTMLInputElement).value);
  if (!isNaN(val) && val >= 0) store.setCurrentDay(val);
}

function handleTimeChange(e: Event) {
  const val = (e.target as HTMLInputElement).value;
  if (/^\d{1,2}:\d{2}$/.test(val)) {
    store.setDayTime(hhmmToDayTime(val));
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleWeatherTypeChange(index: number, value: any) {
  store.updateWeatherEvent(index, { typeName: String(value) });
}

function handleSnowChange(value: number[] | undefined) {
  if (value) store.setSnowHeight(value[0]);
}

function handleWetnessChange(value: number[] | undefined) {
  if (value) store.setGroundWetness(value[0]);
}
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-2xl font-semibold">{{ t("world.title") }}</h2>
        <p class="text-sm text-muted-foreground">{{ t("world.subtitle") }}</p>
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

    <template v-if="store.environment">
      <!-- Game Time Card -->
      <Card>
        <CardHeader>
          <CardTitle>{{ t("world.gameTime") }}</CardTitle>
        </CardHeader>
        <CardContent>
          <div class="grid gap-4 sm:grid-cols-3">
            <div class="space-y-2">
              <Label>{{ t("world.currentDay") }}</Label>
              <Input
                type="number"
                :model-value="store.environment.currentDay"
                min="1"
                @change="handleDayChange"
              />
            </div>
            <div class="space-y-2">
              <Label>{{ t("world.time") }}</Label>
              <Input
                type="time"
                :model-value="store.currentTime"
                @change="handleTimeChange"
              />
            </div>
            <div class="space-y-2">
              <Label>{{ t("world.season") }}</Label>
              <div class="flex h-9 items-center">
                <Badge variant="secondary" class="text-sm">
                  {{ t("world." + store.currentSeason) }}
                </Badge>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- Weather Forecast -->
      <Card>
        <CardHeader>
          <div class="flex items-center justify-between">
            <div>
              <CardTitle>{{ t("world.forecast") }}</CardTitle>
              <CardDescription>
                {{ store.environment.weatherForecast.length }} {{ t("world.forecast").toLowerCase() }}
              </CardDescription>
            </div>
            <div class="flex gap-2">
              <AlertDialog>
                <AlertDialogTrigger as-child>
                  <Button variant="outline" size="sm">
                    <Sun class="size-4" />
                    {{ t("world.forceSunny") }}
                  </Button>
                </AlertDialogTrigger>
                <AlertDialogContent>
                  <AlertDialogHeader>
                    <AlertDialogTitle>{{ t("world.forceSunny") }}</AlertDialogTitle>
                    <AlertDialogDescription>{{ t("world.forceSunnyDesc") }}</AlertDialogDescription>
                  </AlertDialogHeader>
                  <AlertDialogFooter>
                    <AlertDialogCancel>{{ t("common.cancel") }}</AlertDialogCancel>
                    <AlertDialogAction @click="store.forceSunny()">{{ t("common.confirm") }}</AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>

              <AlertDialog>
                <AlertDialogTrigger as-child>
                  <Button variant="outline" size="sm">
                    <Tornado class="size-4" />
                    {{ t("world.removeTwisters") }}
                  </Button>
                </AlertDialogTrigger>
                <AlertDialogContent>
                  <AlertDialogHeader>
                    <AlertDialogTitle>{{ t("world.removeTwisters") }}</AlertDialogTitle>
                    <AlertDialogDescription>{{ t("world.removeTwistersDesc") }}</AlertDialogDescription>
                  </AlertDialogHeader>
                  <AlertDialogFooter>
                    <AlertDialogCancel>{{ t("common.cancel") }}</AlertDialogCancel>
                    <AlertDialogAction @click="store.removeTwisters()">{{ t("common.confirm") }}</AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div v-if="store.environment.weatherForecast.length > 0" class="rounded-md border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{{ t("world.weatherType") }}</TableHead>
                  <TableHead>{{ t("world.weatherSeason") }}</TableHead>
                  <TableHead>{{ t("world.weatherDay") }}</TableHead>
                  <TableHead>{{ t("world.weatherStart") }}</TableHead>
                  <TableHead>{{ t("world.weatherDuration") }}</TableHead>
                  <TableHead class="w-10"></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow v-for="(event, index) in store.environment.weatherForecast" :key="index">
                  <TableCell>
                    <Select
                      :model-value="event.typeName"
                      @update:model-value="handleWeatherTypeChange(index, $event)"
                    >
                      <SelectTrigger class="w-36">
                        <div class="flex items-center gap-2">
                          <component :is="weatherIcons[event.typeName] ?? Cloud" class="size-4" />
                          <SelectValue />
                        </div>
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem v-for="wt in WEATHER_TYPES" :key="wt" :value="wt">
                          <div class="flex items-center gap-2">
                            <component :is="weatherIcons[wt] ?? Cloud" class="size-4" />
                            {{ t("world." + wt) }}
                          </div>
                        </SelectItem>
                      </SelectContent>
                    </Select>
                  </TableCell>
                  <TableCell>
                    <Badge variant="outline" class="text-xs">
                      {{ t("world." + event.season) }}
                    </Badge>
                  </TableCell>
                  <TableCell class="font-mono">
                    J{{ event.startDay }}
                  </TableCell>
                  <TableCell class="font-mono text-sm">
                    {{ dayTimeToHHMM(event.startDayTime / 1000) }}
                  </TableCell>
                  <TableCell class="text-sm">
                    {{ formatDuration(event.duration) }}
                  </TableCell>
                  <TableCell>
                    <Button variant="ghost" size="icon" class="size-8" @click="store.deleteWeatherEvent(index)">
                      <Trash2 class="size-4 text-destructive" />
                    </Button>
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </div>
          <p v-else class="text-sm text-muted-foreground">{{ t("world.noForecast") }}</p>
        </CardContent>
      </Card>

      <!-- Advanced: Snow & Wetness -->
      <template v-if="settings.advancedMode">
        <Separator />
        <Card>
          <CardHeader>
            <CardTitle class="text-base">{{ t("common.advancedMode") }}</CardTitle>
          </CardHeader>
          <CardContent class="space-y-6">
            <div class="space-y-2">
              <div class="flex items-center justify-between text-sm">
                <Label>{{ t("world.snowHeight") }}</Label>
                <span class="font-mono text-muted-foreground">{{ store.environment.snowHeight.toFixed(2) }}</span>
              </div>
              <Slider
                :model-value="[store.environment.snowHeight]"
                :max="5"
                :min="0"
                :step="0.1"
                @update:model-value="handleSnowChange"
              />
            </div>
            <div class="space-y-2">
              <div class="flex items-center justify-between text-sm">
                <Label>{{ t("world.groundWetness") }}</Label>
                <span class="font-mono text-muted-foreground">{{ store.environment.groundWetness.toFixed(2) }}</span>
              </div>
              <Slider
                :model-value="[store.environment.groundWetness]"
                :max="1"
                :min="0"
                :step="0.01"
                @update:model-value="handleWetnessChange"
              />
            </div>
          </CardContent>
        </Card>
      </template>
    </template>

    <!-- No environment data -->
    <div v-else class="py-12 text-center">
      <Cloud class="mx-auto size-12 text-muted-foreground/50" />
      <p class="mt-4 text-muted-foreground">{{ t("common.noData") }}</p>
    </div>
  </div>
</template>
