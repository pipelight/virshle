import { type VNode, type RenderFunction, watchEffect } from "vue";

import {
  useTemplateRef,
  defineComponent,
  useAttrs,
  h,
} from "vue/dist/vue.esm-browser.js";

import { useImage } from "@vueuse/core";

const vlazy = defineComponent(() => {
  const template = useTemplateRef("vlazy");

  const { low, high } = useAttrs();

  const { isLoading: isSmLoading } = useImage({ src: low });
  const { isLoading: isXlLoading } = useImage({ src: high });

  watchEffect(() => {
    if (!isXlLoading.value) {
      template.value.classList.add("fadeIn");
      template.value.style["background-image"] = [
        `url("${high}")`,
        `url("${low}")`,
      ];
    } else if (!isSmLoading.value) {
      template.value.style["background-image"] = `url("${low}")`;
    }
  });

  return () => h("div", { class: "vlazy", ref: "vlazy" });
});

export { vlazy };
