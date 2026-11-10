# live-probe: a real window, real keys and mouse, pictures — without touching the user's session

Found on 2026-11-08 (Thread G). It runs an app in a nested `kwin_wayland --virtual` on a private D-Bus, sends keys and
mouse through `org_kde_kwin_fake_input` and takes pictures with `spectacle`. It found the black window (Blinc issue 70),
the invisible text in a focused field and the cursor behind the text (issue 69). **Never use `uinput`: it types into the
user's real session.**

## What is here
- `run.sh <script>`: starts the nested KWin (1280x800, socket `g-probe`) and runs `<script>` inside it
  (`--exit-with-session`). Set `PROBE_DIR` to this directory.
- `inject/`: a small Rust program (`cargo build --release` there; needs `wayland-client` and
  `wayland-protocols-plasma`). `inject repl` reads lines from stdin: `move X Y`, `click X Y`, `type TEXT`
  (US layout key codes), `key CODE`, `wait MS`, `quit`. It must stay alive for the whole run: if it exits the fake pointer
  goes, and winit panics with "failed to get pointer data".
- `xdg/applications/fs-inject.desktop.in`: KWin only lets a program use `org_kde_kwin_fake_input` when a desktop file
  names its executable and has `X-KDE-Wayland-Interfaces=org_kde_kwin_fake_input`. Copy it to
  `$PROBE_DIR/xdg/applications/fs-inject.desktop`, replace `@PROBE_DIR@` and the `Exec` path; `run.sh` puts that directory
  in `XDG_DATA_DIRS` for the nested KWin only. (`KWIN_WAYLAND_NO_PERMISSION_CHECKS` does nothing in KWin 6.8.)
- `mk.sh <scenario.txt>`: makes `app-scn.sh` from a scenario (lines `c "cmd" SECONDS`, `shot NAME`); set `APP` to the
  program, pictures go to `$PROBE_DIR/s/`. `scnA/B/C.txt` are the scenarios of the black window and the focus text
  (fs-tasks: New, click Title, type, click a status button).

## Run
```bash
export PROBE_DIR=$PWD; (cd inject && cargo build --release)
mkdir -p s; APP=/path/to/fs-tasks ./mk.sh scnB.txt
timeout 100 ./run.sh $PWD/app-scn.sh; ls s
```
Kill a left-over nested KWin by its pid, not with `pkill -f` (that matches your own shell). Pictures: `s/B1.png` ...

## Known gaps
No libei (`org.kde.KWin.EIS.RemoteDesktop`, crate `reis`), no keyboard layout other than US in `type` (for the AltGr/@ bug
start KWin with `KWIN_XKB_DEFAULT_LAYOUT`/`XKB_DEFAULT_LAYOUT=de` and send raw key codes with `key`), no frame timing.

## Added 2026-11-10 (Thread C)
`inject repl` also takes `down CODE` and `up CODE` (evdev codes, to hold AltGr = 100 or Shift = 42 while another key goes). `run.sh` takes `LAYOUT=de` (sets `XKB_DEFAULT_LAYOUT` for the nested KWin). `scnD-keys.txt` is the key test on a German layout: with the example `cargo run -p fs-components-fields --example form` as `APP`, it types AltGr+Q, `+`, `#`, Shift+7 into the fields; Blinc gives `2 = 3 ?` (git-bug 809d1e72a4db0575b63bc93f9074433deb6fe4e897b2a003ef87b5eff33a6844 in the Blinc clone). The "known gap" about layouts above is closed.
