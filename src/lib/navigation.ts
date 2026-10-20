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
  type LucideIcon,
} from "@lucide/vue";

export interface NavItem {
  labelKey: string;
  icon: LucideIcon;
  route: string;
}

export interface NavGroup {
  titleKey: string;
  items: NavItem[];
}

/** The editor's sections, grouped as the sidebar shows them. */
export const NAV_GROUPS: NavGroup[] = [
  {
    titleKey: "sidebar.groupFarm",
    items: [
      { labelKey: "sidebar.finance", icon: Banknote, route: "/editor/finance" },
      { labelKey: "sidebar.vehicles", icon: Tractor, route: "/editor/vehicles" },
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
];

export const UTIL_NAV: NavItem[] = [
  { labelKey: "sidebar.backups", icon: Archive, route: "/editor/backups" },
  { labelKey: "sidebar.settings", icon: Settings, route: "/editor/settings" },
];

export const NAV_ITEMS: NavItem[] = [...NAV_GROUPS.flatMap((g) => g.items), ...UTIL_NAV];
