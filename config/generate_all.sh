#!/bin/bash

run_folder="$1"
proxy_folder="$2"
velocity_jar="$3"
minibit_plugin="$4"
out_folder="$5"

cp -r "$proxy_folder" "$out_folder/"
cp -r "$run_folder" "$out_folder/"

result=""
declare -i port=25566
for file in "$out_folder"/run/**/server.json; do
    folder=$(dirname "$file")
    name=$(basename "$folder")

    port+=1
    result+="$name = \"127.0.0.1:$port\"\n"
    (
        cd "./$folder"
`       config=$(<server.json)
        config=${config//25565/$port}
        config=${config//\"connection_mode\": 1/\"connection_mode\": 3}

        echo "$config" > server.json`
    )
done

# Remove the last newline
result=$(echo -e "$result" | sed '$d')

cp "$velocity_jar" "$out_folder/proxy/velocity.jar"
mkdir -p "$out_folder/proxy/plugins"
cp "$minibit_plugin" "$out_folder/proxy/plugins"

(
  cd "$out_folder/proxy"

  proxy_config=$(<velocity.toml)
  proxy_config=${proxy_config//lobby = \"127.0.0.1:25566\"/$result}
  echo "$proxy_config" > velocity.toml
)
