# qmpo ブラウザ拡張機能

**Open Directory With Browser** - `file://` リンクを `directory://` スキームに変換し、
qmpoハンドラ経由でローカルファイル/フォルダへのアクセスを可能にします。

ブラウザで `file://` リンクをクリックしても動作しない問題を解決します。

qmpoはクロスプラットフォームの `directory://` URIスキームハンドラで、
ディレクトリをファイルマネージャで開きます。この拡張機能は `file://` リンクを
qmpoに橋渡しし、Local Explorerなどの既存ソリューションより優れた代替手段となります。

## 機能

- `file://` リンクを自動的に `directory://` に変換
- 任意のWebサイトで動作（ドメイン許可リスト/ブロックリスト設定可能）
- 変換されたリンクに視覚的インジケーター（フォルダアイコン）を表示
- 外部依存なし - 純粋なJavaScript

## 必要条件

- **qmpo** がインストールされ、`directory://` URIハンドラとして登録されていること
- ChromeまたはChromiumベースのブラウザ（Edge、Braveなど）

### qmpoのインストール

[GitHub Releases](https://github.com/tagawa0525/qmpo/releases)からqmpoをダウンロード:

| OS | ダウンロード |
| --- | --- |
| Windows | `qmpo-windows-x64.zip` |
| macOS (Intel) | `qmpo-macos-x64.tar.gz` |
| macOS (Apple Silicon) | `qmpo-macos-arm64.tar.gz` |
| Linux | `qmpo-linux-x64.tar.gz` |

またはインストールスクリプトを使用:

```bash
# macOS/Linux
curl -sSL \
  https://raw.githubusercontent.com/tagawa0525/qmpo/main/scripts/install.sh \
  | bash

# Windows (PowerShell)
irm \
  https://raw.githubusercontent.com/tagawa0525/qmpo/main/scripts/install.ps1 \
  | iex
```

詳細は[qmpoリポジトリ](https://github.com/tagawa0525/qmpo)を参照してください。

## インストール

### ソースから（開発者モード）

1. **リポジトリをクローンまたはダウンロード**

   ```bash
   git clone https://github.com/tagawa0525/qmpo.git
   cd qmpo
   ```

2. **Chrome拡張機能ページを開く**

   - Chromeで `chrome://extensions/` にアクセス
   - または: メニュー（⋮）→ 拡張機能 → 拡張機能を管理

3. **デベロッパーモードを有効化**

   - 右上の「デベロッパーモード」スイッチをオンにする

4. **拡張機能を読み込む**

   - 「パッケージ化されていない拡張機能を読み込む」ボタンをクリック
   - `qmpo-extension` フォルダを選択（親の `qmpo` フォルダではなく）
   - 拡張機能リストに表示されます

5. **インストールの確認**

   - 拡張機能リストに「qmpo - Open Local Files」が表示されること
   - ツールバーに拡張機能アイコンが表示されること（ピン留めが必要な場合あり）

### トラブルシューティング

#### マニフェストファイルが見つからないか読み取れません

- `qmpo-extension` フォルダを選択していることを確認（親ディレクトリではなく）
- 選択したフォルダに `manifest.json` が存在することを確認

#### ページで拡張機能が動作しない

- 拡張機能インストール後、ページを更新してください
- ドメインがブロックリストに含まれていないか確認（拡張機能アイコン → 設定）
- qmpoがインストールされているか確認: ターミナルで `qmpo-lau status` を実行

#### ディレクトリを開けませんでした エラー

- qmpoをインストール: [qmpoのインストール](#qmpoのインストール)セクションを参照
- プロトコルハンドラを登録: `qmpo-lau register`

### Chrome Web Storeから

（近日公開予定）

## 使い方

1. qmpoがインストールされ登録されていることを確認:

   ```bash
   qmpo-lau status
   # 登録されていない場合:
   qmpo-lau register
   ```

2. `file://` リンクのあるページにアクセス（例: 社内Wiki、Confluence）

3. `file://` リンクをクリック - ファイルが選択された状態でファイルマネージャが開きます

## 設定

拡張機能アイコンをクリックするか、設定に移動:

- **変換を有効化**: 拡張機能のオン/オフを切り替え
- **フォルダアイコンを表示**: 変換されたリンクに視覚的インジケーターを表示
- **許可ドメイン**: これらのドメインでのみリンクを変換（空 = すべて許可）
- **ブロックドメイン**: これらのドメインではリンクを変換しない

### 例: 社内イントラネットのみ

```text
許可ドメイン:
wiki.company.com
confluence.company.com
intranet.internal
```

## 動作の仕組み

```text
ユーザーが file://path/to/file をクリック
        ↓
拡張機能がクリックをインターセプト
        ↓
directory://path/to/file に変換 🔄
        ↓
ブラウザがURIハンドラを起動
        ↓
qmpoがファイルを選択した状態でファイルマネージャを開く
```

## ファイル構成

```text
qmpo-extension/
├── manifest.json    # 拡張機能の設定（権限、スクリプト登録など）
├── background.js    # Service Worker（バックグラウンド処理）
├── content.js       # コンテンツスクリプト（ページ内で実行）
├── popup.html       # ツールバーアイコンクリック時のポップアップUI
├── popup.js         # ポップアップのロジック
├── options.html     # 詳細設定ページのUI
├── options.js       # 設定ページのロジック
├── icons/           # 拡張機能アイコン
│   ├── icon16.png
│   ├── icon48.png
│   └── icon128.png
├── test.html        # ローカルテスト用ページ
├── README.md        # 英語ドキュメント
└── README-ja.md     # 日本語ドキュメント（このファイル）
```

### 各ファイルの役割

#### manifest.json

Chrome拡張機能の設定ファイル。以下を定義:

- 拡張機能名、バージョン、説明
- 必要な権限（`storage`, `activeTab`, `tabs`）
- コンテンツスクリプトの適用範囲
- バックグラウンドService Workerの指定

#### content.js

Webページ内で実行されるスクリプト。主な機能:

- `file://` で始まるリンクを検出
- クリックイベントをインターセプト
- `file://` を `directory://` に変換
- 隠しiframeでプロトコルハンドラを起動
- 視覚的インジケーター（📂）の追加

#### background.js

拡張機能のバックグラウンドで動作するService Worker:

- コンテンツスクリプトからのメッセージを処理
- （現在は隠しiframe方式に移行したため、主要機能は使用されていない）

#### popup.html / popup.js

ツールバーの拡張機能アイコンをクリックした時に表示されるUI:

- 拡張機能の有効/無効切り替え
- フォルダアイコン表示の切り替え
- 設定ページへのリンク

#### options.html / options.js

拡張機能の詳細設定ページ:

- 許可ドメインリストの管理
- ブロックドメインリストの管理
- 設定のインポート/エクスポート

## Local Explorerとの比較

| 機能 | Local Explorer | qmpo + 拡張機能 |
| --- | --- | --- |
| プラットフォーム | Windowsのみ | Windows/macOS/Linux |
| ブラウザ | Chrome/Edge | Chrome/Chromiumベース |
| 通信方式 | Native Messaging API | カスタムURIスキーム |
| 動作 | ファイルを直接開く | ファイルを選択した状態でディレクトリを開く |
| セキュリティ | ファイルを実行可能 | ディレクトリのみ（より安全） |

## 開発

```bash
# 変更を監視（オプション）
npm install -g web-ext
web-ext run --source-dir=.
```

### デバッグ

1. `chrome://extensions/` でqmpo拡張機能の「詳細」をクリック
2. 「Service Worker」リンクをクリックしてDevToolsを開く
3. コンソールでbackground.jsのログを確認

コンテンツスクリプトのデバッグ:

1. 対象ページでDevToolsを開く（F12）
2. コンソールで `qmpo:` プレフィックスのログを確認

## ライセンス

MIT
