#!/bin/bash
# mk.sh <scenario-file> : body lines are inject commands, "shot NAME", or "wait S"
BODY=$(cat "$1")
cat > $PROBE_DIR/app-scn.sh <<EOS
#!/bin/bash
export WAYLAND_DISPLAY=g-probe
export XDG_DATA_HOME=$PROBE_DIR/data
D=$PROBE_DIR
rm -f \$D/in; mkfifo \$D/in
\$D/inject/target/release/inject repl < \$D/in >> \$D/driver.log 2>&1 &
exec 3>\$D/in
c() { echo "\$1" >&3; sleep "\${2:-1}"; }
shot() { spectacle -b -n -o \$D/s/\$1.png >> \$D/driver.log 2>&1; }
${APP:-/srv/FreeSynergy/fs-tasks/target/debug/fs-tasks} &
APP=\$!
sleep 7
$BODY
c "quit" 0.5
kill \$APP
EOS
chmod +x $PROBE_DIR/app-scn.sh
