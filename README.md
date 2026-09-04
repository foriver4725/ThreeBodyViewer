# ThreeBodyViewer
三体恒星系、及びその惑星から見た恒星の動きをシミュレーションする

## 実行

```sh
cargo run
```

数字1〜9・0でプリセット切替、スペース長押しで早送り、上下キーで倍率調整。

## Web / Wasm

RustとPython 3が必要です。最初にWasmターゲットを追加します。

```sh
rustup target add wasm32-unknown-unknown
bash scripts/build-web.sh
python3 -m http.server 8080 --directory dist
```

ブラウザで http://localhost:8080 を開いてください。`file://`では起動できません。
キーボード操作を行う場合は画面をクリックしてください。PCブラウザ向けです。

`dist/`内の3ファイルを静的ホスティングへ配置すれば公開できます。
Wasmは`application/wasm`で配信してください。すべて相対パスのため、サブディレクトリへの配置にも対応します。
ローダーはCargo.lockに固定されたmacroquadと同梱のものを使用し、外部CDNには依存しません。
生成物の`dist/`はGit管理対象外です。公開先へのデプロイは別途行ってください。

## ソース構成

- `src/main.rs`：起動、入力、フレーム進行
- `src/model.rs`：恒星・惑星・プリセットのデータ
- `src/presets.rs`：初期値ファクトリ
- `src/physics.rs`：重力と時間刻み
- `src/lagrange.rs`：ラグランジュ点計算
- `src/observer.rs`：地上座標・飛星境界・相対熱量
- `src/render/`：俯瞰図・地上の空・熱量パネル
- `src/tests.rs`：計算と境界のテスト

```sh
cargo test
```
