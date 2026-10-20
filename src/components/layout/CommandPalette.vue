<script setup lang="ts">
import { computed, nextTick, ref, watch, type Component } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import {
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from "reka-ui";
import {
  Search,
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

interface Command {
  id: string;
  group: "actions" | "pages" | "fields" | "vehicles";
  label: string;
  hint?: string;
  icon: LucideIcon | Component;
  shortcut?: string;
  run: () => void;
}

const query = ref("");
const active = ref(0);
const listEl = ref<HTMLElement | null>(null);

function fruitName(name: string | null | undefined): string {
  if (!name) return t("field.none");
  const unknown = name.match(/^UNKNOWN_(\d+)$/);
  return unknown ? t("field.unknownFruit", { index: unknown[1] }) : t(`fillTypes.${name}`, name);
}

const commands = computed<Command[]>(() => {
  const list: Command[] = [];
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

const normalize = (s: string) => s.toLowerCase().normalize("NFD").replace(/[̀-ͯ]/g, "");

const results = computed(() => {
  const q = normalize(query.value.trim());
  const hits = q
    ? commands.value.filter((c) => normalize(`${c.label} ${c.hint ?? ""}`).includes(q))
    : commands.value.filter((c) => c.group === "actions" || c.group === "pages");
  // Keep each group short so every group stays reachable
  const perGroup: Record<string, number> = {};
  return hits.filter((c) => (perGroup[c.group] = (perGroup[c.group] ?? 0) + 1) <= (q ? 8 : 20));
});

const groups = computed(() => {
  const order: Command["group"][] = ["actions", "pages", "fields", "vehicles"];
  return order
    .map((g) => ({
      key: g,
      title: t(`palette.group.${g}`),
      items: results.value.map((c, index) => ({ c, index })).filter(({ c }) => c.group === g),
    }))
    .filter((g) => g.items.length > 0);
});

watch(query, () => (active.value = 0));
watch(open, (isOpen) => {
  if (isOpen) {
    query.value = "";
    active.value = 0;
  }
});

function scrollActiveIntoView() {
  nextTick(() => {
    listEl.value?.querySelector(`#palette-opt-${active.value}`)?.scrollIntoView({ block: "nearest" });
  });
}

function onKeydown(e: KeyboardEvent) {
  const count = results.value.length;
  if (e.key === "ArrowDown" && count) {
    e.preventDefault();
    active.value = (active.value + 1) % count;
    scrollActiveIntoView();
  } else if (e.key === "ArrowUp" && count) {
    e.preventDefault();
    active.value = (active.value - 1 + count) % count;
    scrollActiveIntoView();
  } else if (e.key === "Enter" && count) {
    e.preventDefault();
    runCommand(results.value[active.value]);
  }
}

function runCommand(c: Command | undefined) {
  if (!c) return;
  open.value = false;
  c.run();
}
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay
        class="fixed inset-0 z-50 bg-black/30 backdrop-blur-[6px] data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0"
      />
      <DialogContent
        class="glass-strong fixed left-1/2 top-[12vh] z-50 flex w-[min(640px,calc(100%-2rem))] -translate-x-1/2 flex-col overflow-hidden rounded-[20px] duration-300 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-[0.97] data-[state=open]:zoom-in-[0.97] data-[state=open]:slide-in-from-top-3"
        @keydown="onKeydown"
      >
        <DialogTitle class="sr-only">{{ t("palette.title") }}</DialogTitle>
        <DialogDescription class="sr-only">{{ t("palette.description") }}</DialogDescription>
        <div class="flex h-[60px] items-center gap-3 border-b px-[18px]">
          <Search class="size-[18px] shrink-0 text-muted-foreground" aria-hidden="true" />
          <input
            v-model="query"
            type="text"
            role="combobox"
            aria-expanded="true"
            aria-controls="palette-list"
            aria-autocomplete="list"
            :aria-activedescendant="results.length ? `palette-opt-${active}` : undefined"
            :aria-label="t('palette.placeholder')"
            :placeholder="t('palette.placeholder')"
            autocomplete="off"
            spellcheck="false"
            class="h-11 min-w-0 flex-1 bg-transparent text-[17px] outline-none placeholder:text-muted-foreground"
          />
          <kbd class="rounded-md border px-1.5 py-0.5 text-xs text-muted-foreground">{{ t("palette.keyEsc") }}</kbd>
        </div>

        <div ref="listEl" class="max-h-[min(460px,60vh)] overflow-y-auto p-2">
          <ul id="palette-list" role="listbox" :aria-label="t('palette.results')">
            <li v-for="group in groups" :key="group.key" role="presentation">
              <div role="presentation" class="px-3 pb-1 pt-2.5 text-xs font-semibold text-muted-foreground">
                {{ group.title }}
              </div>
              <ul role="group" :aria-label="group.title">
                <li
                  v-for="{ c, index } in group.items"
                  :id="`palette-opt-${index}`"
                  :key="c.id"
                  role="option"
                  :aria-selected="index === active"
                  class="relative flex min-h-11 cursor-pointer items-center gap-3 rounded-[11px] px-3"
                  @mousemove="active = index"
                  @click="runCommand(c)"
                >
                  <span
                    v-if="index === active"
                    class="absolute inset-0 rounded-[11px] bg-primary-soft"
                    aria-hidden="true"
                  />
                  <span
                    class="relative inline-flex size-7 shrink-0 items-center justify-center rounded-lg transition-colors"
                    :class="index === active ? 'bg-primary text-primary-foreground' : 'bg-hover text-foreground'"
                    aria-hidden="true"
                  >
                    <component :is="c.icon" class="size-[15px]" />
                  </span>
                  <span class="relative flex min-w-0 flex-1 flex-col leading-tight">
                    <span class="truncate font-medium">{{ c.label }}</span>
                    <span v-if="c.hint" class="truncate text-xs text-muted-foreground">{{ c.hint }}</span>
                  </span>
                  <kbd
                    v-if="c.shortcut"
                    class="relative rounded-md border px-1.5 py-0.5 text-xs text-muted-foreground"
                  >{{ c.shortcut }}</kbd>
                </li>
              </ul>
            </li>
          </ul>
          <p
            v-if="results.length === 0"
            role="status"
            class="px-3 py-7 text-center text-muted-foreground"
          >
            {{ t("palette.empty", { query }) }}
          </p>
        </div>

        <div class="flex flex-wrap items-center gap-4 border-t px-[18px] py-2.5 text-xs text-muted-foreground">
          <span><kbd class="font-semibold">↑ ↓</kbd> {{ t("palette.navigate") }}</span>
          <span><kbd class="font-semibold">{{ t("palette.keyEnter") }}</kbd> {{ t("palette.open") }}</span>
          <span><kbd class="font-semibold">Ctrl K</kbd> {{ t("palette.anywhere") }}</span>
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
