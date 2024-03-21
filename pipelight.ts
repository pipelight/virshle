import type { Config } from "https://deno.land/x/pipelight/mod.ts";
import { pipeline, step } from "https://deno.land/x/pipelight/mod.ts";

// Binaries
const bin = {
  test: "deno run -A ./mod.ts",
  default: "virshle",
};

// Build VM iso or qcow2
const build_image = pipeline("build_image", () => [
  step("build nixos iso file", () => [
    "nix build ~/Fast/nixos/vm",
  ]),
  step("copy result to pwd", () => [
    "sudo cp -r ~/Fast/nixos/vm/result/iso/* ./iso/",
    "sudo chown anon:users ./iso/*",
    "sudo chmod u+w ./iso/*",
  ]),
]).detach();

const create_vm = pipeline("create_vm", () => [
  step("ensure network", () => [
    `${bin.test} net create \
  ./base/networks/default.toml -vvv`,
  ]),
  step("create vm", () => [
    `${bin.test} vm create \
  ./base/machines/console.toml -vvv`,
  ]),
]);

const clean_env = pipeline("clean_env", () => [
  step("delete vm(domain)", () => [
    `${bin.test} vm delete vm-nixos -vvv`,
  ]),
  step("delete network", () => [
    `${bin.test} net remove default_6 -vvv`,
  ]),
]);

const config = {
  options: {
    attach: true,
  },
  pipelines: [
    build_image,
    create_vm,
    clean_env,
  ],
};
export default config;
