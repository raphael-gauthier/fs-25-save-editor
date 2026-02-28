<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { useVehicleStore } from "@/stores/vehicle";
import { useSettingsStore } from "@/stores/settings";
import { formatMoney, vehicleType } from "@/lib/utils";
import { useVehicleImages } from "@/composables/useVehicleImages";
import FillLevelSlider from "@/components/vehicles/FillLevelSlider.vue";
import VehicleImage from "@/components/vehicles/VehicleImage.vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Alert, AlertDescription } from "@/components/ui/alert";
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
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  Table,
  TableBody,
  TableCell,
  TableRow,
} from "@/components/ui/table";
import {
  ArrowLeft,
  Droplets,
  Trash2,
  RotateCcw,
  ChevronRight,
  AlertTriangle,
  Info,
} from "lucide-vue-next";

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const store = useVehicleStore();
const settings = useSettingsStore();

const { loadBatch } = useVehicleImages();

const vehicleId = computed(() => route.params.id as string);
const vehicle = computed(() => store.getVehicleById(vehicleId.value));
const originalVehicle = computed(() => store.getOriginalVehicleById(vehicleId.value));

watch(
  vehicle,
  (v) => {
    if (v && settings.gamePath) {
      loadBatch(settings.gamePath, [v.filename]);
    }
  },
  { immediate: true },
);

function isFieldModified(field: keyof import("@/lib/types").Vehicle): boolean {
  if (!vehicle.value || !originalVehicle.value) return false;
  return vehicle.value[field] !== originalVehicle.value[field];
}

function modifiedClass(field: keyof import("@/lib/types").Vehicle): string {
  return isFieldModified(field) ? "border-l-2 border-amber-500 pl-2" : "";
}

const showDeleteDialog = ref(false);

function goBack() {
  router.push("/editor/vehicles");
}

function handlePriceInput(event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  if (!isNaN(value) && vehicle.value) {
    store.updateVehicle(vehicle.value.uniqueId, { price: Math.max(0, value) });
  }
}

function handleAgeInput(event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  if (!isNaN(value) && vehicle.value) {
    store.updateVehicle(vehicle.value.uniqueId, { age: Math.max(0, value) });
  }
}

function handleOperatingTimeInput(event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  if (!isNaN(value) && vehicle.value) {
    store.updateVehicle(vehicle.value.uniqueId, { operatingTime: Math.max(0, value) });
  }
}

function handleWearInput(event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  if (!isNaN(value) && vehicle.value) {
    store.updateVehicle(vehicle.value.uniqueId, { wear: Math.max(0, Math.min(100, value)) / 100 });
  }
}

