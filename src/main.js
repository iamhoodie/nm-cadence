import "@fontsource/roboto/300.css";
import "@fontsource/roboto/400.css";
import "@fontsource/roboto/500.css";
import "@fontsource/roboto/700.css";
import "./app.css";
import App from "./App.svelte";
import { mount } from "svelte";

const app = mount(App, { target: document.getElementById("app") });

export default app;
