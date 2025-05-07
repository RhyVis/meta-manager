import AppLayout from "@/layout/AppLayout.vue";
import type { RouteRecordRaw } from "vue-router";

const records: RouteRecordRaw[] = [
  {
    path: "/",
    component: AppLayout,
    children: [
      {
        path: "",
        name: "home",
        component: () => import("@/pages/base/HomePage.vue"),
      },
      {
        path: "dashboard",
        name: "dashboard",
        component: () => import("@/pages/manage/dashboard/index.vue"),
      },
      {
        path: "config",
        name: "config",
        component: () => import("@/pages/base/ConfigPage.vue"),
      },
    ],
  },
];

export default records;
