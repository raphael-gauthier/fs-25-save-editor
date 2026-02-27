<script setup lang="ts">
import { onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { open } from "@tauri-apps/plugin-dialog";
import { useSavegameStore } from "@/stores/savegame";
import { useSettingsStore } from "@/stores/settings";
import { formatMoney, formatPlayTime } from "@/lib/utils";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Tractor,
  FolderOpen,
  AlertCircle,
  SearchX,
  MapPin,
  Clock,
  Coins,
  BarChart3,
  Calendar,
} from "lucide-vue-next";

const { t } = useI18n();
const store = useSavegameStore();
const settings = useSettingsStore();

onMounted(() => {
  store.listSavegames(settings.defaultPath || undefined);
});

async function openCustomFolder() {
  const selected = await open({
    directory: true,
    title: t("savegame.folderDialogTitle"),
  });
  if (selected) {
    await store.listSavegames(selected);
  }
}

function formatDifficulty(code: string): string {
  const key = `difficulty.${code}`;
  const translated = t(key);
  return translated === key ? code : translated;
}
</script>

<template>
  <main class="flex min-h-screen flex-col items-center px-4 py-12">
    <div class="w-full max-w-2xl space-y-8">
      <!-- Header -->
      <div class="text-center space-y-2">
        <div class="flex items-center justify-center gap-3">
          <Tractor class="size-8 text-primary" />
          <h1 class="text-3xl font-bold tracking-tight">{{ t("common.appName") }}</h1>
        </div>
        <p class="text-muted-foreground">
          {{ t("savegame.selectSubtitle") }}
        </p>
      </div>

      <!-- Loading skeletons -->
      <div v-if="store.isLoading" class="space-y-3">
        <Skeleton v-for="i in 3" :key="i" class="h-28 w-full rounded-xl" />
      </div>

      <!-- Error state -->
      <Alert v-else-if="store.error" variant="destructive">
        <AlertCircle class="size-4" />
        <AlertTitle>{{ t("common.error") }}</AlertTitle>
        <AlertDescription>{{ t(store.error.code, store.error.params) }}</AlertDescription>
      </Alert>

      <!-- Empty state -->
      <div
        v-else-if="store.savegames.length === 0"
        class="flex flex-col items-center gap-4 rounded-xl border border-dashed p-10 text-center"
      >
        <SearchX class="size-10 text-muted-foreground" />
        <div class="space-y-1">
          <p class="font-medium">{{ t("savegame.noSavegameTitle") }}</p>
          <p class="text-sm text-muted-foreground">
            {{ t("savegame.noSavegameDesc") }}
          </p>
        </div>
      </div>

      <!-- Savegame list -->
      <div v-else class="space-y-3">
        <Card
          v-for="save in store.savegames"
          :key="save.path"
          class="cursor-pointer transition-colors hover:bg-accent/50"
          @click="store.selectSavegame(save.path)"
        >
          <CardHeader class="pb-2">
            <CardTitle class="text-lg">{{ save.name }}</CardTitle>
            <CardDescription class="flex items-center gap-1.5">
              <MapPin class="size-3.5" />
              {{ save.mapTitle }}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div class="flex flex-wrap gap-x-5 gap-y-1 text-sm text-muted-foreground">
              <span class="flex items-center gap-1.5">
                <Coins class="size-3.5" />
                {{ formatMoney(save.money) }} $
              </span>
              <span class="flex items-center gap-1.5">
                <Clock class="size-3.5" />
                {{ formatPlayTime(save.playTime) }}
              </span>
              <span class="flex items-center gap-1.5">
                <BarChart3 class="size-3.5" />
                {{ formatDifficulty(save.economicDifficulty) }}
              </span>
              <span class="flex items-center gap-1.5">
                <Calendar class="size-3.5" />
                {{ save.saveDate }}
              </span>
            </div>
          </CardContent>
        </Card>
      </div>

      <!-- Open custom folder button -->
      <div class="flex justify-center">
        <Button variant="outline" @click="openCustomFolder">
          <FolderOpen class="size-4" />
          {{ t("savegame.openFolder") }}
        </Button>
      </div>
    </div>
  </main>
</template>
