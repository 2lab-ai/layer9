# "이 코드 어디에 둬야 해?" 문제

## 핵심 통찰
좋은 아키텍처는 코드의 집 주소가 명확하다. 나쁜 아키텍처는 노숙자를 양산한다.

## 명확한 주소 체계
```
새 기능: 사용자 포인트 계산

질문 체크리스트:
1. 순수 비즈니스 로직? → L1_domain/
2. 다른 도메인과 협업? → L2_services/  
3. 외부 노출 필요? → L3_api/
```

## 모호함의 증상
- utils/ 폴더가 거대함
- helpers/ 가 쓰레기통
- "일단 여기 두고..."
- 같은 로직이 3군데 중복

## HAF의 해답
각 계층은 명확한 책임:
- L1: WHAT (무엇)
- L2: HOW (어떻게)
- L3: WHEN/WHERE (언제/어디서)

## 실전 예시
```
포인트 계산 로직 → L1 (순수 비즈니스)
포인트 이력 조회 → L2 (서비스 조율)
포인트 API 엔드포인트 → L3 (외부 인터페이스)
```

## 핵심 원칙
"If you don't know where it goes, your layers are wrong."

#placement #clarity #architecture #decision