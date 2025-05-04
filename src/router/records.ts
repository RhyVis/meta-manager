import type { RouteRecordRaw } from "vue-router";
import ApplicationLayout from "@/layout/ApplicationLayout.vue";

const records: RouteRecordRaw[] = [
  {
    path: "/",
    component: ApplicationLayout,
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
