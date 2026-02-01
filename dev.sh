#!/bin/bash

SECRET=$(LC_ALL=C tr -dc 'A-Za-z0-9' < /dev/urandom | head -c 12)

tmux new-session -d "MINIBIT_FORWARDING_SECRET=$SECRET bazel run //:minibit_server -- -c $PWD/example_configs/velocity/minibit.yml" \; split-window "./dev_proxy.sh" \; attach
