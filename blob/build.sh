#!/bin/bash
set -e

# 1. Compile 64-Bit Kernel
gcc -m64 -mcmodel=kernel -mno-red-zone -mno-mmx -mno-sse -mno-sse2 \
  -ffreestanding -fno-stack-protector -fno-pic -fno-pie \
  -c kernel.c -o kernel.o

# 2. Link 64-Bit Kernel
ld -m elf_x86_64 -T linker.ld kernel.o -o kernel.elf

# 3. Download Limine bootloader if not present
if [ ! -d "limine" ]; then
  git clone https://github.com/limine-bootloader/limine.git --branch=v7.x-binary --depth=1
  make -C limine
fi

# 4. Create ISO directory structure
mkdir -p iso_root
cp kernel.elf limine.cfg limine/limine-bios.sys limine/limine-bios-cd.bin limine/limine-uefi-cd.bin iso_root/

# 5. Build bootable ISO with XORRISO
xorriso -as mkisofs -b limine-bios-cd.bin \
  -no-emul-boot -boot-load-size 4 -boot-info-table \
  --efi-boot limine-uefi-cd.bin \
  -efi-boot-part --efi-boot-image --protective-msdos-label \
  iso_root -o os.iso

# 6. Install Limine BIOS stage to ISO
./limine/limine bios-install os.iso

# 7. Run in QEMU!
qemu-system-x86_64 -cdrom os.iso
