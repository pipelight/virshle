import type { Config } from "https://deno.land/x/pipelight/mod.ts";
import { Mode, pipeline, step } from "https://deno.land/x/pipelight/mod.ts";

// Binaries
const bin = {
  test: "cargo run --bin virshle",
  default: "virshle",
};

const test = pipeline("test", () => [
  step("build nixos iso file", () => [
    "pipelight run create_env --attach",
    "pipelight run create_luks --attach",
    "pipelight run test_templates --attach",
  ]),
]).detach();

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
]);

const create_luks = pipeline("create_luks", () => [
  step("encrypt root", () => [
    `qemu-img create \
      -f qcow2 \
      --object secret,id=password,data=abc123 \
      -o encrypt.format=luks,encrypt.key-secret=password \
      ./iso/encrypted.qcow2 50G`,
    // Copy and encrypt device
    `qemu-img convert \
      --object secret,id=password,data=abc123 \
      --image-opts driver=qcow2,file.filename=./iso/nixos.qcow2 \
      --target-image-opts driver=qcow2,encrypt.key-secret=password,file.filename=./iso/encrypted.qcow2 \
      -n -p`,
  ]).set_mode(Mode.JumpNextOnFailure),
]);

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
      --partition=gpt \
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
    test,
    test_templates,
    create_env,
    create_luks,

    make_ci_vol,
    clean_env,
    generate_standard_empty_volumes,
  ],
};
export default config;
