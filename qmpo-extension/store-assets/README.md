# Chrome Web Store 提出手順

## 1. zip パッケージ作成

`qmpo-extension/` ディレクトリで実行：

```bash
zip -r ../qmpo-extension.zip . -x "*.md" "*.svg" "store-assets/*"
```

## 2. Developer Dashboard にアクセス

<https://chrome.google.com/webstore/devconsole>

「新しいアイテム」→ `qmpo-extension.zip` をアップロード。

## 3. ストアの掲載情報

| 項目               | 内容                                         |
| ------------------ | -------------------------------------------- |
| 説明文（英語）     | `store-listing-en.txt` をコピペ              |
| 説明文（日本語）   | 言語を追加 → `store-listing-ja.txt` をコピペ |
| カテゴリ           | ツール（Tools）                              |
| ストアアイコン     | `../icons/icon128.png`（128x128）            |
| スクリーンショット | 以下の順にアップロード（1280x800）           |

### スクリーンショットのアップロード順

1. `1-popup-usecase.png` — ポップアップ UI + フォルダアイコン付きリンク
2. `2-explorer.png` — Explorer でフォルダが開いた結果
3. `3-settings.png` — 設定画面（ドメイン許可/ブロックリスト）
4. `4-test-links.png` — テストページ
5. `5-landing.png` — ランディングページ

## 4. プライバシー

| 項目                     | 内容                                                                |
| ------------------------ | ------------------------------------------------------------------- |
| 単一目的の説明           | Converts file:// links to directory:// scheme to open local folders |
| プライバシーポリシー URL | `https://tagawa0525.github.io/qmpo/privacy-policy.html`             |
| リモートコード           | いいえ                                                              |
| データ使用の開示         | すべて未選択（データ収集なし）                                      |
| 3つの開示方法            | すべてチェック（販売・転送・融資目的利用なし）                      |

### 権限が必要な理由

**storage:**

> Stores user preferences: enable/disable conversion toggle, show/hide folder icon setting, and domain allowlist/blocklist configuration.

（和訳: 変換の有効/無効、フォルダアイコンの表示/非表示、ドメインの許可リスト/ブロックリストなどのユーザー設定を保存するため）

**activeTab:**

> Accesses the current tab's URL to check if the domain is in the user's allowlist or blocklist, and to display the extension status in the popup.

（和訳: 現在のタブの URL を取得し、ドメインが許可リスト/ブロックリストに含まれるか確認し、ポップアップに拡張機能の状態を表示するため）

**tabs:**

> Reads the current tab's URL to determine whether file:// link conversion should be active on the current page based on user's domain settings.

（和訳: 現在のタブの URL を読み取り、ユーザーのドメイン設定に基づいて file:// リンク変換を有効にすべきか判定するため）

**ホスト権限:**

> The content script must run on all pages to detect and convert file:// links to directory:// URIs. Without broad host permissions, the extension cannot function on internal wikis, documentation sites, or other pages where file:// links appear. No page data is collected or transmitted.

（和訳: file:// リンクを検出して directory:// URI に変換するため、コンテンツスクリプトはすべてのページで実行する必要がある。広範なホスト権限がなければ、社内 Wiki やドキュメントサイトなど file:// リンクが存在するページで機能できない。ページデータの収集・送信は一切行わない）

## 5. 配布

| 項目     | 内容         |
| -------- | ------------ |
| 公開設定 | 公開         |
| 対象地域 | すべての地域 |

## 6. 審査に提出

全タブを入力後「審査に提出」。初回審査は数営業日かかる。
