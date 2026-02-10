import { defineComponent, useAttrs, h } from "vue/dist/vue.esm-browser.js"

import { parse, format, formatDistanceToNowStrict } from "date-fns";

const date = defineComponent(() => {
  const { date, since } = useAttrs();
  // zola date is of the form: 2025-02-15
  const parsed = parse(date, "yyyy-MM-dd", new Date());
  let formated: string;
  if (since) {
    formated = formatDistanceToNowStrict(parsed, { addSuffix: true });
    console.info("[petite-vue]: LOG - succesfully parsed date.");
    return () => h("div", { class: ["date updated"] }, formated);
  } else {
    formated = format(parsed, "d MMMM yyyy")
    console.info("[petite-vue]: LOG - succesfully parsed date.");
    return () => h("div", { class: ["date created"] }, formated);
  }
});

export {
  date,
}
