<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { open } from "@tauri-apps/plugin-dialog";
import { toast } from "vue-sonner";
import { useSettingsStore } from "@/stores/settings";
import { useVehicleImages } from "@/composables/useVehicleImages";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { FolderOpen, Search, Trash2, RefreshCw } from "lucide-vue-next";
import { getVersion } from "@tauri-apps/api/app";
import { useUpdateChecker } from "@/composables/useUpdateChecker";

const { t } = useI18n();
const settings = useSettingsStore();
const { detectGamePath, clearDiskCache, getCacheSize } = useVehicleImages();
const { isChecking, checkForUpdates } = useUpdateChecker();

const cacheSize = ref(0);
const checkResult = ref<"up-to-date" | "error" | null>(null);
const appVersion = ref("");

onMounted(async () => {
  cacheSize.value = await getCacheSize();
  appVersion.value = await getVersion();
});

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

async function browseGamePath() {
  const selected = await open({
    directory: true,
    title: t("settings.gamePathDialogTitle"),
  });
  if (selected) {
    await settings.setGamePath(selected);
    cacheSize.value = await getCacheSize();
  }
}

async function handleDetectGamePath() {
  const detected = await detectGamePath();
  if (detected) {
    await settings.setGamePath(detected);
    toast.success(t("settings.gamePathDetected"));
  } else {
    toast.error(t("settings.gamePathNotFound"));
  }
}

async function handleClearCache() {
  const freed = await clearDiskCache();
  cacheSize.value = 0;
  toast.success(t("settings.cacheClearedMsg", { size: formatBytes(freed) }));
}

async function browsePath() {
  const selected = await open({
    directory: true,
    title: t("savegame.folderDialogTitle"),
  });
  if (selected) {
    settings.setDefaultPath(selected);
  }
}

async function onManualCheck() {
  checkResult.value = null;
  try {
    const result = await checkForUpdates();
    if (!result) {
      checkResult.value = "up-to-date";
    }
    // If result !== null, the composable updates updateAvailable
    // and UpdateDialog shows automatically
  } catch {
    checkResult.value = "error";
  }
}

function handleMaxBackupsInput(event: Event) {
  const value = parseInt((event.target as HTMLInputElement).value);
  if (!isNaN(value) && value >= 1) {
    settings.setMaxBackups(value);
  }
}
</script>

