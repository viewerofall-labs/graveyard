#!/bin/bash
set -e

mkdir -p third_party
cd third_party

if [ ! -d "imgui" ]; then
    echo "Fetching ImGui..."
    git clone https://github.com/ocornut/imgui.git --depth=1
fi

cd ..

mkdir -p build
cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
make -j$(nproc)
make install

ln -sf /home/abyss/dead/exec/filediff /home/abyss/.local/bin/filediff 2>/dev/null || true

echo "Build complete. Binary at /home/abyss/dead/exec/filediff"
echo "Symlinked to ~/.local/bin/filediff"
