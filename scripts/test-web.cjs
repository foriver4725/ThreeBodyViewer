// 実際のローダーと起動コードの宣言衝突、およびWasm読み込みの開始を検証する。
const fs = require('node:fs');
const vm = require('node:vm');
const assert = require('node:assert/strict');
const html = fs.readFileSync('dist/index.html', 'utf8');
const loader = fs.readFileSync('dist/mq_js_bundle.js', 'utf8');
const inline = [...html.matchAll(/<script>([\s\S]*?)<\/script>/g)].map(match => match[1]).join('\n');
assert.ok(inline.length > 0);
// classic script間で共有されるグローバル字句スコープをまとめて構文チェック。
new vm.Script(loader + '\n' + inline);
let requested;
const statusElement = { textContent: '読み込み中' };
const canvasElement = { addEventListener() {}, focus() {} };
const sandbox = {
  document: { getElementById: id => id === 'status' ? statusElement : canvasElement },
  window: { addEventListener() {} },
  load: path => { requested = path; },
};
vm.runInNewContext(inline, sandbox);
assert.equal(requested, 'ThreeBodyViewer.wasm');
assert.equal(statusElement.textContent, '');
delete sandbox.load;
vm.runInNewContext(inline, sandbox);
assert.match(statusElement.textContent, /ランタイムが見つかりません/);
console.log('Web起動テスト成功：変数衝突なし、Wasm読み込み開始、ローダー欠落時のエラー表示');