<template>
  <div class="space-y-6">
    <div>
      <h2 class="text-2xl font-semibold">{{ t("settings.title") }}</h2>
      <p class="text-sm text-muted-foreground">{{ t("settings.subtitle") }}</p>
    </div>

    <div class="grid gap-6 max-w-2xl">
      <!-- Language -->
      <Card>
        <CardHeader>
          <CardTitle class="text-base">{{ t("settings.language") }}</CardTitle>
          <CardDescription>{{ t("settings.languageDesc") }}</CardDescription>
        </CardHeader>
        <CardContent>
          <Select
            :model-value="settings.locale"
            @update:model-value="settings.setLocale($event as string)"
          >
            <SelectTrigger class="w-[200px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="fr">{{ t("settings.french") }}</SelectItem>
              <SelectItem value="en">{{ t("settings.english") }}</SelectItem>
            </SelectContent>
          </Select>
        </CardContent>
      </Card>

      <!-- Theme -->
      <Card>
        <CardHeader>
          <CardTitle class="text-base">{{ t("settings.theme") }}</CardTitle>
          <CardDescription>{{ t("settings.themeDesc") }}</CardDescription>
        </CardHeader>
        <CardContent>
          <Select
            :model-value="settings.theme"
            @update:model-value="settings.setTheme($event as 'light' | 'dark' | 'system')"
          >
            <SelectTrigger class="w-[200px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="light">{{ t("settings.themeLight") }}</SelectItem>
              <SelectItem value="dark">{{ t("settings.themeDark") }}</SelectItem>
              <SelectItem value="system">{{ t("settings.themeSystem") }}</SelectItem>
            </SelectContent>
          </Select>
        </CardContent>
      </Card>

      <!-- Advanced Mode -->
      <Card>
        <CardHeader>
          <CardTitle class="text-base">{{ t("settings.advancedMode") }}</CardTitle>
          <CardDescription>{{ t("settings.advancedModeDesc") }}</CardDescription>
        </CardHeader>
        <CardContent>
          <div class="flex items-center gap-3">
            <Switch
              :model-value="settings.advancedMode"
              @update:model-value="settings.setAdvancedMode($event)"
            />
            <Label class="text-sm">{{ t("common.advancedMode") }}</Label>
          </div>
        </CardContent>
      </Card>

      <!-- Default Path -->
      <Card>
        <CardHeader>
          <CardTitle class="text-base">{{ t("settings.defaultPath") }}</CardTitle>
          <CardDescription>{{ t("settings.defaultPathDesc") }}</CardDescription>
        </CardHeader>
        <CardContent>
          <div class="flex items-center gap-2">
            <Input
              :model-value="settings.defaultPath"
              :placeholder="t('settings.defaultPathPlaceholder')"
              readonly
              class="flex-1"
            />
            <Button variant="outline" size="sm" @click="browsePath">
              <FolderOpen class="size-4" />
              {{ t("settings.browse") }}
            </Button>
          </div>
        </CardContent>
      </Card>

      <!-- Max Backups -->
      <Card>
        <CardHeader>
          <CardTitle class="text-base">{{ t("settings.maxBackups") }}</CardTitle>
          <CardDescription>{{ t("settings.maxBackupsDesc") }}</CardDescription>
        </CardHeader>
        <CardContent>
          <Input
            type="number"
            :model-value="settings.maxBackups"
            @input="handleMaxBackupsInput"
            class="w-[120px]"
            min="1"
            max="100"
            step="1"
          />
        </CardContent>
      </Card>

      <!-- Game Path -->
      <Card>
        <CardHeader>
          <CardTitle class="text-base">{{ t("settings.gamePath") }}</CardTitle>
          <CardDescription>{{ t("settings.gamePathDesc") }}</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-center gap-2">
            <Input
              :model-value="settings.gamePath ?? ''"
              :placeholder="t('settings.gamePathPlaceholder')"
              readonly
              class="flex-1"
            />
            <Button variant="outline" size="sm" @click="browseGamePath">
              <FolderOpen class="size-4" />
              {{ t("settings.browse") }}
            </Button>
            <Button variant="outline" size="sm" @click="handleDetectGamePath">
              <Search class="size-4" />
              {{ t("settings.detect") }}
            </Button>
          </div>
          <div v-if="settings.gamePath" class="flex items-center justify-between text-sm text-muted-foreground">
            <span>{{ t("settings.imageCache") }}: {{ formatBytes(cacheSize) }}</span>
            <Button variant="ghost" size="sm" @click="handleClearCache">
              <Trash2 class="size-4" />
              {{ t("settings.clearCache") }}
            </Button>
          </div>
        </CardContent>
      </Card>
      <!-- Updates -->
      <Card>
        <CardHeader>
          <CardTitle class="text-base">{{ t("settings.updates") }}</CardTitle>
          <CardDescription>{{ t("settings.updatesDesc") }}</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-center justify-between">
            <div class="space-y-0.5">
              <Label>{{ t("settings.autoCheckUpdates") }}</Label>
              <p class="text-muted-foreground text-sm">
                {{ t("settings.autoCheckUpdatesDesc") }}
              </p>
            </div>
            <Switch
              :checked="settings.checkForUpdatesOnStartup"
              @update:checked="settings.setCheckForUpdatesOnStartup"
            />
          </div>

          <div class="flex items-center gap-3">
            <Button
              variant="outline"
              size="sm"
              :disabled="isChecking"
              @click="onManualCheck"
            >
              <RefreshCw v-if="isChecking" class="mr-2 h-4 w-4 animate-spin" />
              <RefreshCw v-else class="mr-2 h-4 w-4" />
              {{ t("settings.checkNow") }}
            </Button>
            <span v-if="checkResult === 'up-to-date'" class="text-sm text-green-600 dark:text-green-400">
              {{ t("settings.upToDate") }}
            </span>
            <span v-if="checkResult === 'error'" class="text-sm text-red-600 dark:text-red-400">
              {{ t("settings.checkFailed") }}
            </span>
          </div>

          <p class="text-muted-foreground text-xs">
            {{ t("settings.currentVersion", { version: appVersion }) }}
          </p>
        </CardContent>
      </Card>
    </div>
  </div>
</template>
