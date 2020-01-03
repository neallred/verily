command -v rustc || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.profile
command -v wasm-pack || curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
if [ -e data-bundler/data/words-index.json.gz ]
then
    echo "found indices, skipping index creation"
else
  cargo run --release --bin data-bundler
fi
wasm-pack build --release client/
pushd client/web/
npm i
NODE_ENV=production npm run build
popd
cargo build --release --bin server
