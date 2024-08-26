import type { Config } from "https://deno.land/x/pipelight/mod.ts";
import { Mode, pipeline, step } from "https://deno.land/x/pipelight/mod.ts";

// Binaries
const bin = {
  test: "deno run -A ./mod.ts",
  default: "virshle",
};

// Build VM iso or qcow2
const build_image = pipeline("build_image", () => [
  step("build nixos iso file", () => [
    // "nixos-generate -c ~/Fast/nixos/vm/flake.nix",
    "nix build ~/Fast/nixos/vm",
  ]),
  step("copy result to pwd", () => [
    "sudo cp -r ./result/iso/* ./iso/",
    "sudo chown anon:users ./iso/*",
    "sudo chmod u+w ./iso/*",
  ]).set_mode(Mode.JumpNextOnFailure),
  step("copy result to pwd", () => [
    "sudo cp -r ./result/iso/* ./iso/",
    "sudo chown anon:users ./iso/*",
    "sudo chmod u+w ./iso/*",
  ]).set_mode(Mode.JumpNextOnFailure),
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

/**
 * Create empty template volume with standard sizes
 */
const generate_standard_empty_volumes = pipeline(
  "generate_standard_empty_volumes",
  () => {
    const dir = "~/.libvirt/volumes/standard";

    const steps = [step(`ensure storage directory`, () => [`mkdir -p ${dir}`])];
    const sizes = [
      {
        count: "20",
        unit: "M",
      },
      {
        count: "10",
        unit: "G",
      },
    ];

    for (const { count, unit } of sizes) {
      steps.push(
        step(`create empty storage volume of ${count}${unit}`, () => [
          `dd if=/dev/zero of=${dir}/${count}${unit}.img bs=1${unit} count=${count}`,
        ]),
      );
    }
    return steps;
  },
).log_level("info");

/**
 * Create a cdrom volume
 * with the pipelight-init data inside
 */
const make_ci_vol = pipeline("make_ci_vol", () => [
  // step("create user-data iso file", () => [
  //   `genisoimage \
  //   -output ./iso/pipelight-init.img \
  //   -volid pipelight-init -rational-rock -joliet \
  //   ./base/pipelight-init`,
  // ]),
  step("create user-data iso file", () => [
    `virt-make-fs \
      ./base/pipelight-init/ ./iso/pipelight-init.img`,
  ]),
]);

const clean_env = pipeline("clean_env", () => [
  step("delete vm(domain)", () => [`${bin.test} vm delete vm-nixos -vvv`]),
  step("delete network", () => [`${bin.test} net remove default_6 -vvv`]),
]);

const config = {
  options: {
    attach: false,
    log_level: "info",
  },
  pipelines: [
    build_image,
    make_ci_vol,
    create_vm,
    clean_env,
    generate_standard_empty_volumes,
  ],
};
export default config;
