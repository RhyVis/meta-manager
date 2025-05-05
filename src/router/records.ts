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
    ],
  },
];

export default records;
