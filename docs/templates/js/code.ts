import { type VNode, type RenderFunction } from "vue";
import { useClipboard, useTimeoutFn } from "@vueuse/core";
import {
  useTemplateRef,
  defineComponent,
  useSlots,
  onMounted,
  onBeforeMount,
  h,
} from "vue/dist/vue.esm-browser.js";

const map_title_to_content = (
  titleTag: string,
  contentTags: string[],
  vnodes: VNode[],
) => {
  // remove artifacts
  let filtered: VNode[] = vnodes.filter((el: VNode) => {
    return ![Symbol("v-txt").toString()].includes(el.type.toString());
  });

  let res: { vnode: VNode; name: string }[] = [];
  for (const [i, node] of filtered.entries()) {
    if (contentTags.includes(node.type.toString())) {
      const prevnode = filtered.at(i - 1);
      let name;
      if (prevnode.type == titleTag) {
        name = prevnode.children.toString();
        name = name.replace("[", "").replace("]", "");
      }
      res.push({ vnode: node, name });
    }
  }
  return res;
};
/* Wrap markdown code blocks
 * Create a tab like https://daisyui.com/components/tab
 * With the input trick.
 */
const wrap_code = (slot: VNode[]): RenderFunction => {
  // group unique identifier
  const uuid = Math.random().toString(36).substring(2);

  let mapped = map_title_to_content("p", ["pre"], slot);

  // modify nodes
  let code_blocks = mapped.map(({ vnode, name }) => {
    const lang = vnode.props!["data-lang"];

    const label = name ? name : lang;
    const tab: VNode = h("input", {
      class: "tab",
      type: "radio",
      name: uuid,
      "aria-label": label,
    });
    const content: VNode = h("div", { class: ["code", "tab-content"] }, vnode);
    return [tab, content];
  });
  // Select first block
  code_blocks.at(0).at(0).props.checked = "checked";

  const res = h("div", { class: ["codeGroup tabs"] }, code_blocks);

  console.info("[petite-vue]: LOG - succesfully wrapped code blocks.");
  return res;
};

const codegroup = defineComponent(() => {
  const slot = useSlots().default();
  let group = wrap_code(slot);
  return () => group;
});

const wrap_details = (slot: VNode[]): RenderFunction => {
  let { name, vnode } = map_title_to_content("p", ["div", "pre"], slot).shift();
  const input: VNode = h("input", {
    type: "checkbox",
  });
  const title: VNode = h(
    "div",
    {
      class: ["collapse-title", "font-bold"],
    },
    name ? name : "details",
  );
  const content: VNode = h("div", { class: ["collapse-content"] }, vnode);
  const res = h("div", { class: ["collapse collapse-arrow"] }, [
    input,
    title,
    content,
  ]);
  console.info("[petite-vue]: LOG - succesfully wrapped details.");
  return res;
};

const vdetails = defineComponent(() => {
  const slot = useSlots().default();
  let details = wrap_details(slot);
  return () => details;
});

/*
 * Add a copy button on top of code blocks.
 */
const code = defineComponent(() => {
  onMounted(() => {
    let code: NodeList = document.querySelectorAll(`pre[class^="language-"]`);

    console.info("[petite-vue]: LOG - adding code block copy button.");

    code.forEach((e) => {
      // Add copy button
      let copy_button = document.createElement("button");
      copy_button.classList.add("copy_btn");
      copy_button.onclick = (event: Event) => {
        const { copy, copied } = useClipboard();
        copy(e.textContent);
        if (copied) {
          event.target.classList.add(...["success"]);
          const { start } = useTimeoutFn(() => {
            event.target.classList.remove("success");
          }, 3000);
          start();
        }
      };
      e.appendChild(copy_button);
    });
  });
  console.info("[petite-vue]: LOG - [success] added code block copy button.");
  return () =>
    h("div", {
      class: "code",
      ref: "code",
    });
});

const side_by_side = defineComponent(() => {
  const slot = useSlots().default();
  const template = useTemplateRef("sbs");
  // remove artifacts
  return () =>
    h("div", { class: ["flex flex-col lg:flex-row"], ref: "sbs" }, slot);
});

/*
 * bug fix: make vue ignore inner elements
 */
const escape_code_blocks = () => {
  let code: NodeList = document.querySelectorAll(`pre[class^="language-"]`);
  console.info("[petite-vue]: LOG - escaping code blocks.");

  code.forEach((e) => {
    e.setAttribute("v-pre", "");
  });

  console.info("[petite-vue]: LOG - [success] escaped code blocks.");
};

export { codegroup, vdetails, code, side_by_side, escape_code_blocks };
