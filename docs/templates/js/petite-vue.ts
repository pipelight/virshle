import {
  createApp,
  defineComponent,
  useSlots,
  useTemplateRef,
  onMounted,
  onBeforeMount,
  h,
} from "vue/dist/vue.esm-browser.js";

import { type VNode, type RenderFunction } from "vue";

/* Vue helpers */
import {
  // tailwindcss
  useBreakpoints,
  breakpointsTailwind,
  // viewport
  useWindowSize,
  useWindowScroll,
  useIntersectionObserver,
  // timing
  useThrottleFn,
  useDebounceFn,
  // events
  useEventListener,
} from "@vueuse/core";

/* Vue components */
import {
  codegroup,
  vdetails,
  code,
  side_by_side,
  escape_code_blocks,
} from "./code.ts";

const app = createApp();

escape_code_blocks();

import { vlazy } from "./image.ts";
app.component("vlazy", vlazy);

app.component("codegroup", codegroup);
app.component("vdetails", vdetails);
app.component("vcode", code);
// app.component("sbs", side_by_side);

import { related } from "./related.js";
app.component("related", related);

import { date } from "./date.ts";
app.component("date", date);

import { timeline } from "./timeline.ts";
app.component("timeline", timeline);

/*
 * Show popup when user scroll near bottom
 */
const popup = defineComponent(() => {
  const slot = useSlots().default();
  const template = useTemplateRef("popup");

  const breakpoints = useBreakpoints(breakpointsTailwind);

  const toggle = useDebounceFn(() => {
    const xl = breakpoints.greaterOrEqual("xl");
    const { x, y, arrivedState } = useWindowScroll();
    if (!xl.value) {
      template.value.style["z-index"] = -10;
      template.value.style.opacity = "0";
    } else {
      let height = document.querySelector("#app").clientHeight;
      if (y.value > height / 3 || arrivedState.bottom) {
        template.value.style["z-index"] = 10;
        template.value.style.opacity = "1";
      } else {
        template.value.style["z-index"] = -10;
        template.value.style.opacity = "0";
      }
    }
  }, 100);

  useEventListener(window, "scroll", toggle);
  return () => h("div", { ref: "popup" }, slot);
});

const progress = defineComponent(() => {
  if (CSS.supports("animation-timeline", "scroll")) {
    return () => h("div", { class: "progress" });
  } else {
    return () => h("div");
  }
});

const toc = defineComponent(() => {
  const slot = useSlots().default();
  const template = useTemplateRef("toc");

  const update_toc = useDebounceFn(() => {
    const toc_titles = template.value.querySelectorAll("a");
    const map_toc: Record<string, Node> = {};
    for (const [_, el] of toc_titles.entries()) {
      let path = new URL(el.href).hash.replace("#", "");
      map_toc[path] = el;
    }

    const page_titles = document.querySelectorAll(
      ".postContainer h1, .postContainer h2, .postContainer h3",
    );

    for (const [_, el] of page_titles.entries()) {
      useIntersectionObserver(el, ([entry], observerElement) => {
        const toc_title = map_toc[entry.target.id];
        if (entry?.isIntersecting) {
          toc_title.parentElement.classList.add("isVisible");
        } else {
          toc_title.parentElement.classList.remove("isVisible");
        }
      });
    }
  }, 100);

  onBeforeMount(() => {
    update_toc();
  });

  console.info("[petite-vue]: LOG - loaded dynamyc table of content.");
  return () => h("div", { ref: "toc" }, slot);
});

app.component("popup", popup);
app.component("toc", toc);
app.component("vprogress", progress);

app.mount("#app");
