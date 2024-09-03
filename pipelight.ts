import type { Config } from "https://deno.land/x/pipelight/mod.ts";
import { Mode, pipeline, step } from "https://deno.land/x/pipelight/mod.ts";

// Binaries
const bin = {
  test: "cargo run --bin virshle",
  default: "virshle",
};

/**
 * Build qcow2 image
 */
const create_env = pipeline("create_env", () => [
  step("build nixos iso file", () => [
    "nix flake update ~/Fast/nixos/vm",
    "nix build ~/Fast/nixos/vm",
  ]),
  step("copy image to repo root", () => [
    "sudo cp -Lr ./result/*.qcow2 ./iso/",
    "sudo chown anon:users ./iso/*",
    "sudo chmod u+w ./iso/*",
  ]),
  step("copy efi vars to repo root", () => [
    "sudo cp -Lr /run/libvirt/nix-ovmf/* ./iso/",
  ]).set_mode(Mode.JumpNextOnFailure),
]).detach();

/**
 * Create template network and vm
 */
const test_templates = pipeline("test_templates", () => [
  step("delete existing testing resources", () => [
    `${bin.test} net rm \
    default_6`,
    `${bin.test} vm rm \
    vm-nixos`,
  ]).set_mode(Mode.JumpNextOnFailure),
  step("ensure network", () => [
    `${bin.test} create \
    ./templates/net/base.toml -vvv`,
  ]),
  step("create vm", () => [
    `${bin.test} create \
    ./templates/vm/base.toml -vvv`,
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
    test_templates,
    create_env,

    make_ci_vol,
    clean_env,
    generate_standard_empty_volumes,
  ],
};
export default config;
