# pumpkin!

<img width="1085" height="531" alt="image" src="https://github.com/user-attachments/assets/d150930d-885f-4a0f-a5a4-6e7ea60274a3" />

pumpkin is a cozy, autumn-themed platformer! your character can pick up and throw around pumpkins which can be stood on and used to reach higher platforms.

the game is really short, but i hope you enjoy! 

## about

project made in rust with macroquad. all assets and code done by myself

## building

you need rust installed.

standalone: `cargo run`

for web with `basic-http-server`, do:
```bash
cargo build --release --target wasm32-unknown-unknown && cp target/wasm32-unknown-unknown/release/pumpkin.wasm web/ && basic-http-server web/
```
