<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useMissionStore } from "@/stores/mission";
import { useSettingsStore } from "@/stores/settings";
import type { Mission } from "@/lib/types";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Separator } from "@/components/ui/separator";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
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
import { ClipboardList, CheckCircle } from "lucide-vue-next";

interface Props {
  mission: Mission | null;
  open: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  "update:open": [value: boolean];
}>();

const { t } = useI18n();
const store = useMissionStore();
const settings = useSettingsStore();

const original = computed(() => {
  if (!props.mission) return undefined;
  return store.missions.find((m) => m.uniqueId === props.mission!.uniqueId);
});

function handleRewardChange(event: Event) {
  const target = event.target as HTMLInputElement;
  const value = parseFloat(target.value);
  if (props.mission && !isNaN(value)) {
    store.updateMission(props.mission.uniqueId, { reward: value });
  }
}

function handleCompletionChange(value: number[] | undefined) {
  if (value && props.mission) {
    store.updateMission(props.mission.uniqueId, { completion: value[0] / 100 });
  }
}

function handleCompleteMission() {
  if (props.mission) {
    store.completeMission(props.mission.uniqueId);
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handleStatusChange(value: any) {
  if (props.mission) {
    store.updateMission(props.mission.uniqueId, { status: value as Mission["status"] });
  }
}

function handleReimbursementChange(event: Event) {
  const target = event.target as HTMLInputElement;
  const value = parseFloat(target.value);
  if (props.mission && !isNaN(value)) {
    store.updateMission(props.mission.uniqueId, { reimbursement: value });
  }
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
</script>

<template>
  <Sheet :open="open" @update:open="emit('update:open', $event)">
    <SheetContent class="overflow-y-auto sm:max-w-md">
      <template v-if="mission && original">
        <SheetHeader>
          <SheetTitle class="flex items-center gap-2">
            <ClipboardList class="size-5" />
            {{ mission.missionType }}
            <template v-if="mission.fruitType">
              ({{ t(`fillTypes.${mission.fruitType}`, mission.fruitType) }})
            </template>
          </SheetTitle>
          <SheetDescription>
            {{ t("mission.missionId", { id: mission.uniqueId }) }}
            <template v-if="mission.fieldId">
              &middot; {{ t("mission.field") }} #{{ mission.fieldId }}
            </template>
          </SheetDescription>
        </SheetHeader>

        <div class="mt-6 space-y-6">
          <!-- Status badge -->
          <div class="flex items-center gap-2">
            <Badge
              :variant="mission.status === 'Running' ? 'default' : mission.status === 'Completed' ? 'secondary' : 'outline'"
              class="text-xs"
            >
              {{ statusLabel(mission.status) }}
            </Badge>
          </div>

          <!-- Reward -->
          <div class="space-y-2">
            <Label>{{ t("mission.reward") }}</Label>
            <Input
              type="number"
              :model-value="original.reward"
              @change="handleRewardChange"
            />
          </div>

          <!-- Completion slider -->
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <Label>{{ t("mission.completion") }}</Label>
              <span class="font-mono text-sm text-muted-foreground">
                {{ Math.round(original.completion * 100) }}%
              </span>
            </div>
            <Slider
              :model-value="[Math.round(original.completion * 100)]"
              :max="100"
              :min="0"
              :step="1"
              @update:model-value="handleCompletionChange"
            />
          </div>

          <!-- Complete mission button -->
          <AlertDialog v-if="original.status !== 'Completed'">
            <AlertDialogTrigger as-child>
              <Button variant="default" class="w-full">
                <CheckCircle class="size-4" />
                {{ t("mission.completeMission") }}
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>{{ t("mission.completeMission") }}</AlertDialogTitle>
                <AlertDialogDescription>
                  {{ t("mission.completeMissionDesc") }}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>{{ t("common.cancel") }}</AlertDialogCancel>
                <AlertDialogAction @click="handleCompleteMission">
                  {{ t("common.confirm") }}
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>

          <!-- Advanced mode -->
          <template v-if="settings.advancedMode">
            <Separator />

            <!-- Status select -->
            <div class="space-y-2">
              <Label>{{ t("mission.status") }}</Label>
              <Select :model-value="original.status" @update:model-value="handleStatusChange">
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Created">{{ t("mission.statusCreated") }}</SelectItem>
                  <SelectItem value="Running">{{ t("mission.statusRunning") }}</SelectItem>
                  <SelectItem value="Completed">{{ t("mission.statusCompleted") }}</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <!-- Reimbursement -->
            <div class="space-y-2">
              <Label>{{ t("mission.reimbursement") }}</Label>
              <Input
                type="number"
                :model-value="original.reimbursement"
                @change="handleReimbursementChange"
              />
            </div>

            <!-- Mission-specific info (read-only) -->
            <template v-if="mission.expectedLiters || mission.depositedLiters">
              <Separator />
              <div class="space-y-2">
                <div v-if="mission.expectedLiters" class="flex justify-between text-sm">
                  <span class="text-muted-foreground">{{ t("mission.expectedLiters") }}</span>
                  <span class="font-mono">{{ Math.round(mission.expectedLiters).toLocaleString() }} L</span>
                </div>
                <div v-if="mission.depositedLiters" class="flex justify-between text-sm">
                  <span class="text-muted-foreground">{{ t("mission.depositedLiters") }}</span>
                  <span class="font-mono">{{ Math.round(mission.depositedLiters).toLocaleString() }} L</span>
                </div>
              </div>
            </template>
          </template>
        </div>
      </template>
    </SheetContent>
  </Sheet>
</template>
