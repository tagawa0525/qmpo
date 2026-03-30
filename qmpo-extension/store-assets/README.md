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
| 単一目的の説明           | Converts file:// links into directory:// URIs to open local folders |
| プライバシーポリシー URL | `https://tagawa0525.github.io/qmpo/privacy-policy.html`             |
| リモートコード           | いいえ                                                              |
| データ使用の開示         | すべて未選択（データ収集なし）                                      |
| 3つの開示方法            | すべてチェック（販売・転送・融資目的利用なし）                      |

### 権限が必要な理由

**storage:**

> Stores user preferences: enable/disable conversion toggle, show/hide folder icon setting, and domain allowlist/blocklist configuration.

（和訳: 変換の有効/無効、フォルダアイコンの表示/非表示、ドメインの許可リスト/ブロックリストなどのユーザー設定を保存するため）

**ホスト権限:**

> The content script runs on all pages to detect file:// links and convert them to directory:// URIs. Target pages (internal wikis, Confluence, SharePoint) vary by organization, so specific hosts cannot be predetermined. The extension only reads link href attributes — no other page content is accessed or transmitted. The native handler opens directories only (never executes files), blocks non-private UNC paths to prevent NTLM leaks, and validates paths before opening.

（和訳: file:// リンクを検出して directory:// に変換するため全ページで実行が必要。対象ページは組織ごとに異なり特定不可。拡張機能は href 属性の読み取りのみ。ネイティブハンドラーはディレクトリのみ開き、非プライベート UNC パスをブロックし NTLM 漏洩を防止、パスを検証してから開く）

## 5. 配布

| 項目     | 内容         |
| -------- | ------------ |
| 公開設定 | 公開         |
| 対象地域 | すべての地域 |

## 6. 審査に提出

全タブを入力後「審査に提出」。初回審査は数営業日かかる。
