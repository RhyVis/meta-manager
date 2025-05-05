import App from "@/App.vue";
import router from "@/router";
import store from "@/stores";
import "@quasar/extras/material-icons/material-icons.css";
import { Notify, Dialog, Quasar } from "quasar";
import quasarLang from "quasar/lang/zh-CN";
import "quasar/src/css/index.sass";
import { createApp } from "vue";

console.log("Application Loaded");

const app = createApp(App);

app.use(Quasar, {
  plugins: {
    Notify,
    Dialog,
  },
  lang: quasarLang,
});
app.use(store);
app.use(router);

app.mount("#app");
