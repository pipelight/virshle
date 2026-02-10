import {
  type VNode,
  type RenderFunction,
} from "vue";

import {
  defineComponent,
  useSlots,
  onMounted,
  useTemplateRef,
  h
  // events
} from "vue/dist/vue.esm-browser.js";

/*
* Map list element title to date.
*/
const wrap_list = (vnodes: VNode[]): RenderFunction => {

  // remove artifacts
  let filtered: VNode[] = vnodes.filter((el: VNode) => {
    return ![Symbol("v-txt").toString()].includes(el.type.toString());
  }).filter((el: VNode) => {
    return ["ul"].includes(el.type.toString());
  });

  // Get the item list
  const items: VNode[] = [];
  const list = filtered.shift().children;
  for (let [i, node] of list.entries()) {
    // Get text
    const text = node.children.shift().children;
    const [titleText, descriptionText] = text.split(",")

    const title: VNode = h("div", { class: "timeline-start" }, titleText);
    const dot: VNode = h("div", { class: "timeline-middle icon check" });
    const description = h("div", { class: "timeline-end box" }, descriptionText);
    const bulkItem = [title, dot, description]

    if (i == 0) {
      bulkItem.push(h("hr"))
    }
    else if (i < list.length - 1) {
      bulkItem.unshift(h("hr"))
      bulkItem.push(h("hr"))
    }
    else {
      bulkItem.unshift(h("hr"))
    }

    const item = h("li", null, bulkItem);
    items.push(item);
  };

  const res = h("div", { class: "timelineContainer" },

    h("ul", {
      class: [
        "timeline"
      ]
    }, items));
  return res;
};

const timeline = defineComponent(() => {
  const slot = useSlots().default();
  // const template = useTemplateRef("timeline");
  let list = wrap_list(slot);
  return () => list;
})

export {
  timeline,
}
