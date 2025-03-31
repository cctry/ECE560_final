# ECE560 final project
## Run the project
1. Clone the repository
```bash
git clone https://github.com/cctry/ECE560_final.git
cd ECE560_final
```
2. Launch an HTTP server
```bash
python3 -m http.server 8000
```
3. Open your web browser and navigate to `http://localhost:8000`

## Compile the project
1. Install Rust and Cargo if you haven't already

2. Add the WebAssembly target
```bash
rustup target add wasm32-unknown-unknown
```

3. Install wasm-pack
```bash
cargo install wasm-pack
```

4. Build the project
```bash
wasm-pack build --target web
```