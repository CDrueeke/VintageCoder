// SPDX-License-Identifier: GPL-3.0-or-later
import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";

const target = document.getElementById("app");
if (!target) throw new Error("#app is missing");
const vo = new URLSearchParams(location.search).get("vo-test");
if (vo) { void import("./lib/votest").then((m) => m.voTest(["a", "b", "c"].includes(vo) ? vo : "a")); }
export default vo ? {} : mount(App, { target });
