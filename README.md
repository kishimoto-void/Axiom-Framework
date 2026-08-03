# Axiom Framework

**A Universal Protocol Framework for Intelligence-Neutral AI Systems**

「同じ入力なのに、言語や実装が違うだけで状態がズレる」  
そのズレが、LLM同士・マルチエージェント間の不整合やハルシネーションの温床になる。

Axiom Framework は、この問題を根本から解決するために設計された  
**言語中立・知性中立・決定論的なプロトコル層**です。

Instead of defining *how* an AI should think, Axiom defines *how state is represented, verified, and constrained* — so different languages, runtimes, and models can share the same reality.

**Latest**: [v1.1.0](https://github.com/kishimoto-void/Axiom-Framework/releases/tag/v1.1.0) · Rust companion: [`axiomFrameworkRUSTv1.5/`](axiomFrameworkRUSTv1.5/)

---

## なぜこのシステムを作ったのか / Why

従来の AI システムでは、同じ入力でも次の差異が起きやすい：

- Python / Rust / C でシリアライズ結果が微妙に異なる  
- タイムスタンプや浮動小数点の扱いが実装依存になる  
- ハッシュや状態 ID が一致しない  
- 結果として「見た目は同じなのに中身が違う」状態が生まれる  

この差異が積み重なると、マルチエージェントでは **LLM が他の LLM の出力を正しく検証・制約できなくなる**。

Axiom Framework は、この「見えないズレ」を消すために作られました。

---

## 設計思想（独自性の核心） / Design Philosophy

### 1. 不変と可変の明確な分離

| レイヤー | 役割 | 性質 |
|---------|------|------|
| **ACP** (AXIOM Common Protocol) | 状態の整合性・因果・証明 | **不変** |
| **PLP Capsule** | 実行時状態・観測・成長 | **可変** |

- **不変部分（ACP）** は、言語や実装を超えてビット単位で同一であることを保証する  
- **成長・変化** はすべて Capsule に閉じ込める  
- これにより「絶対に守るべき制約」と「進化してよい部分」を分離できる  

### 2. Intelligence Neutral（知性中立）

Axiom は「どう考えるか」を定義しない。  
「状態をどう表現し、どう証明するか」だけを定義する。

- LLM の推論ロジックを持たない  
- モデル固有の仮定を入れない  
- どの LLM・どのエージェントでも共通で使える基盤になる  

### 3. マルチエージェントでの「制約の相互監視」

不変の制約（例：他者プログラムへの攻撃禁止など）を、  
LLM が他の LLM に対して検証・抑制できる構造を目指している。

- 監視 LLM をラウンド制で交代させる設計  
- 時差リセットによる腐敗防止  
- 企業向けにはブラックボックス化も可能な汎用性  

### 4. 差異縮小を「目に見える形」にする

技術的に差異を消すだけでなく、  
誰が見ても「ズレていない」と確認できることを重視している。

- **Golden Vectors** によるクロス言語ハッシュ完全一致の証明  
- 手書き決定論シリアライザ（言語デフォルトの JSON に依存しない）  
- `timestamp_ns` の文字列固定、浮動小数点の正規化など、細部まで決定論を徹底  

> 「見えなければ誰も信じない」  
> だからこそ、差異縮小を**証明可能な形**で公開していく。

---

## Architecture

```
Applications
     │
  LRP / PSS / DCK
     │
  Capsule（可変・成長）
     │
┌────┴────┐
▼         ▼
ACP       PLP
State     State
Integrity Representation
（不変）   （表現）
     │
Runtime / Reality
```

| Layer | Role |
|-------|------|
| **ACP** | いつ・どこから・どう変化したか・誰が証明したか |
| **PLP** | 状態が「何であるか」（粒子・幾何・ダイナミクス） |
| **Capsule** | 可変の実行時状態・観測・成長 |
| **PSS** | 問題規格の定義 |
| **DCK** | 共通基盤・差分収束 |
| **LRP** | 推論を観測可能な状態遷移として扱う |
| **UPR** | プロトコル実行基盤（知性を持たない） |

ACP can wrap any state (robot pose, transaction, sensor, LLM internal state, …).  
PLP is the first native state-representation profile of ACP.

---

## Current Status

| Module | Status |
|--------|--------|
| **ACP v1.1.0** | Stable Reference（Golden Vectors 10/10 PASS） |
| **PLP Capsule v1.1.3** | Stable（Rust / Python クロス言語一致確認済み） |
| **UPR v1.2** | Stable |
| PSS / LRP / DCK | 進行中 |

特に重要な成果：

- 同一入力に対して **Rust と Python でバイト完全一致** の Canonical Hash を達成  
- 言語差異による状態ズレを、プロトコルレベルで解消済み  

### Rust companion

[`axiomFrameworkRUSTv1.5/`](axiomFrameworkRUSTv1.5/) に以下を収録：

- `plp_capsule_v1_1_3.rs` — 手書き決定論シリアライザ + Golden Hash 固定  
- `acp_v1_1_0_reference.rs` — ACP 規範的参照実装  
- `PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md` — 10 ケース（空・複数 Observer・Added/Modified/Removed・日本語・制御文字）  

---

## Key Documents

- [ACP SPECIFICATION (RFC-AXIOM-0001) v1.1.0](docs/AXIOM_COMMON_PROTOCOL_SPECIFICATION.md)
- [ACP Overview](docs/AXIOM_COMMON_PROTOCOL.md)
- [ACP Roadmap](docs/AXIOM_COMMON_PROTOCOL_ROADMAP.md)
- [Golden Test Vectors (ACP)](tests/vectors/README.md)
- [PLP Capsule Golden Vectors v1.1.3](axiomFrameworkRUSTv1.5/PLP_CAPSULE_GOLDEN_VECTORS_v1_1_3.md)
- [UPR v1.2](docs/UPR_v1.2_Specification.md)

---

## Repository Structure

```
Axiom-Framework/
├── README.md
├── ROADMAP.md
├── LICENSE
├── docs/                          # ACP / UPR specs
├── src/
│   ├── axiom/                     # ACP + UPR (Python)
│   └── modules/                   # PLP kernel / capsule, DCK, …
├── tests/vectors/                 # ACP Golden Vectors
└── axiomFrameworkRUSTv1.5/        # Rust reference + Capsule Golden Vectors
```

---

## このフレームワークが目指すもの / Goals

単なる「便利なライブラリ」ではなく、  
**異なる言語・異なる LLM・異なるエージェントが、同じ状態を共有し、同じ制約を守れる共通基盤**を作ること。

1. 物理的なズレ（ハッシュ・シリアライズ）を消す  
2. 意味的なズレを可視化し、徐々に縮小する  
3. 不変の制約を、LLM 同士で相互に監視できる構造を提供する  

---

## License

**[AXIOM Framework Research License v1.0](LICENSE)**

- 個人・学術・教育・非営利利用可  
- 帰属表示必須  
- 軍事・有害利用禁止  
- 商業利用は別途許諾  
- DCK は MIT（別コンポーネント）  

---

**Axiom Framework**  
不変をプロトコルで固定し、成長を Capsule で許す。  
言語と知性を超えた、共通の状態基盤を目指して。
