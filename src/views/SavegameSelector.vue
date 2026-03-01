<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
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
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import logo from "@/assets/logo.png";
import {
  FolderOpen,
  AlertCircle,
  SearchX,
  MapPin,
  Clock,
  Coins,
  BarChart3,
  Calendar,
  Search,
  ArrowUpDown,
} from "lucide-vue-next";

const { t } = useI18n();
const store = useSavegameStore();
const settings = useSettingsStore();

const searchQuery = ref("");
const sortKey = ref<"date" | "name" | "money" | "playTime">("date");

const filteredSavegames = computed(() => {
  let result = store.savegames;

  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase();
    result = result.filter(
      (s) =>
        s.name.toLowerCase().includes(q) ||
        s.mapTitle.toLowerCase().includes(q),
    );
  }

  // For "date" sort, preserve the backend order (already sorted by filesystem mtime desc)
  if (sortKey.value !== "date") {
    result = [...result].sort((a, b) => {
      switch (sortKey.value) {
        case "name":
          return a.name.localeCompare(b.name);
        case "money":
          return b.money - a.money;
        case "playTime":
          return b.playTime - a.playTime;
        default:
          return 0;
      }
    });
  }

  return result;
});

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
    <div class="w-full max-w-5xl space-y-8">
      <!-- Header -->
      <div class="text-center space-y-3">
        <img
          :src="logo"
          :alt="t('common.appName')"
          class="mx-auto h-24 w-auto dark:drop-shadow-[0_0_0.5px_rgba(255,255,255,0.8)] dark:brightness-110"
        />
        <p class="text-muted-foreground">
          {{ t("savegame.selectSubtitle") }}
        </p>
      </div>

      <!-- Loading skeletons -->
      <div
        v-if="store.isLoading"
        class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3"
      >
        <Skeleton v-for="i in 6" :key="i" class="h-40 w-full rounded-xl" />
      </div>

      <!-- Error state -->
      <Alert v-else-if="store.error" variant="destructive">
        <AlertCircle class="size-4" />
        <AlertTitle>{{ t("common.error") }}</AlertTitle>
        <AlertDescription>{{
          t(store.error.code, store.error.params)
        }}</AlertDescription>
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

      <!-- Savegame grid -->
      <template v-else>
        <!-- Toolbar -->
        <div class="flex flex-col gap-3 sm:flex-row sm:items-center">
          <div class="relative flex-1">
            <Search
              class="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
            />
            <Input
              v-model="searchQuery"
              :placeholder="t('savegame.searchPlaceholder')"
              class="pl-9"
            />
          </div>
          <div class="flex items-center gap-3">
            <Select v-model="sortKey">
              <SelectTrigger class="w-44">
                <ArrowUpDown class="mr-2 size-4 text-muted-foreground" />
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="date">{{
                  t("savegame.sortDate")
                }}</SelectItem>
                <SelectItem value="name">{{
                  t("savegame.sortName")
                }}</SelectItem>
                <SelectItem value="money">{{
                  t("savegame.sortMoney")
                }}</SelectItem>
                <SelectItem value="playTime">{{
                  t("savegame.sortPlayTime")
                }}</SelectItem>
              </SelectContent>
            </Select>
            <span class="text-xs text-muted-foreground whitespace-nowrap">
              {{ t("savegame.savegameCount", { count: filteredSavegames.length }) }}
            </span>
          </div>
        </div>

        <!-- No search results -->
        <div
          v-if="filteredSavegames.length === 0"
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

        <!-- Grid -->
        <div
          v-else
          class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3"
        >
          <Card
            v-for="save in filteredSavegames"
            :key="save.path"
            class="cursor-pointer transition-colors hover:bg-accent/50"
            @click="store.selectSavegame(save.path)"
          >
            <CardHeader class="pb-2">
              <CardTitle class="text-base">{{ save.name }}</CardTitle>
              <CardDescription class="flex items-center gap-1.5">
                <MapPin class="size-3.5 shrink-0" />
                {{ save.mapTitle }}
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div class="grid grid-cols-2 gap-x-4 gap-y-1 text-xs text-muted-foreground">
                <span class="flex items-center gap-1.5">
                  <Coins class="size-3 shrink-0" />
                  {{ formatMoney(save.money) }} $
                </span>
                <span class="flex items-center gap-1.5">
                  <Clock class="size-3 shrink-0" />
                  {{ formatPlayTime(save.playTime) }}
                </span>
                <span class="flex items-center gap-1.5">
                  <BarChart3 class="size-3 shrink-0" />
                  {{ formatDifficulty(save.economicDifficulty) }}
                </span>
                <span class="flex items-center gap-1.5">
                  <Calendar class="size-3 shrink-0" />
                  {{ save.saveDate }}
                </span>
              </div>
            </CardContent>
          </Card>
        </div>
      </template>

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
