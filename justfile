clean:
    cargo clean
    rm -rf web/pkg

build:
    wasm-pack build \
        --mode no-install \
        --target web \
        --profiling \
        --out-dir web/pkg \
        --no-pack \
        --no-typescript \
        .

serve port="8080":
    python3 -m http.server --bind localhost -d web {{port}}

dev:
    cargo watch -qcs 'just build' -s 'just serve'
