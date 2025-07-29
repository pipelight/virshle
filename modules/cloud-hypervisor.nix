{
  lib,
  config,
  inputs,
  pkgs,
  ...
}:
with lib; let
  moduleName = "virshle";
  cfg = config.services.${moduleName};
in
  mkIf cfg.enable {
    systemd.tmpfiles.rules = let
      cloud-hypervisor-fw = pkgs.fetchurl {
        url = "https://github.com/cloud-hypervisor/rust-hypervisor-firmware/releases/download/0.4.6/hypervisor-fw";
        sha256 = "0h0j0zc65pjnzrznmc3c3lrsyks6lgxh0k8j30zp41k6ph9ldhaq";
      };
      cloud-hypervisor-ovmf = pkgs.fetchurl {
        # url = "https://github.com/cloud-hypervisor/edk2/releases/download/ch-6624aa331f/CLOUDHV.fd";
        url = "https://github.com/cloud-hypervisor/edk2/releases/download/ch-a54f262b09/CLOUDHV.fd";
        sha256 = "0lh1ikngvf7lln3x9ng7c9nqb6ylv68yy0mvvlkhhk94l4c35j7x";
      };
    in [
      " L+ /run/cloud-hypervisor/hypervisor-fw - - - - ${cloud-hypervisor-fw}"
      " L+ /run/cloud-hypervisor/CLOUDVH.fd - - - - ${cloud-hypervisor-ovmf}"
    ];

    environment.systemPackages = with pkgs; [
      # VMMs
      cloud-hypervisor #v0.46.0

      # Efi related?
      OVMF-cloud-hypervisor
      OVMF
    ];
  }
