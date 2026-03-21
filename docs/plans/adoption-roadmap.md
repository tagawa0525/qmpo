# qmpo 普及計画

## 現状（2026-03-22 時点）

- v0.2.0 リリース済み
- Windows / macOS / Linux クロスプラットフォーム対応
- GitHub Releases でバイナリ配布
- AUR（Arch Linux）、Nix flake でパッケージ配布
- Chrome 拡張機能あり（未公開）
- GitHub Pages 有効化済み（ランディングページ + テストページ + プライバシーポリシー）
- NTLM leak 防止等のセキュリティ対策実装済み

## 優先度順の施策

### P0: インストール障壁の除去

#### 1. Chrome Web Store 公開

- **状態**: プライバシーポリシー作成済み、Manifest V3 対応済み
- **残タスク**:
  - [ ] デベロッパー登録（$5）
  - [ ] ストア用スクリーンショット作成（1280x800、2〜5枚）
  - [ ] ストア掲載テキスト（短い説明 132文字以内 + 詳細説明）
  - [ ] ZIP パッケージ作成（test.html, icon.svg, README 除外）
  - [ ] 審査提出 → `<all_urls>` の正当性説明を準備
- **効果**: 「デベロッパーモードで読み込み」が不要になり、一般ユーザーの導入障壁が劇的に下がる

#### 2. Homebrew formula

- macOS ユーザーの標準インストール手段
- `brew install qmpo` → qmpo + qmpo-lau をインストール
- Homebrew tap として `tagawa0525/homebrew-qmpo` を作成

#### 3. Winget / Scoop

- Windows の主要パッケージマネージャ
- Scoop は CLI ツールとの相性が良い（bucket に manifest を追加）
- Winget は Microsoft Store 経由での認知にもつながる

#### 4. crates.io 公開

- `cargo install qmpo` で Rust ユーザー層にリーチ
- ソースビルドのため全プラットフォーム対応

### P1: 認知の獲得

#### 5. ランディングページの強化

- GIF/動画デモ: `file://` リンクをクリック → ファイルマネージャが開く一連の流れ
- ユースケース例: 社内 Wiki、Confluence、チケット管理ツールから共有フォルダを開く
- 「file:// が使えない問題」の背景説明

#### 6. 比較表の公開

- Local Explorer との比較（既に README にあるが、ランディングページにも）
- ブラウザ設定変更、VBScript 等の回避策との比較

#### 7. 記事・フォーラム投稿

- Zenn / Qiita: 日本語で技術記事
- Reddit: r/rust, r/sysadmin
- Hacker News: Show HN

### P2: 信頼性・企業導入

#### 8. SECURITY.md

- NTLM leak 対策の説明
- UNC パス検証の仕組み
- 設定ファイルによる allowlist 管理
- 企業の IT 部門が導入判断する際の材料

#### 9. Firefox 対応

- WebExtension API は Chrome と大部分互換
- `browser.storage.sync` への書き換え等が必要
- Chrome の次に大きいブラウザシェア

### P3: 将来的な拡張

#### 10. macOS AutoLaunchProtocolsFromOrigins 対応

- Issue #18 で追跡中
- macOS 実機での確認が必要

#### 11. Edge Add-ons ストア公開

- Chrome 拡張機能がそのまま動作する可能性が高い
- Edge ユーザー（企業環境で多い）へのリーチ

## 次のアクション

直近で着手すべきは **Chrome Web Store 公開**。他のすべての認知施策の前提になる。
