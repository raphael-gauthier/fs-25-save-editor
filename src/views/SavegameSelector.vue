<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { open } from "@tauri-apps/plugin-dialog";
import { AnimatePresence, motion } from "motion-v";
import { useSavegameStore } from "@/stores/savegame";
import { useSettingsStore } from "@/stores/settings";
import { formatMoney, formatPlayTime } from "@/lib/utils";
import type { SavegameSummary } from "@/lib/types";
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
import WindowTitleBar from "@/components/layout/WindowTitleBar.vue";
import {
  FolderOpen,
  AlertCircle,
  SearchX,
  Search,
  ArrowUpDown,
  ShieldCheck,
  Folder,
} from "@lucide/vue";

const { t } = useI18n();
const store = useSavegameStore();
const settings = useSettingsStore();

const searchQuery = ref("");
const sortKey = ref<"date" | "name" | "money" | "playTime">("date");
const selectedPath = ref<string | null>(null);

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

watch(
  filteredSavegames,
  (list) => {
    if (!list.some((s) => s.path === selectedPath.value)) selectedPath.value = list[0]?.path ?? null;
  },
  { immediate: true },
);

const selected = computed(() => filteredSavegames.value.find((s) => s.path === selectedPath.value));

onMounted(() => {
  store.listSavegames(settings.defaultPath || undefined);
});

async function openCustomFolder() {
  const selectedFolder = await open({
    directory: true,
    title: t("savegame.folderDialogTitle"),
  });
  if (selectedFolder) {
    await store.listSavegames(selectedFolder);
  }
}

function formatDifficulty(code: string): string {
  const key = `difficulty.${code}`;
  const translated = t(key);
  return translated === key ? code : translated;
}

/** "savegame3" → 3, the slot the game shows. */
function slotOf(save: SavegameSummary): string | null {
  const match = save.path.replace(/[\\/]+$/, "").match(/savegame(\d+)$/i);
  return match ? match[1] : null;
}

function pick(save: SavegameSummary) {
  // A second click on the chosen save opens it
  if (selectedPath.value === save.path) store.selectSavegame(save.path);
  else selectedPath.value = save.path;
}

function onListKeydown(e: KeyboardEvent) {
  const list = filteredSavegames.value;
  if (!list.length || (e.key !== "ArrowDown" && e.key !== "ArrowUp")) return;
  e.preventDefault();
  const i = list.findIndex((s) => s.path === selectedPath.value);
  const next = list[(i + (e.key === "ArrowDown" ? 1 : -1) + list.length) % list.length];
  selectedPath.value = next.path;
  document.getElementById(`save-${next.path}`)?.focus();
}

// Decorative field mosaic, its colors seeded from the map's name
const CROPS = ["#7baf5e", "#d6c23a", "#c9a458", "#5fa05a", "#e0b53a", "#8fa448", "#b08a4a", "#d9772b", "#9dbf6b", "#a8543a"];

function mosaic(seedText: string) {
  let seed = 0;
  for (const ch of seedText) seed = (seed * 31 + ch.charCodeAt(0)) | 0;
  const rand = () => {
    seed = (seed * 1103515245 + 12345) & 0x7fffffff;
    return seed / 0x7fffffff;
  };
  return Array.from({ length: 24 }, (_, i) => ({
    color: CROPS[Math.floor(rand() * CROPS.length)],
    vertical: rand() > 0.5,
    delay: (i % 8) * 0.03 + Math.floor(i / 8) * 0.05,
  }));
}

const tiles = computed(() => (selected.value ? mosaic(selected.value.mapTitle) : []));
</script>

