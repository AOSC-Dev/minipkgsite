# minipkgsite
AOSC OS mini package site

## Build

```
cargo build --release

# You can use .env file in SRCDIR root
export ABBS_TREE=/tmp/aosc-os-abbs
export REDIS=redis://127.0.0.1
export MINIPKGSITE=127.0.0.1:2333

./target/release/minipkgsite

# another terminal

# You can use .env in SRCDIR/frontend
export VITE_MINIPKGSITE=http://127.0.0.1:2333

yarn dev
```
