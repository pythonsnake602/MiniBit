#!/bin/bash

server="$1"
upper_server=$(echo "$server" | tr '[:lower:]' '[:upper:]')

cat <<EOF > config.yml
telemetry:
  tracing: true

${server}:
  enabled: true
  path: ${server}
  network:
    port: 25565
    max_players: 1000
EOF

cargo run -r -- -c config.yml --data-path ../../data
