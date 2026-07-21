# sage — a deep sage zsh theme

A minimal, single-file zsh prompt theme. Built as a p10k replacement, no nerd-font
icons required, no external binaries (`gitstatusd`, etc). Just zsh + git.

## Features

- Deep sage color palette (truecolor, `%F{#rrggbb}`)
- Two-line prompt: context on top, arrow on the bottom
- Git status via `vcs_info`: branch, staged (`●`), unstaged (`✗`), untracked (`…`), ahead/behind (`↑`/`↓`)
- Command duration for anything over `SAGE_DURATION_THRESHOLD` seconds (default 2s)
- Right-side clock (`HH:MM:SS`), always visible
- Exit code indicator (red arrow + `✗ <code>`) — only on real failures
- SSH / tmux context tags when applicable
- Background job count (`⚙<n>`)

## Install

### Standalone (any zsh, oh-my-zsh or not)

```bash
mkdir -p ~/.zsh/themes
cp sage.zsh-theme ~/.zsh/themes/sage.zsh-theme
echo 'source ~/.zsh/themes/sage.zsh-theme' >> ~/.zshrc
```

If another framework/theme (e.g. Powerlevel10k) is already setting `PROMPT`/`RPROMPT` or
hooking `precmd`/`preexec`, source this *after* it and strip the old hooks first:

```bash
precmd_functions=(${precmd_functions:#*p10k*})
preexec_functions=(${preexec_functions:#*p10k*})
source ~/.zsh/themes/sage.zsh-theme
```

### oh-my-zsh

```bash
cp sage.zsh-theme $ZSH_CUSTOM/themes/sage.zsh-theme
# in ~/.zshrc:
ZSH_THEME="sage"
```

## Config

Set before sourcing to override:

```bash
SAGE_DURATION_THRESHOLD=2   # seconds before a command's duration is shown
```

## Why not upstream to ohmyzsh/ohmyzsh?

Their theme PR queue has been effectively frozen for years — most standalone prompt
projects (p10k, spaceship, pure, starship) live outside the main tree for that reason.
This lives here as a standalone, discoverable thing instead.
