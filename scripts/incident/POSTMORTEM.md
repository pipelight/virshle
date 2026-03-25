# Post mortem

## Self corruption of boot partition on efi.raw.

**25 Mars 2026**

Some machines have abnormal boot, although they are all the same.

- During tests:
  One time out of 10 it occures that a VM can't boot and is stuck on boot partition discovery.
  I ignored it because I had to build this fucking hypervisor in time.

- In production:
  A machine containing a simple webserver and 2 websites just corrupted itself after running perfectly for 2 weeks.

If machine can get such corruption over randome time,
now that is a **huge problem**!

I suspect boot (ext4) partition to be unmountable/corrupted.

## Solutions?

Maybe Update cloud-hypervisor.
