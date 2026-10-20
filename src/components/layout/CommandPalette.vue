<script setup lang="ts">
import { computed, ref, watch, type Component } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import {
  Save,
  Undo2,
  RotateCcw,
  SunMoon,
  FlaskConical,
  FolderOpen,
  Wheat,
  Tractor,
  type LucideIcon,
} from "@lucide/vue";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandShortcut,
} from "@/components/ui/command";
import { Kbd, KbdGroup } from "@/components/ui/kbd";
import { NAV_ITEMS } from "@/lib/navigation";
import { useFieldStore } from "@/stores/field";
import { useVehicleStore } from "@/stores/vehicle";
import { useSettingsStore } from "@/stores/settings";
import { vehicleDisplayName } from "@/lib/utils";

export interface PaletteActions {
  save: () => void;
  discard: () => void;
  reload: () => void;
  canSave: boolean;
}

const props = defineProps<{ actions: PaletteActions }>();
const open = defineModel<boolean>("open", { required: true });

const { t } = useI18n();
const router = useRouter();
const fields = useFieldStore();
const vehicles = useVehicleStore();
const settings = useSettingsStore();

type Group = "actions" | "pages" | "fields" | "vehicles";

interface PaletteCommand {
  id: string;
  group: Group;
  label: string;
  hint?: string;
  icon: LucideIcon | Component;
  shortcut?: string;
  run: () => void;
}

/** What is typed in the search field: fields and vehicles only show once searching. */
const query = ref("");
watch(open, (isOpen) => {
  if (isOpen) query.value = "";
});

function fruitName(name: string | null | undefined): string {
  if (!name) return t("field.none");
  const unknown = name.match(/^UNKNOWN_(\d+)$/);
  return unknown ? t("field.unknownFruit", { index: unknown[1] }) : t(`fillTypes.${name}`, name);
}

const commands = computed<PaletteCommand[]>(() => {
  const list: PaletteCommand[] = [];
  if (props.actions.canSave) {
    list.push(
      { id: "save", group: "actions", label: t("common.save"), icon: Save, shortcut: "Ctrl S", run: props.actions.save },
      { id: "discard", group: "actions", label: t("dirtyTracking.discardChanges"), icon: Undo2, shortcut: "Ctrl Z", run: props.actions.discard },
      { id: "reload", group: "actions", label: t("dirtyTracking.reloadFromDisk"), icon: RotateCcw, run: props.actions.reload },
    );
  }
  const dark = document.documentElement.classList.contains("dark");
  list.push(
    {
      id: "theme",
      group: "actions",
      label: dark ? t("palette.lightTheme") : t("palette.darkTheme"),
      icon: SunMoon,
      run: () => settings.setTheme(dark ? "light" : "dark"),
    },
    {
      id: "advanced",
      group: "actions",
      label: settings.advancedMode ? t("palette.advancedOff") : t("palette.advancedOn"),
      icon: FlaskConical,
      run: () => settings.toggleAdvancedMode(),
    },
    { id: "home", group: "actions", label: t("palette.otherSave"), icon: FolderOpen, run: () => router.push("/") },
  );
  for (const item of NAV_ITEMS) {
    list.push({
      id: "page:" + item.route,
      group: "pages",
      label: t(item.labelKey),
      icon: item.icon,
      run: () => router.push(item.route),
    });
  }
  if (!query.value.trim()) return list;
  for (const field of fields.fields) {
    const density = fields.getFieldDensity(field.id);
    const crop = density ? fruitName(density.dominantFruit) : fruitName(field.fruitType === "UNKNOWN" ? null : field.fruitType);
    list.push({
      id: "field:" + field.id,
      group: "fields",
      label: t("palette.field", { id: field.id }),
      hint: crop,
      icon: Wheat,
      run: () => router.push({ path: "/editor/fields", query: { field: String(field.id) } }),
    });
  }
  for (const v of vehicles.playerVehicles) {
    list.push({
      id: "vehicle:" + v.uniqueId,
      group: "vehicles",
      label: v.displayName || vehicleDisplayName(v.filename),
      icon: Tractor,
      run: () => router.push(`/editor/vehicles/${v.uniqueId}`),
    });
  }
  return list;
});

const groups = computed(() =>
  (["actions", "pages", "fields", "vehicles"] as Group[])
    .map((key) => ({ key, title: t(`palette.group.${key}`), items: commands.value.filter((c) => c.group === key) }))
    .filter((g) => g.items.length > 0),
);

function runCommand(c: PaletteCommand) {
  open.value = false;
  c.run();
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent
      :show-close-button="false"
      class="top-[12vh] w-[min(640px,calc(100%-2rem))] max-w-none translate-y-0 gap-0 overflow-hidden rounded-[20px] p-0 duration-300 sm:max-w-none"
    >
      <DialogTitle class="sr-only">{{ t("palette.title") }}</DialogTitle>
      <DialogDescription class="sr-only">{{ t("palette.description") }}</DialogDescription>
      <Command class="rounded-none bg-transparent">
        <CommandInput
          :placeholder="t('palette.placeholder')"
          class="text-base"
          @update:model-value="query = String($event ?? '')"
        />
        <CommandList class="max-h-[min(460px,60vh)] p-1.5">
          <CommandEmpty class="py-7 text-muted-foreground">
            {{ t("palette.empty", { query }) }}
          </CommandEmpty>
          <CommandGroup
            v-for="group in groups"
            :key="group.key"
            :heading="group.title"
            class="[&_[data-slot=command-group-heading]]:font-semibold"
          >
            <CommandItem
              v-for="c in group.items"
              :key="c.id"
              :value="c.id"
              class="group min-h-11 gap-3 rounded-[11px] px-3 data-[highlighted]:bg-primary-soft data-[highlighted]:text-foreground"
              @select="runCommand(c)"
            >
              <span
                class="inline-flex size-7 shrink-0 items-center justify-center rounded-lg bg-hover transition-colors group-data-[highlighted]:bg-primary group-data-[highlighted]:text-primary-foreground"
                aria-hidden="true"
              >
                <component :is="c.icon" class="size-[15px] text-current" />
              </span>
              <span class="flex min-w-0 flex-1 flex-col leading-tight">
                <span class="truncate font-medium">{{ c.label }}</span>
                <span v-if="c.hint" class="truncate text-xs text-muted-foreground">{{ c.hint }}</span>
              </span>
              <CommandShortcut v-if="c.shortcut" class="tracking-normal">
                <Kbd>{{ c.shortcut }}</Kbd>
              </CommandShortcut>
            </CommandItem>
          </CommandGroup>
        </CommandList>
      </Command>
      <div class="flex flex-wrap items-center gap-4 border-t px-4 py-2.5 text-xs text-muted-foreground">
        <span class="inline-flex items-center gap-1.5"><KbdGroup><Kbd>↑</Kbd><Kbd>↓</Kbd></KbdGroup> {{ t("palette.navigate") }}</span>
        <span class="inline-flex items-center gap-1.5"><Kbd>{{ t("palette.keyEnter") }}</Kbd> {{ t("palette.open") }}</span>
        <span class="inline-flex items-center gap-1.5"><Kbd>{{ t("palette.keyEsc") }}</Kbd> {{ t("window.close") }}</span>
        <span class="ml-auto inline-flex items-center gap-1.5"><Kbd>Ctrl K</Kbd> {{ t("palette.anywhere") }}</span>
      </div>
    </DialogContent>
  </Dialog>
</template>
