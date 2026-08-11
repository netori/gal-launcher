import { createApp } from "vue";
import App from "./App.vue";
import reveal from "./directives/reveal";
import "./style/tokens.css";
import "./style/base.css";
import "./style/buttons.css";
import "./style/reveal.css";

createApp(App).directive("reveal", reveal).mount("#app");
