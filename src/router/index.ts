import { createRouter, createWebHistory } from "vue-router";
import ApplicationLayout from "@/layout/ApplicationLayout.vue";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/",
      component: ApplicationLayout,
      children: [
        {
          path: "",
          name: "home",
          component: () => import("@/pages/base/HomePage.vue"),
        },
      ],
    },
  ],
});

export default router;
