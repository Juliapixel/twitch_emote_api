_default:
    just --list

dev:
    cd src-web && pnpm run dev

format:
    cd src-web && pnpm run format
    cd api && cargo fmt
