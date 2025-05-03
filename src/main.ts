import "@quasar/extras/material-icons/material-icons.css";
import "quasar/src/css/index.sass";

import { createApp } from "vue";

import quasarLang from "quasar/lang/zh-CN";

import App from "@/App.vue";
import router from "@/router";
import store from "@/stores";
import { Quasar } from "quasar";

console.log("Application Loaded");

const app = createApp(App);

app.use(Quasar, {
  lang: quasarLang,
});
app.use(store);
app.use(router);

app.mount("#app");