<template>
  <div class="flex h-full flex-col">
    <WindowTitleBar />

    <div class="flex min-h-0 flex-1 gap-2.5 px-2.5 pb-2.5">
      <!-- Saves -->
      <aside
        aria-labelledby="saves-title"
        class="glass flex w-[340px] shrink-0 flex-col gap-3.5 rounded-2xl px-3 pb-3.5 pt-5"
      >
        <div class="flex flex-col gap-0.5 px-2">
          <h1 id="saves-title" class="text-2xl font-semibold tracking-tight">{{ t("launcher.title") }}</h1>
          <p class="text-[13px] text-muted-foreground">{{ t("savegame.selectSubtitle") }}</p>
        </div>

        <div class="flex gap-2 px-2">
          <div class="relative flex-1">
            <Search class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
            <Input
              v-model="searchQuery"
              type="search"
              :aria-label="t('savegame.searchPlaceholder')"
              :placeholder="t('launcher.search')"
              class="h-10 pl-9"
            />
          </div>
          <Select v-model="sortKey">
            <SelectTrigger class="h-10 w-auto" :aria-label="t('savegame.sortBy')">
              <ArrowUpDown class="size-4 text-muted-foreground" aria-hidden="true" />
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="date">{{ t("savegame.sortDate") }}</SelectItem>
              <SelectItem value="name">{{ t("savegame.sortName") }}</SelectItem>
              <SelectItem value="money">{{ t("savegame.sortMoney") }}</SelectItem>
              <SelectItem value="playTime">{{ t("savegame.sortPlayTime") }}</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div class="min-h-0 flex-1 overflow-y-auto">
          <div v-if="store.isLoading" class="flex flex-col gap-2">
            <Skeleton v-for="i in 3" :key="i" class="h-[68px] w-full rounded-xl" />
          </div>
          <ul
            v-else
            :aria-label="t('launcher.title')"
            class="flex flex-col"
            @keydown="onListKeydown"
          >
            <li v-for="save in filteredSavegames" :key="save.path" class="relative">
              <motion.span
                v-if="save.path === selectedPath"
                layout-id="launcher-active"
                class="absolute inset-0 rounded-xl bg-primary-soft"
                :transition="{ type: 'spring', stiffness: 480, damping: 40 }"
              />
              <button
                :id="`save-${save.path}`"
                type="button"
                :aria-current="save.path === selectedPath ? 'true' : undefined"
                class="relative flex h-[68px] w-full items-center gap-3 rounded-xl px-2.5 text-left transition-colors hover:bg-hover"
                @click="pick(save)"
                @dblclick="store.selectSavegame(save.path)"
              >
                <span
                  class="size-11 shrink-0 rounded-[10px] bg-[repeating-linear-gradient(90deg,rgb(0_0_0/0.09)_0_2px,transparent_2px_9px)]"
                  :style="{ backgroundColor: mosaic(save.mapTitle)[0].color }"
                  aria-hidden="true"
                />
                <span class="flex min-w-0 flex-1 flex-col leading-snug">
                  <span class="truncate" :class="save.path === selectedPath ? 'font-semibold' : 'font-medium'">{{ save.mapTitle }}</span>
                  <span class="truncate text-[13px] text-muted-foreground">
                    <template v-if="slotOf(save)">{{ t("launcher.slot", { n: slotOf(save) }) }} · </template>{{ save.name }}
                  </span>
                </span>
                <span class="shrink-0 text-xs text-muted-foreground tabular">{{ save.saveDate }}</span>
              </button>
            </li>
          </ul>
        </div>

        <Button variant="outline" class="mx-2 h-10" @click="openCustomFolder">
          <FolderOpen class="size-4" />
          {{ t("savegame.openFolder") }}
        </Button>
      </aside>

      <!-- Preview -->
      <main class="glass-strong flex min-w-0 flex-1 flex-col gap-6 overflow-y-auto rounded-2xl p-[clamp(16px,3vw,32px)]">
        <Alert v-if="store.error" variant="destructive">
          <AlertCircle class="size-4" />
          <AlertTitle>{{ t("common.error") }}</AlertTitle>
          <AlertDescription>{{ t(store.error.code, store.error.params) }}</AlertDescription>
        </Alert>

        <div
          v-else-if="!store.isLoading && store.savegames.length === 0"
          class="m-auto flex max-w-sm flex-col items-center gap-4 text-center"
        >
          <SearchX class="size-10 text-muted-foreground" />
          <div class="space-y-1">
            <p class="font-medium">{{ t("savegame.noSavegameTitle") }}</p>
            <p class="text-sm text-muted-foreground">{{ t("savegame.noSavegameDesc") }}</p>
          </div>
        </div>

        <AnimatePresence mode="wait">
          <motion.div
            v-if="selected"
            :key="selected.path"
            class="flex flex-col gap-6"
            :initial="{ opacity: 0, y: 8 }"
            :animate="{ opacity: 1, y: 0 }"
            :exit="{ opacity: 0, y: -6 }"
            :transition="{ duration: 0.22, ease: [0.32, 0.72, 0, 1] }"
          >
            <div
              class="grid h-[min(260px,32vh)] grid-cols-8 grid-rows-3 gap-[5px] overflow-hidden rounded-2xl bg-[#3f3424] p-[5px]"
              aria-hidden="true"
            >
              <motion.span
                v-for="(tile, i) in tiles"
                :key="i"
                class="rounded-md"
                :class="tile.vertical
                  ? 'bg-[repeating-linear-gradient(90deg,rgb(0_0_0/0.09)_0_2px,transparent_2px_9px)]'
                  : 'bg-[repeating-linear-gradient(0deg,rgb(0_0_0/0.09)_0_2px,transparent_2px_9px)]'"
                :style="{ backgroundColor: tile.color }"
                :initial="{ opacity: 0, scale: 0.6 }"
                :animate="{ opacity: 1, scale: 1 }"
                :transition="{ type: 'spring', stiffness: 300, damping: 24, delay: tile.delay }"
              />
            </div>

            <div class="flex flex-wrap items-end justify-between gap-4">
              <div class="flex flex-col gap-1">
                <span
                  v-if="slotOf(selected)"
                  class="self-start rounded-full bg-hover px-2.5 py-0.5 text-xs font-semibold text-muted-foreground"
                >
                  {{ t("launcher.slot", { n: slotOf(selected) }) }}
                </span>
                <h2 class="text-4xl font-semibold leading-tight tracking-[-0.03em]">{{ selected.mapTitle }}</h2>
                <span class="text-muted-foreground">
                  {{ selected.name }} · {{ t("launcher.savedOn", { date: selected.saveDate }) }}
                </span>
              </div>
              <Button size="lg" class="h-11 gap-2.5 px-5" @click="store.selectSavegame(selected.path)">
                {{ t("launcher.open") }}
              </Button>
            </div>

            <dl class="grid grid-cols-[repeat(auto-fit,minmax(150px,1fr))] gap-2.5">
              <div class="rounded-xl bg-hover px-4 py-3.5">
                <dt class="text-[13px] text-muted-foreground">{{ t("savegame.sortMoney") }}</dt>
                <dd class="text-2xl font-semibold tracking-tight tabular">{{ formatMoney(selected.money) }}</dd>
              </div>
              <div class="rounded-xl bg-hover px-4 py-3.5">
                <dt class="text-[13px] text-muted-foreground">{{ t("savegame.sortPlayTime") }}</dt>
                <dd class="text-2xl font-semibold tracking-tight tabular">{{ formatPlayTime(selected.playTime) }}</dd>
              </div>
              <div class="rounded-xl bg-hover px-4 py-3.5">
                <dt class="text-[13px] text-muted-foreground">{{ t("launcher.difficulty") }}</dt>
                <dd class="text-2xl font-semibold tracking-tight">{{ formatDifficulty(selected.economicDifficulty) }}</dd>
              </div>
            </dl>

            <div class="flex flex-col gap-2 text-[13px] text-muted-foreground">
              <p class="flex items-center gap-2">
                <ShieldCheck class="size-4 shrink-0" aria-hidden="true" />
                {{ t("launcher.backupNote") }}
              </p>
              <p class="flex min-w-0 items-center gap-2">
                <Folder class="size-4 shrink-0" aria-hidden="true" />
                <span class="truncate">{{ selected.path }}</span>
              </p>
            </div>
          </motion.div>
        </AnimatePresence>
      </main>
    </div>
  </div>
</template>
