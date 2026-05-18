#!/usr/bin/env bash

# Colors matching TWM/OneShot aesthetic
export GUM_CHOOSE_CURSOR_FOREGROUND="#c792ea"
export GUM_CHOOSE_SELECTED_FOREGROUND="#00e5c8"
export GUM_CHOOSE_HEADER_FOREGROUND="#c792ea"
export GUM_INPUT_PROMPT_FOREGROUND="#c792ea"
export GUM_INPUT_CURSOR_FOREGROUND="#00e5c8"

_pick() {
    local header="$1"; shift
    if command -v gum &>/dev/null; then
        gum choose --header="$header" "$@"
    else
        printf '%s\n' "$@" | fzf --prompt="$header > " --height=10 --border
    fi
}

_input() {
    local prompt="$1" placeholder="$2"
    if command -v gum &>/dev/null; then
        gum input --prompt="$prompt: " --placeholder="$placeholder"
    else
        read -rp "$prompt: " val; echo "$val"
    fi
}

_sinks() {
    pactl list sinks short | awk '{print $2}'
}

_default_sink() {
    pactl get-default-sink
}

_current_vol_pct() {
    wpctl get-volume @DEFAULT_AUDIO_SINK@ | awk '{printf "%d", $2 * 100}'
}

do_set_default() {
    local current; current=$(_default_sink)
    mapfile -t sinks < <(_sinks)
    local choice; choice=$(_pick "Select default speaker  [current: $current]" "${sinks[@]}")
    [[ -z "$choice" ]] && return
    pactl set-default-sink "$choice"
    echo "Default sink → $choice"
}

do_test() {
    local sink; sink=$(_default_sink)
    echo "Testing audio on: $sink"
    paplay --device="$sink" /usr/share/sounds/freedesktop/stereo/audio-channel-front-left.oga &
    sleep 0.6
    paplay --device="$sink" /usr/share/sounds/freedesktop/stereo/audio-channel-front-right.oga &
    wait
    echo "Done."
}

do_volume() {
    local cur; cur=$(_current_vol_pct)
    local vol; vol=$(_input "Set volume 0-150%" "${cur}%")
    [[ -z "$vol" ]] && return
    vol="${vol//%/}"
    if ! [[ "$vol" =~ ^[0-9]+$ ]] || (( vol > 150 )); then
        echo "Invalid value: $vol"
        return
    fi
    wpctl set-volume @DEFAULT_AUDIO_SINK@ "${vol}%"
    echo "Volume → ${vol}%"
}

while true; do
    cur_vol=$(_current_vol_pct)
    cur_sink=$(_default_sink)
    choice=$(_pick "PipeWire Audio  [${cur_sink} @ ${cur_vol}%]" \
        "Set Default Speaker" \
        "Test Audio" \
        "Set Volume" \
        "Exit")

    case "$choice" in
        "Set Default Speaker") do_set_default ;;
        "Test Audio")          do_test ;;
        "Set Volume")          do_volume ;;
        "Exit"|"")             break ;;
    esac
done
