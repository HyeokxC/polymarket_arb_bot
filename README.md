# Polymarket Arbitrage Bot

> **15분 만기 암호화폐 시장 무위험 차익거래 봇 (Hybrid Maker-Taker Strategy)**

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-15%2F15-brightgreen.svg)](https://github.com)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## 개요

Polymarket 15분 만기 암호화폐 시장에서 Maker Rebate와 Taker Fee 구조의 차이를 활용한 하이브리드 차익거래 봇입니다.

### 핵심 전략

- 저가 구간($0.00 ~ $0.30)에 지정가 매수(Maker)를 배치하여 진입 비용 최소화
- 체결 즉시 반대 포지션을 시장가(Taker)로 매수하여 양방향 포지션 합계 비용을 $1.00 미만으로 확정
- 정확한 Polymarket 수수료 공식 적용: `fee_rate = 2 × price × (1 - price) × 0.0312`

### 수익 방정식

```
Profit = Q × [1 - {(P_maker - R_m) + (P_taker + F_t(P_taker))}]
```

---

## 기능

✅ **완료된 기능 (Phase 1-7)**

- [x] 환경변수 기반 설정 관리
- [x] WebSocket 실시간 오더북 수신
- [x] 정확한 수수료 계산 (Polymarket 공식 테이블)
- [x] BTreeMap 기반 로컬 오더북
- [x] 차익거래 기회 감지 엔진
- [x] ECDSA 트랜잭션 서명 (k256)
- [x] Limit/Market Order API
- [x] Fill 모니터링 및 즉시 헷징
- [x] Kill Switch (일일 손실 한도, 연속 실패 감지)

🚧 **진행 중 (Phase 8-10)**

- [ ] 시뮬레이션 모드
- [ ] TUI 모니터링 대시보드 (ratatui)
- [ ] 로그 파일 저장 (일별 거래 기록 CSV)

---

## 설치

### 요구사항

- Rust 1.75+ ([설치](https://rustup.rs/))
- macOS / Linux

### 빌드

```bash
cd polymarket_arb_bot
cargo build --release
```

---

## 설정

### 1. 환경변수 파일 생성

```bash
cp .env.example .env
```

### 2. `.env` 파일 편집

```bash
# Polymarket API Credentials
POLYMARKET_API_KEY=your_api_key_here
POLYMARKET_SECRET=your_secret_here

# Strategy Parameters
MAX_MAKER_PRICE=0.30
MIN_PROFIT_MARGIN=0.005
MAKER_REBATE_RATE=0.0005

# Risk Management
DAILY_LOSS_LIMIT=100.0
KILL_SWITCH_LOSS_THRESHOLD=0.02
```

---

## 실행

### 개발 모드

```bash
RUST_LOG=debug cargo run
```

### 릴리즈 모드

```bash
./target/release/polymarket_arb_bot
```

### 로그 레벨 조정

```bash
# 전체 디버그
RUST_LOG=debug cargo run

# 특정 모듈만 디버그
RUST_LOG=polymarket_arb_bot=info,strategy=debug cargo run
```

---

## 테스트

```bash
# 전체 테스트 실행
cargo test

# 특정 모듈 테스트
cargo test strategy::

# 테스트 출력 표시
cargo test -- --nocapture
```

**현재 테스트 상태:** ✅ 15/15 통과

---

## 아키텍처

```
┌─────────────────────────────────────────────────────────────┐
│                    Rust Low-Latency Bot                     │
├─────────────────┬─────────────────┬─────────────────────────┤
│  Market Data    │  Strategy       │  Execution              │
│  Adapter        │  Engine         │  Management             │
├─────────────────┼─────────────────┼─────────────────────────┤
│ - WebSocket     │ - Local Order   │ - Tx Signer            │
│   Client        │   Book (LOB)    │ - HTTP Client          │
│ - SIMD-JSON     │ - Arb Logic     │ - Order Submission     │
│   Parser        │ - Risk Check    │ - Fill Monitoring      │
└─────────────────┴─────────────────┴─────────────────────────┘
         │                 │                   │
         └────── SPSC Channel ──────┴───── SPSC Channel ──────┘
```

### 핵심 모듈

| 모듈 | 책임 | 파일 |
|------|------|------|
| **Config** | 환경변수 로드 | `src/config/mod.rs` |
| **Market Data** | WebSocket + 파싱 | `src/market_data/` |
| **Strategy** | 오더북 + 차익거래 | `src/strategy/` |
| **Execution** | 서명 + 주문 API | `src/execution/` |
| **Risk** | Kill Switch | `src/risk/` |

---

## 수수료 테이블

| Price | Effective Rate | Example ($100) |
|-------|----------------|----------------|
| $0.01 | 0.00% | $0.00 |
| $0.10 | 0.20% | $0.02 |
| $0.30 | 1.10% | $0.33 |
| **$0.50** | **1.56% (MAX)** | **$0.78** |
| $0.70 | 1.10% | $0.77 |
| $0.99 | 0.00% | $0.00 |

---

## 리스크 관리

### Kill Switch 트리거 조건

1. **일일 손실 한도 도달** - 설정된 한도 초과 시
2. **연속 3회 실패** - 3번 연속 손실 거래 시
3. **단일 거래 손실 임계값** - 한 거래에서 2% 이상 손실 시

### 비상 대응

Kill Switch 활성화 시:
- 모든 신규 주문 중단
- 열린 포지션 시장가 청산
- 로그에 경고 메시지 기록

---

## 성능 최적화

- **BTreeMap** - 가격 레벨 자동 정렬 (O(log n) 조회)
- **Decimal** - 금액 계산 정확도 보장
- **SPSC Channel** - Lock-free 메시지 패싱
- **Keep-Alive HTTP** - TCP 재사용으로 지연 감소

---

## 라이선스

MIT License

---

## 면책 조항

⚠️ **이 소프트웨어는 교육 목적으로만 제공됩니다.**

- 실제 자금으로 사용 전 충분한 테스트 필수
- 암호화폐 거래는 높은 리스크를 수반합니다
- 개발자는 손실에 대해 책임지지 않습니다

---

## 기여

이슈 및 Pull Request 환영합니다!

---

**Built with ❤️ using Rust**
