<script setup lang="ts">
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
  SidebarSeparator,
} from "@/components/ui/sidebar";
import { Badge } from "@/components/ui/badge";
import {
  Banknote,
  Tractor,
  ShoppingCart,
  Wheat,
  Cloud,
  Building2,
  ClipboardList,
  Trophy,
  TrendingUp,
  Archive,
  Settings,
  type LucideIcon,
} from "lucide-vue-next";

interface NavItem {
  labelKey: string;
  icon: LucideIcon;
  route: string;
  disabled?: boolean;
}

const { t } = useI18n();
const route = useRoute();

const mainNav: NavItem[] = [
  { labelKey: "sidebar.finance", icon: Banknote, route: "/editor/finance" },
  { labelKey: "sidebar.vehicles", icon: Tractor, route: "/editor/vehicles" },
  { labelKey: "sidebar.sales", icon: ShoppingCart, route: "/editor/sales" },
];

const phase2Nav: NavItem[] = [
  { labelKey: "sidebar.fields", icon: Wheat, route: "/editor/fields" },
  { labelKey: "sidebar.world", icon: Cloud, route: "/editor/world" },
  { labelKey: "sidebar.buildings", icon: Building2, route: "/editor/buildings" },
  { labelKey: "sidebar.missions", icon: ClipboardList, route: "/editor/missions" },
  { labelKey: "sidebar.collectibles", icon: Trophy, route: "/editor/collectibles" },
  { labelKey: "sidebar.economy", icon: TrendingUp, route: "/editor/economy" },
];

const utilNav: NavItem[] = [
  { labelKey: "sidebar.backups", icon: Archive, route: "/editor/backups" },
  { labelKey: "sidebar.settings", icon: Settings, route: "/editor/settings" },
];

function isActive(itemRoute: string): boolean {
  return route.path === itemRoute || route.path.startsWith(itemRoute + "/");
}
</script>

<template>
  <Sidebar collapsible="icon">
    <SidebarHeader>
      <div class="flex items-center justify-center px-2 py-3">
        <img
          src="@/assets/logo.png"
          :alt="t('common.appName')"
          class="w-full max-w-[180px] h-auto group-data-[collapsible=icon]:max-w-[32px] dark:drop-shadow-[0_0_0.5px_rgba(255,255,255,0.8)] dark:brightness-110"
        />
      </div>
    </SidebarHeader>

    <SidebarContent class="overflow-x-hidden">
      <SidebarGroup>
        <SidebarGroupLabel>{{ t("sidebar.editor") }}</SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem v-for="item in mainNav" :key="item.route">
              <SidebarMenuButton
                as-child
                :is-active="isActive(item.route)"
              >
                <router-link :to="item.route">
                  <component :is="item.icon" />
                  <span>{{ t(item.labelKey) }}</span>
                </router-link>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>

      <SidebarSeparator />

      <SidebarGroup>
        <SidebarGroupLabel>{{ t("sidebar.advanced") }}</SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem v-for="item in phase2Nav" :key="item.route">
              <SidebarMenuButton
                v-if="!item.disabled"
                as-child
                :is-active="isActive(item.route)"
              >
                <router-link :to="item.route">
                  <component :is="item.icon" />
                  <span>{{ t(item.labelKey) }}</span>
                </router-link>
              </SidebarMenuButton>
              <SidebarMenuButton
                v-else
                disabled
                class="opacity-50"
              >
                <component :is="item.icon" />
                <span>{{ t(item.labelKey) }}</span>
                <Badge
                  variant="secondary"
                  class="ml-auto shrink-0 text-[10px] px-1.5 py-0 group-data-[collapsible=icon]:hidden"
                >
                  {{ t("sidebar.comingSoon") }}
                </Badge>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>

      <SidebarSeparator />

      <SidebarGroup>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem v-for="item in utilNav" :key="item.route">
              <SidebarMenuButton
                as-child
                :is-active="isActive(item.route)"
              >
                <router-link :to="item.route">
                  <component :is="item.icon" />
                  <span>{{ t(item.labelKey) }}</span>
                </router-link>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    </SidebarContent>

    <SidebarFooter>
      <div class="px-2 py-1 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden">
        v0.1.0
      </div>
    </SidebarFooter>

    <SidebarRail />
  </Sidebar>
</template>
