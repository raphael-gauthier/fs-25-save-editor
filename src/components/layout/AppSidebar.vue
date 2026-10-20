<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";
import { useStorage } from "@vueuse/core";
import { motion } from "motion-v";
import {
  Banknote,
  Tractor,
  ShoppingCart,
  Wheat,
  LandPlot,
  Warehouse,
  SlidersHorizontal,
  Cloud,
  Building2,
  ClipboardList,
  Trophy,
  TrendingUp,
  Archive,
  Settings,
  PiggyBank,
  PanelLeftClose,
  PanelLeftOpen,
  type LucideIcon,
} from "@lucide/vue";
import { useFinanceStore } from "@/stores/finance";
import { useVehicleStore } from "@/stores/vehicle";
import { useSavegameStore } from "@/stores/savegame";
import { formatMoney } from "@/lib/utils";

interface NavItem {
  labelKey: string;
  icon: LucideIcon;
  route: string;
  count?: number;
}

const { t } = useI18n();
const route = useRoute();
const finance = useFinanceStore();
const vehicles = useVehicleStore();
const savegame = useSavegameStore();

const collapsed = useStorage("sidebar-collapsed", false);

const groups = computed<{ titleKey: string; items: NavItem[] }[]>(() => [
  {
    titleKey: "sidebar.groupFarm",
    items: [
      { labelKey: "sidebar.finance", icon: Banknote, route: "/editor/finance" },
      {
        labelKey: "sidebar.vehicles",
        icon: Tractor,
        route: "/editor/vehicles",
        count: vehicles.playerVehicles.length || undefined,
      },
      { labelKey: "sidebar.sales", icon: ShoppingCart, route: "/editor/sales" },
    ],
  },
  {
    titleKey: "sidebar.groupLand",
    items: [
      { labelKey: "sidebar.fields", icon: Wheat, route: "/editor/fields" },
      { labelKey: "sidebar.farmlands", icon: LandPlot, route: "/editor/farmlands" },
    ],
  },
  {
    titleKey: "sidebar.groupOperation",
    items: [
      { labelKey: "sidebar.buildings", icon: Building2, route: "/editor/buildings" },
      { labelKey: "sidebar.storage", icon: Warehouse, route: "/editor/storage" },
      { labelKey: "sidebar.animals", icon: PiggyBank, route: "/editor/animals" },
    ],
  },
  {
    titleKey: "sidebar.groupWorld",
    items: [
      { labelKey: "sidebar.world", icon: Cloud, route: "/editor/world" },
      { labelKey: "sidebar.game", icon: SlidersHorizontal, route: "/editor/game" },
      { labelKey: "sidebar.missions", icon: ClipboardList, route: "/editor/missions" },
      { labelKey: "sidebar.collectibles", icon: Trophy, route: "/editor/collectibles" },
      { labelKey: "sidebar.economy", icon: TrendingUp, route: "/editor/economy" },
    ],
  },
]);

const utilNav: NavItem[] = [
  { labelKey: "sidebar.backups", icon: Archive, route: "/editor/backups" },
  { labelKey: "sidebar.settings", icon: Settings, route: "/editor/settings" },
];

const farmName = computed(() => savegame.currentSavegame?.career.savegameName ?? "");

function isActive(itemRoute: string): boolean {
  return route.path === itemRoute || route.path.startsWith(itemRoute + "/");
}

const spring = { type: "spring", stiffness: 520, damping: 42, mass: 0.9 } as const;
</script>

<template>
  <motion.nav
    :aria-label="t('sidebar.navigation')"
    class="glass flex shrink-0 flex-col gap-3 overflow-hidden rounded-2xl px-2 py-3"
    :initial="false"
    :animate="{ width: collapsed ? 60 : 232 }"
    :transition="spring"
  >
    <div class="flex min-h-0 flex-1 flex-col gap-3.5 overflow-y-auto overflow-x-hidden">
      <div v-for="group in groups" :key="group.titleKey" class="flex flex-col gap-px">
        <span
          class="h-5 truncate px-2.5 text-xs font-semibold text-muted-foreground transition-opacity"
          :class="collapsed ? 'opacity-0' : 'opacity-100'"
          :aria-hidden="collapsed"
        >
          {{ t(group.titleKey) }}
        </span>
        <router-link
          v-for="item in group.items"
          :key="item.route"
          :to="item.route"
          :aria-current="isActive(item.route) ? 'page' : undefined"
          :title="collapsed ? t(item.labelKey) : undefined"
          class="relative flex h-9 items-center gap-2.5 rounded-[9px] px-2.5 text-sm transition-colors hover:bg-hover"
          :class="isActive(item.route) ? 'font-semibold text-primary-text' : 'font-medium text-foreground'"
        >
          <motion.span
            v-if="isActive(item.route)"
            layout-id="sidebar-active"
            class="absolute inset-0 rounded-[9px] bg-primary-soft"
            :transition="spring"
          />
          <component :is="item.icon" class="relative size-[17px] shrink-0" aria-hidden="true" />
          <span class="relative flex-1 truncate" :class="{ 'sr-only': collapsed }">{{ t(item.labelKey) }}</span>
          <span
            v-if="item.count && !collapsed"
            class="relative text-xs font-normal text-muted-foreground tabular"
          >
            {{ item.count }}
          </span>
        </router-link>
      </div>
    </div>

    <div class="flex flex-col gap-px border-t pt-2">
      <router-link
        v-for="item in utilNav"
        :key="item.route"
        :to="item.route"
        :aria-current="isActive(item.route) ? 'page' : undefined"
        :title="collapsed ? t(item.labelKey) : undefined"
        class="relative flex h-9 items-center gap-2.5 rounded-[9px] px-2.5 text-sm transition-colors hover:bg-hover"
        :class="isActive(item.route) ? 'font-semibold text-primary-text' : 'font-medium text-foreground'"
      >
        <motion.span
          v-if="isActive(item.route)"
          layout-id="sidebar-active"
          class="absolute inset-0 rounded-[9px] bg-primary-soft"
          :transition="spring"
        />
        <component :is="item.icon" class="relative size-[17px] shrink-0" aria-hidden="true" />
        <span class="relative flex-1 truncate" :class="{ 'sr-only': collapsed }">{{ t(item.labelKey) }}</span>
      </router-link>
    </div>

    <div class="flex items-center gap-2.5 rounded-xl bg-hover p-2">
      <span
        v-if="!collapsed"
        class="inline-flex size-8 shrink-0 items-center justify-center rounded-[9px] bg-primary-soft text-sm font-bold text-primary-text"
        aria-hidden="true"
      >
        {{ farmName.charAt(0).toUpperCase() || "F" }}
      </span>
      <span v-if="!collapsed" class="flex min-w-0 flex-1 flex-col leading-tight">
        <span class="truncate text-sm font-semibold">{{ farmName }}</span>
        <span class="truncate text-[13px] text-muted-foreground tabular">{{ formatMoney(finance.money) }}</span>
      </span>
      <button
        type="button"
        class="inline-flex size-8 shrink-0 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-hover hover:text-foreground"
        :aria-label="collapsed ? t('sidebar.expand') : t('sidebar.collapse')"
        :title="collapsed ? t('sidebar.expand') : t('sidebar.collapse')"
        :aria-expanded="!collapsed"
        @click="collapsed = !collapsed"
      >
        <PanelLeftOpen v-if="collapsed" class="size-4" aria-hidden="true" />
        <PanelLeftClose v-else class="size-4" aria-hidden="true" />
      </button>
    </div>
  </motion.nav>
</template>
