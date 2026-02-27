<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { open } from "@tauri-apps/plugin-dialog";
import { useSettingsStore } from "@/stores/settings";
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
import { FolderOpen } from "lucide-vue-next";

const { t } = useI18n();
const settings = useSettingsStore();

async function browsePath() {
  const selected = await open({
    directory: true,
    title: t("savegame.folderDialogTitle"),
  });
  if (selected) {
    settings.setDefaultPath(selected);
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
    </div>
  </div>
</template>
