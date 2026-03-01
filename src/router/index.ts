import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      name: "home",
      component: () => import("@/views/SavegameSelector.vue"),
    },
    {
      path: "/editor",
      component: () => import("@/components/layout/EditorLayout.vue"),
      redirect: "/editor/finance",
      children: [
        {
          path: "finance",
          name: "finance",
          component: () => import("@/views/FinanceView.vue"),
        },
        {
          path: "vehicles",
          name: "vehicles",
          component: () => import("@/views/VehicleListView.vue"),
        },
        {
          path: "vehicles/:id",
          name: "vehicle-detail",
          component: () => import("@/views/VehicleDetailView.vue"),
        },
        {
          path: "sales",
          name: "sales",
          component: () => import("@/views/SaleListView.vue"),
        },
        {
          path: "sales/add",
          name: "sale-add",
          component: () => import("@/views/SaleAddView.vue"),
        },
        {
          path: "fields",
          name: "fields",
          component: () => import("@/views/FieldListView.vue"),
        },
        {
          path: "world",
          name: "world",
          component: () => import("@/views/WorldView.vue"),
        },
        {
          path: "buildings",
          name: "buildings",
          component: () => import("@/views/BuildingListView.vue"),
        },
        {
          path: "missions",
          name: "missions",
          component: () => import("@/views/MissionListView.vue"),
        },
        {
          path: "collectibles",
          name: "collectibles",
          component: () => import("@/views/CollectibleView.vue"),
        },
        {
          path: "economy",
          name: "economy",
          component: () => import("@/views/EconomyView.vue"),
        },
        {
          path: "settings",
          name: "settings",
          component: () => import("@/views/SettingsView.vue"),
        },
        {
          path: "backups",
          name: "backups",
          component: () => import("@/views/BackupView.vue"),
        },
      ],
    },
  ],
});

export default router;
