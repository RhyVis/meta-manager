import { createRouter, createWebHistory } from "vue-router";
import records from "@/router/records.ts";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: records,
});

export default router;
