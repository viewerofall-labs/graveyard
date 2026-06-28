# filediff

GUI file diff tool in C++ with ImGui.

## Features

- Load files from disk
- Paste text directly into editors
- Side-by-side view with line-level and word-level diff highlights
- Line-by-line diff algorithm (LCS-based)
- Color-coded diff output (green for additions, red for removals, white for context)

## Build

```bash
cd ~/dead/filediff
./setup.sh
```

Binary: `~/.local/bin/filediff` or `/home/abyss/dead/exec/filediff`

## Usage

```bash
filediff
```

1. Paste or load files into the left/right editors
2. Click "Compare" to compute the diff
3. Check "Show Diff" to view the results

## Project Structure

```
filediff/
├── src/
│   ├── main.cpp      OpenGL + GLFW + ImGui setup
│   ├── ui.cpp/h      UI rendering and file editors
│   ├── diff.cpp/h    LCS-based diff algorithm
├── CMakeLists.txt    Build config
├── setup.sh          Bootstrap script (fetches ImGui, builds)
└── third_party/      ImGui (fetched by setup.sh)
```

## Dependencies

- OpenGL
- GLFW3
- ImGui (fetched by setup.sh)

Install on CachyOS:
```bash
sudo pacman -S glfw-x11 mesa
```
