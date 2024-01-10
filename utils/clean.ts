// Object manipulation utils
// Colors
import { colors } from "https://deno.land/x/cliffy/ansi/mod.ts";

const red = colors.bold.red;

export const removeEmpty = (obj: any) => {
  Object.keys(obj).forEach((k) =>
    (obj[k] && typeof obj[k] === "object") && removeEmpty(obj[k]) ||
    (!obj[k] && obj[k] !== undefined) && delete obj[k]
  );
  return obj;
};

const home = Deno.env.get("HOME");
export const replaceRelativePaths = async (obj: any) => {
  for (const [key, value] of Object.entries(obj)) {
    if (value && typeof value === "object") {
      obj[key] = await replaceRelativePaths(obj[key]);
    } else if (value && typeof obj[key] === "string") {
      if (
        (value as string).includes("~")
      ) {
        obj[key] = (value as string).replace("~", home!);
      }
      if (
        (value as string).includes("./") ||
        (value as string).includes("../")
      ) {
        try {
          obj[key] = await Deno.realPath(value as string);
        } catch (err) {
          const msg = `The relative path: ${value} resolves to nothing`;
          throw new Error(err);
        }
      }
    }
  }
  return obj;
};
