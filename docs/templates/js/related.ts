import { type VNode, type RenderFunction } from "vue";

import {
  useTemplateRef,
  defineComponent,
  useSlots,
  onMounted,
  onBeforeMount,
  h,
} from "vue/dist/vue.esm-browser.js";

/*
 * Generate the previous / next buttons
 * Returns only 2 thumbnails from the availabe thumbnail pool.
 */
const related = defineComponent(() => {
  let slots = useSlots();
  if (slots.default === undefined) {
    return () => h("div");
  }
  const slot = useSlots().default();
  // remove artifacts
  let filtered: VNode[] = slot.filter((el: VNode) => {
    return ![Symbol("v-txt").toString()].includes(el.type.toString());
  });

  if (filtered.length < 2) {
    return () => h("div", null, filtered);
  } else {
    const prev = Math.floor(Math.random() * filtered.length);
    let next = Math.floor(Math.random() * filtered.length);
    while (prev === next) {
      next = Math.floor(Math.random() * filtered.length);
    }
    let reduced = [filtered[prev], filtered[next]];
    return () => reduced;
  }
});

export { related };
