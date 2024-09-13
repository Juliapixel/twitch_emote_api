_default:
    just --list

import? "local.just"

dev:
    cd web-example && pnpm run dev

format:
    cd web-example && pnpm run format
    cd api && cargo fmt
