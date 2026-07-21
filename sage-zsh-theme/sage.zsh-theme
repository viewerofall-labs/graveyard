# sage.zsh-theme — deep sage prompt, p10k replacement
# Two-line prompt: icon/user/path/git/duration on line 1, arrow on line 2.
# Right side: clock (always) + exit code (on failure only).

setopt PROMPT_SUBST
zmodload zsh/datetime

# -- palette --------------------------------------------------------------
SAGE_DARK='#3E4E41'
SAGE='#6B8F71'
SAGE_LIGHT='#A3C4A8'
SAGE_MUTED='#7C9885'
CLAY='#C08552'
CREAM='#E8E4D9'
ROSE_RED='#C1666B'

# -- git status via vcs_info -----------------------------------------------
autoload -Uz vcs_info

zstyle ':vcs_info:*' enable git
zstyle ':vcs_info:git:*' check-for-changes true
zstyle ':vcs_info:git:*' stagedstr "%F{$CLAY}●%f"
zstyle ':vcs_info:git:*' unstagedstr "%F{$ROSE_RED}✗%f"
zstyle ':vcs_info:git:*' formats "%F{$SAGE_MUTED} on %F{$SAGE_LIGHT}⎇ %b%f%c%u%m"
zstyle ':vcs_info:git:*' actionformats "%F{$SAGE_MUTED} on %F{$SAGE_LIGHT}⎇ %b%f %F{$CLAY}(%a)%f%c%u%m"
zstyle ':vcs_info:git+set-message:*' hooks git-untracked git-aheadbehind

# untracked files -> a small marker
+vi-git-untracked() {
  if git status --porcelain 2>/dev/null | command grep -q '^??'; then
    hook_com[unstaged]+=" %F{$CLAY}…%f"
  fi
}

# ahead/behind upstream -> arrows with counts
+vi-git-aheadbehind() {
  local ahead behind
  ahead=$(git rev-list --count @{upstream}..HEAD 2>/dev/null)
  behind=$(git rev-list --count HEAD..@{upstream} 2>/dev/null)
  local msg=""
  [[ -n $ahead && $ahead -gt 0 ]] && msg+=" %F{$SAGE_LIGHT}↑$ahead%f"
  [[ -n $behind && $behind -gt 0 ]] && msg+=" %F{$ROSE_RED}↓$behind%f"
  hook_com[misc]+="$msg"
}

# -- command duration -------------------------------------------------------
SAGE_DURATION_THRESHOLD=2   # seconds; shorter commands stay quiet

sage_preexec() {
  sage_cmd_start=$EPOCHREALTIME
}

sage_format_duration() {
  local -F frac=$1
  local -i total=$1 h m s
  (( h = total / 3600 ))
  (( m = (total % 3600) / 60 ))
  (( s = total % 60 ))
  if (( h > 0 )); then
    printf '%dh%dm%ds' $h $m $s
  elif (( m > 0 )); then
    printf '%dm%ds' $m $s
  else
    printf '%.1fs' $frac
  fi
}

sage_duration_msg=""
sage_precmd() {
  local exit_code=$?

  if [[ -n $sage_cmd_start ]]; then
    local elapsed=$(( EPOCHREALTIME - sage_cmd_start ))
    if (( elapsed >= SAGE_DURATION_THRESHOLD )); then
      sage_duration_msg=" %F{$SAGE_MUTED}took $(sage_format_duration $elapsed)%f"
    else
      sage_duration_msg=""
    fi
    unset sage_cmd_start
  else
    sage_duration_msg=""
  fi

  vcs_info
  return $exit_code
}

preexec_functions+=(sage_preexec)
precmd_functions+=(sage_precmd)

# -- context: ssh / tmux (static per session, computed once) ---------------
sage_context=""
[[ -n "$SSH_CONNECTION" || -n "$SSH_TTY" ]] && sage_context+="%F{$CLAY}ssh%f "
[[ -n "$TMUX" ]] && sage_context+="%F{$SAGE_MUTED}tmux%f "

# -- prompt -----------------------------------------------------------------
PROMPT='%F{$SAGE_DARK}🌿%f ${sage_context}%F{$SAGE}%n%f%F{$SAGE_MUTED}@%f%F{$SAGE}%m%f %F{$SAGE_DARK}in%f %F{$CREAM}%~%f${vcs_info_msg_0_}${sage_duration_msg}
%F{$SAGE_DARK}╰─%f%(1j.%F{$CLAY}⚙%j %f.)%(?.%F{$SAGE_LIGHT}.%F{$ROSE_RED})❯%f '

RPROMPT='%(?..%F{$ROSE_RED}✗ %?%f )%F{$SAGE_MUTED}%D{%H:%M:%S}%f'
