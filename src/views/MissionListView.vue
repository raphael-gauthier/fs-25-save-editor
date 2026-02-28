<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useMissionStore } from "@/stores/mission";
import type { Mission } from "@/lib/types";
import MissionEditor from "@/components/missions/MissionEditor.vue";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
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
import { ClipboardList } from "lucide-vue-next";

const { t } = useI18n();
const store = useMissionStore();

const editorOpen = ref(false);
const selectedMission = ref<Mission | null>(null);

function openEditor(mission: Mission) {
  selectedMission.value = mission;
  editorOpen.value = true;
}

function statusLabel(status: Mission["status"]): string {
  switch (status) {
    case "Created":
      return t("mission.statusCreated");
    case "Running":
      return t("mission.statusRunning");
    case "Completed":
      return t("mission.statusCompleted");
  }
}

function statusBadgeVariant(status: Mission["status"]): "default" | "secondary" | "outline" {
  switch (status) {
    case "Running":
      return "default";
    case "Completed":
      return "secondary";
    default:
      return "outline";
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleStatusFilter(value: any) {
  store.statusFilter = value === "__all__" ? null : String(value);
}

function formatCompletion(completion: number): string {
  return `${Math.round(completion * 100)}%`;
}
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-2xl font-semibold">{{ t("mission.title") }}</h2>
        <p class="text-sm text-muted-foreground">{{ t("mission.subtitle") }}</p>
      </div>
    </div>

    <!-- Filters -->
    <div class="flex flex-wrap items-center gap-3">
      <Input
        v-model="store.searchQuery"
        :placeholder="t('mission.searchPlaceholder')"
        class="max-w-xs"
      />
      <Select
        :model-value="store.statusFilter ?? '__all__'"
        @update:model-value="handleStatusFilter"
      >
        <SelectTrigger class="w-48">
          <SelectValue :placeholder="t('mission.allStatuses')" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="__all__">{{ t("mission.allStatuses") }}</SelectItem>
          <SelectItem value="Created">{{ t("mission.statusCreated") }}</SelectItem>
          <SelectItem value="Running">{{ t("mission.statusRunning") }}</SelectItem>
          <SelectItem value="Completed">{{ t("mission.statusCompleted") }}</SelectItem>
        </SelectContent>
      </Select>
      <span class="text-sm text-muted-foreground">
        {{ t("mission.found", { count: store.filteredMissions.length }) }}
      </span>
    </div>

    <!-- Table -->
    <div v-if="store.filteredMissions.length > 0" class="rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>{{ t("mission.type") }}</TableHead>
            <TableHead>{{ t("mission.status") }}</TableHead>
            <TableHead>{{ t("mission.field") }}</TableHead>
            <TableHead class="text-right">{{ t("mission.reward") }}</TableHead>
            <TableHead class="text-right">{{ t("mission.completion") }}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow
            v-for="m in store.filteredMissions"
            :key="m.uniqueId"
            class="cursor-pointer"
            @click="openEditor(m)"
          >
            <TableCell>
              <div class="flex items-center gap-2">
                <ClipboardList class="size-4 text-muted-foreground" />
                <span>{{ t(`mission.types.${m.missionType}`, m.missionType) }}</span>
                <span v-if="m.fruitType" class="text-xs text-muted-foreground">
                  ({{ t(`fillTypes.${m.fruitType}`, m.fruitType) }})
                </span>
              </div>
            </TableCell>
            <TableCell>
              <Badge :variant="statusBadgeVariant(m.status)" class="text-xs">
                {{ statusLabel(m.status) }}
              </Badge>
            </TableCell>
            <TableCell>
              {{ m.fieldId ? `#${m.fieldId}` : "-" }}
            </TableCell>
            <TableCell class="text-right font-mono">
              {{ Math.round(m.reward).toLocaleString() }} $
            </TableCell>
            <TableCell class="text-right font-mono">
              {{ formatCompletion(m.completion) }}
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>

    <!-- Empty state -->
    <div v-else-if="store.missions.length === 0" class="py-12 text-center">
      <ClipboardList class="mx-auto size-12 text-muted-foreground/50" />
      <p class="mt-4 text-muted-foreground">{{ t("mission.empty") }}</p>
    </div>

    <!-- No matches -->
    <div v-else class="py-12 text-center">
      <p class="text-muted-foreground">{{ t("mission.noMatch") }}</p>
    </div>

    <!-- Mission Editor Sheet -->
    <MissionEditor
      :mission="selectedMission"
      :open="editorOpen"
      @update:open="editorOpen = $event"
    />
  </div>
</template>
