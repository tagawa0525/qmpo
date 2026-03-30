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

> The content script runs on pages covered by the host permissions to detect file:// links and convert them to directory:// URIs. Target pages (internal wikis, Confluence, SharePoint, etc.) vary by organization, so specific hosts cannot be predetermined. The script reads window.location.hostname for allow/block decisions and scans the DOM for \<a\> elements, inspecting their href attributes to decorate and convert eligible links. No other page content is transmitted outside the browser. The native handler opens the system file manager at the specified path (opening folders and, for file paths, showing the file in its containing folder), never executes files directly, blocks non-private UNC paths to prevent NTLM leaks, and validates paths before opening.

（和訳: file:// リンクを検出して directory:// に変換するため、ホスト権限の範囲内のページでコンテントスクリプトを実行します。対象となるページ（社内 Wiki、Confluence、SharePoint など）は組織ごとに異なり、事前に特定できません。コンテントスクリプトは window.location.hostname を参照してドメインの許可/ブロック判定を行い、DOM を走査して \<a\> 要素を検出し、その href 属性を読み取って対象リンクを装飾・変換します。それ以外のページ内容がブラウザ外へ送信されることはありません。ネイティブハンドラーは指定されたパスをファイルマネージャーで開き（フォルダパスの場合はそのフォルダを、ファイルパスの場合はそのファイルを含むフォルダを選択状態で表示し）、ファイルを直接実行することはありません。また、非プライベートな UNC パスをブロックして NTLM 漏洩を防止し、パスを検証してから開きます。）

## 5. 配布

| 項目     | 内容         |
| -------- | ------------ |
| 公開設定 | 公開         |
| 対象地域 | すべての地域 |

## 6. 審査に提出

全タブを入力後「審査に提出」。初回審査は数営業日かかる。
