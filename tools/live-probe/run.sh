#!/bin/bash
# kwin_wayland --virtual on a private bus, one app inside, for a live probe.
export XDG_RUNTIME_DIR=${XDG_RUNTIME_DIR:-/run/user/$(id -u)}
export XDG_DATA_DIRS=$PROBE_DIR/xdg:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}
export XKB_DEFAULT_LAYOUT=${LAYOUT:-us}
exec dbus-run-session -- kwin_wayland --virtual --width 1280 --height 800 --socket g-probe --no-lockscreen --exit-with-session "$1"
