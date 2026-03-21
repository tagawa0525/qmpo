# Chrome Web Store 提出手順

## 1. zip パッケージ作成

`qmpo-extension/` ディレクトリで実行：

```bash
zip -r ../qmpo-extension.zip . -x "*.md" "*.svg" "store-assets/*"
```

## 2. Developer Dashboard にアクセス

https://chrome.google.com/webstore/devconsole

「新しいアイテム」→ `qmpo-extension.zip` をアップロード。

## 3. ストアの掲載情報

| 項目               | 内容                                         |
| ------------------ | -------------------------------------------- |
| 説明文（英語）     | `store-listing-en.txt` をコピペ              |
| 説明文（日本語）   | 言語を追加 → `store-listing-ja.txt` をコピペ |
| カテゴリ           | ユーティリティ                               |
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
| プライバシーポリシー URL | https://tagawa0525.github.io/qmpo/privacy-policy.html               |
| データ使用の開示         | データの収集・送信なし                                              |

`<all_urls>` 権限の正当性を問われた場合：
> The extension needs to run on all pages to detect and convert file:// links. No data is collected or transmitted.

## 5. 配布

| 項目     | 内容         |
| -------- | ------------ |
| 公開設定 | 公開         |
| 対象地域 | すべての地域 |

## 6. 審査に提出

全タブを入力後「審査に提出」。初回審査は数営業日かかる。
