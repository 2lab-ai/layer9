# 메모리 계층과 소프트웨어 계층의 유사성

## 핵심 통찰
컴퓨터 메모리 계층이 효율적인 이유와 소프트웨어가 그래야 하는 이유는 동일하다.

## 메모리 계층
```
Registers (bytes, nanoseconds)
    ↓
L1 Cache (KB, nanoseconds)
    ↓
L2 Cache (MB, microseconds)
    ↓
RAM (GB, milliseconds)
    ↓
Disk (TB, milliseconds+)
```

## 왜 작동하는가?
- 각 층은 이웃하고만 대화
- 레지스터는 디스크를 모름
- 디스크는 L1 캐시를 모름
- 정보는 계층을 통해 전달

## 소프트웨어 적용
```
L1: Database queries
    ↓
L2: Services
    ↓
L3: Business Logic
    ↓
L4: Platform
    ↓
L5: Strategy
```

## 핵심 원리
"아무도 속이지 않는 전화 게임"
각 층이 자신의 역할만 충실히 하면 전체가 작동한다.

#memory #hierarchy #analogy #principle