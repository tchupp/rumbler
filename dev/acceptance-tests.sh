#!/usr/bin/env bash

set -euo pipefail

docker compose -f acceptance_tests/docker-compose.yml run --build --remove-orphans test