function handleDamageInput(event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  if (!isNaN(value) && vehicle.value) {
    store.updateVehicle(vehicle.value.uniqueId, { damage: Math.max(0, Math.min(100, value)) / 100 });
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function handlePropertyStateChange(value: any) {
  if (vehicle.value) {
    store.updateVehicle(vehicle.value.uniqueId, { propertyState: String(value) as "None" | "Owned" | "Rented" | "Mission" });
  }
}

function handleDeleteVehicle() {
  if (vehicle.value) {
    store.deleteVehicle(vehicle.value.uniqueId);
    router.push("/editor/vehicles");
  }
}

function handleResetAge() {
  if (vehicle.value) {
    store.resetVehicleAge(vehicle.value.uniqueId);
  }
}

function handleFillAll() {
  if (vehicle.value) {
    store.fillAllTanks(vehicle.value.uniqueId);
  }
}

function handleEmptyAll() {
  if (vehicle.value) {
    store.emptyAllTanks(vehicle.value.uniqueId);
  }
}

function handleFillLevelUpdate(unitIndex: number, level: number) {
  if (vehicle.value) {
    store.updateFillLevel(vehicle.value.uniqueId, unitIndex, level);
  }
}

function handlePositionInput(axis: "x" | "y" | "z", event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  if (!isNaN(value) && vehicle.value && vehicle.value.position) {
    const newPosition = { ...vehicle.value.position, [axis]: value };
    store.updateVehicle(vehicle.value.uniqueId, { position: newPosition });
  }
}

function handleRotationInput(axis: "x" | "y" | "z", event: Event) {
  const value = parseFloat((event.target as HTMLInputElement).value);
  if (!isNaN(value) && vehicle.value && vehicle.value.rotation) {
    const newRotation = { ...vehicle.value.rotation, [axis]: value };
    store.updateVehicle(vehicle.value.uniqueId, { rotation: newRotation });
  }
}
</script>

<template>
  <div class="space-y-6">
    <!-- Header with back button -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <Button variant="ghost" size="icon" @click="goBack">
          <ArrowLeft class="size-5" />
        </Button>
        <div v-if="vehicle" class="flex items-center gap-3">
          <VehicleImage v-if="settings.gamePath" :filename="vehicle.filename" size="lg" />
          <div>
            <h2 class="text-2xl font-semibold">{{ vehicle.displayName }}</h2>
            <div class="flex items-center gap-2 text-sm text-muted-foreground">
            <span>{{ t(`vehicleTypes.${vehicleType(vehicle.filename)}`) }}</span>
            <span>&middot;</span>
            <Badge
              :variant="vehicle.propertyState === 'Owned' ? 'default' : vehicle.propertyState === 'Rented' ? 'secondary' : 'outline'"
            >
              {{ t(`propertyStates.${vehicle.propertyState}`) }}
            </Badge>
          </div>
          </div>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <Label for="advanced-mode" class="text-sm">{{ t("common.advancedMode") }}</Label>
        <Switch id="advanced-mode" :model-value="settings.advancedMode" @update:model-value="settings.setAdvancedMode($event)" />
      </div>
    </div>

    <!-- Vehicle not found -->
    <Alert v-if="!vehicle" variant="destructive">
      <AlertTriangle class="size-4" />
      <AlertDescription>
        {{ t("vehicle.notFound") }}
      </AlertDescription>
    </Alert>

    <template v-if="vehicle">
      <!-- General information -->
      <Card>
        <CardHeader>
          <CardTitle class="text-base">{{ t("vehicle.generalInfo") }}</CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <!-- Price -->
            <div class="space-y-2" :class="modifiedClass('price')">
              <Label>{{ t("vehicle.purchasePrice") }}</Label>
              <div class="flex items-center gap-2">
                <Input
                  type="number"
                  :model-value="vehicle.price"
                  @input="handlePriceInput"
                  class="font-mono"
                  min="0"
                  step="1000"
                />
                <span class="shrink-0 text-sm text-muted-foreground">
                  {{ formatMoney(vehicle.price) }} $
                </span>
              </div>
            </div>

            <!-- Age -->
            <div class="space-y-2" :class="modifiedClass('age')">
              <Label>{{ t("vehicle.ageDays") }}</Label>
              <div class="flex items-center gap-2">
                <Input
                  type="number"
                  :model-value="vehicle.age"
                  @input="handleAgeInput"
                  class="font-mono"
                  min="0"
                  step="1"
                />
                <Button variant="outline" size="sm" @click="handleResetAge">
                  <RotateCcw class="size-4" />
                  {{ t("vehicle.resetAge") }}
                </Button>
              </div>
            </div>

            <!-- Operating time -->
            <div class="space-y-2" :class="modifiedClass('operatingTime')">
              <Label>{{ t("vehicle.operatingTime") }}</Label>
              <Input
                type="number"
                :model-value="vehicle.operatingTime"
                @input="handleOperatingTimeInput"
                class="font-mono"
                min="0"
                step="0.1"
              />
            </div>

            <!-- Wear -->
            <div class="space-y-2" :class="modifiedClass('wear')">
              <Label>{{ t("vehicle.wear") }} (%)</Label>
              <Input
                type="number"
                :model-value="Math.round(vehicle.wear * 100)"
                @input="handleWearInput"
                class="font-mono"
                min="0"
                max="100"
                step="1"
              />
            </div>

            <!-- Damage -->
            <div class="space-y-2" :class="modifiedClass('damage')">
              <Label>{{ t("vehicle.damage") }} (%)</Label>
              <Input
                type="number"
                :model-value="Math.round(vehicle.damage * 100)"
                @input="handleDamageInput"
                class="font-mono"
                min="0"
                max="100"
                step="1"
              />
            </div>

            <!-- Property state -->
            <div class="space-y-2" :class="modifiedClass('propertyState')">
              <Label>{{ t("vehicle.propertyState") }}</Label>
              <Select
                :model-value="vehicle.propertyState"
                @update:model-value="handlePropertyStateChange"
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Owned">{{ t("propertyStates.Owned") }}</SelectItem>
                  <SelectItem value="Rented">{{ t("propertyStates.Rented") }}</SelectItem>
                  <SelectItem value="Mission">{{ t("propertyStates.Mission") }}</SelectItem>
                  <SelectItem value="None">{{ t("propertyStates.None") }}</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- Fill levels -->
      <Card v-if="vehicle.fillUnits.length > 0">
        <CardHeader>
          <div class="flex items-center justify-between">
            <CardTitle class="flex items-center gap-2 text-base">
              <Droplets class="size-5" />
              {{ t("vehicle.fillLevels") }}
            </CardTitle>
            <div class="flex gap-2">
              <Button variant="outline" size="sm" @click="handleFillAll">
                {{ t("vehicle.fillAll") }}
              </Button>
              <Button variant="outline" size="sm" @click="handleEmptyAll">
                {{ t("vehicle.emptyAll") }}
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent class="space-y-4">
          <FillLevelSlider
            v-for="unit in vehicle.fillUnits"
            :key="unit.index"
            :fill-type="unit.fillType"
            :fill-level="unit.fillLevel"
            :capacity="unit.capacity"
            @update:fill-level="handleFillLevelUpdate(unit.index, $event)"
          />
        </CardContent>
      </Card>

      <!-- Advanced: Position & Rotation -->
      <Collapsible v-if="settings.advancedMode && vehicle.position">
        <Card>
          <CardHeader class="py-3">
            <CollapsibleTrigger class="flex w-full items-center gap-2 text-left">
              <ChevronRight class="size-4 transition-transform [[data-state=open]>&]:rotate-90" />
              <CardTitle class="text-base">{{ t("vehicle.positionAndRotation") }}</CardTitle>
            </CollapsibleTrigger>
          </CardHeader>
          <CollapsibleContent>
            <CardContent class="space-y-4">
              <Alert>
                <AlertTriangle class="size-4" />
                <AlertDescription>
                  {{ t("vehicle.positionWarning") }}
                </AlertDescription>
              </Alert>

              <div class="space-y-3">
                <Label class="text-sm font-medium">{{ t("vehicle.position") }}</Label>
                <div class="grid grid-cols-3 gap-3">
                  <div class="space-y-1">
                    <Label class="text-xs text-muted-foreground">X</Label>
                    <Input
                      type="number"
                      :model-value="vehicle.position.x"
                      @input="handlePositionInput('x', $event)"
                      class="font-mono"
                      step="0.1"
                    />
                  </div>
                  <div class="space-y-1">
                    <Label class="text-xs text-muted-foreground">Y</Label>
                    <Input
                      type="number"
                      :model-value="vehicle.position.y"
                      @input="handlePositionInput('y', $event)"
                      class="font-mono"
                      step="0.1"
                    />
                  </div>
                  <div class="space-y-1">
                    <Label class="text-xs text-muted-foreground">Z</Label>
                    <Input
                      type="number"
                      :model-value="vehicle.position.z"
                      @input="handlePositionInput('z', $event)"
                      class="font-mono"
                      step="0.1"
                    />
                  </div>
                </div>
              </div>

              <div v-if="vehicle.rotation" class="space-y-3">
                <Label class="text-sm font-medium">{{ t("vehicle.rotation") }}</Label>
                <div class="grid grid-cols-3 gap-3">
                  <div class="space-y-1">
                    <Label class="text-xs text-muted-foreground">X</Label>
                    <Input
                      type="number"
                      :model-value="vehicle.rotation.x"
                      @input="handleRotationInput('x', $event)"
                      class="font-mono"
                      step="0.01"
                    />
                  </div>
                  <div class="space-y-1">
                    <Label class="text-xs text-muted-foreground">Y</Label>
                    <Input
                      type="number"
                      :model-value="vehicle.rotation.y"
                      @input="handleRotationInput('y', $event)"
                      class="font-mono"
                      step="0.01"
                    />
                  </div>
                  <div class="space-y-1">
                    <Label class="text-xs text-muted-foreground">Z</Label>
                    <Input
                      type="number"
                      :model-value="vehicle.rotation.z"
                      @input="handleRotationInput('z', $event)"
                      class="font-mono"
                      step="0.01"
                    />
                  </div>
                </div>
              </div>
            </CardContent>
          </CollapsibleContent>
        </Card>
      </Collapsible>

      <!-- Advanced: Configurations -->
      <Collapsible v-if="settings.advancedMode && vehicle.configurations.length > 0">
        <Card>
          <CardHeader class="py-3">
            <CollapsibleTrigger class="flex w-full items-center gap-2 text-left">
              <ChevronRight class="size-4 transition-transform [[data-state=open]>&]:rotate-90" />
              <CardTitle class="text-base">
                {{ t("vehicle.configuration") }} ({{ vehicle.configurations.length }})
              </CardTitle>
            </CollapsibleTrigger>
          </CardHeader>
          <CollapsibleContent>
            <CardContent>
              <Table>
                <TableBody>
                  <TableRow
                    v-for="config in vehicle.configurations"
                    :key="config.name + config.id"
                  >
                    <TableCell class="font-medium">{{ config.name }}</TableCell>
                    <TableCell class="text-right font-mono text-muted-foreground">
                      {{ config.id }}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </CardContent>
          </CollapsibleContent>
        </Card>
      </Collapsible>

      <!-- Advanced: Attached implements -->
      <Collapsible v-if="settings.advancedMode && vehicle.attachedImplements.length > 0">
        <Card>
          <CardHeader class="py-3">
            <CollapsibleTrigger class="flex w-full items-center gap-2 text-left">
              <ChevronRight class="size-4 transition-transform [[data-state=open]>&]:rotate-90" />
              <CardTitle class="text-base">
                {{ t("vehicle.attachments") }} ({{ vehicle.attachedImplements.length }})
              </CardTitle>
            </CollapsibleTrigger>
          </CardHeader>
          <CollapsibleContent>
            <CardContent>
              <Table>
                <TableBody>
                  <TableRow
                    v-for="impl in vehicle.attachedImplements"
                    :key="impl.jointIndex"
                  >
                    <TableCell>
                      <div class="flex items-center gap-2">
                        <Info class="size-4 text-muted-foreground" />
                        <span>{{ t("vehicle.joint", { index: impl.jointIndex }) }}</span>
                      </div>
                    </TableCell>
                    <TableCell>
                      {{ t("vehicle.vehicleId", { id: impl.attachedVehicleUniqueId }) }}
                      <template v-if="store.getVehicleById(impl.attachedVehicleUniqueId)">
                        ({{ store.getVehicleById(impl.attachedVehicleUniqueId)!.displayName }})
                      </template>
                    </TableCell>
                    <TableCell class="text-right">
                      <Badge v-if="impl.moveDown" variant="secondary">{{ t("vehicle.lowered") }}</Badge>
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </CardContent>
          </CollapsibleContent>
        </Card>
      </Collapsible>

      <!-- Delete vehicle -->
      <Card class="border-destructive/50">
        <CardContent class="pt-6">
          <AlertDialog v-model:open="showDeleteDialog">
            <AlertDialogTrigger as-child>
              <Button variant="destructive">
                <Trash2 class="size-4" />
                {{ t("vehicle.deleteVehicle") }}
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>{{ t("vehicle.deleteTitle") }}</AlertDialogTitle>
                <AlertDialogDescription>
                  {{ t("vehicle.deleteDesc", { name: vehicle.displayName }) }}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>{{ t("common.cancel") }}</AlertDialogCancel>
                <AlertDialogAction @click="handleDeleteVehicle">
                  {{ t("common.delete") }}
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </CardContent>
      </Card>
    </template>
  </div>
</template>
