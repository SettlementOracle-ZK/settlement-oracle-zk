# Product Requirements Document (PRD)

**Product Name:** SettlementOracle ZK  
**Product Type:** Automated Settlement Platform (On-chain) + Oracle Aggregator + Smart Contract Executor + ZK Proof Validator  
**Target Users:** Insurers (Actuaries, Risk Managers, Pricing Analysts, Auditors)

## Overview

An infrastructure for parametric insurance that automates claim settlement. The platform collapses month-long processes into seconds, using oracles (Web3) to verify real-world events and trigger payments via smart contracts. The system generates a zero-knowledge (ZK) proof attesting the validity of the execution trigger, ensuring full transparency and trust between insurer and insured.

---

## 1. Problem Statement

**Current situation:** The claim settlement process is slow, bureaucratic, and expensive, relying on manual inspections and expert assessors. This creates friction with customers and high operational costs. Insurers fail to offer instant payout products due to a lack of reliable bridges between real-world data and financial execution.

**Primary user pain points:**
- Excessive delay (weeks or months) before claim payouts are received
- Lack of trust in the impartiality of the claims audit process
- High operational costs for expert assessment and dispute management

**Business impact of solving this:**
- Up to 90% reduction in operational cost for claims processing
- Extreme competitive differentiation with real-time settlement (Instant Payouts)
- Fraud risk elimination via parametric verification through cryptographic oracles

---

## 2. Proposed Solution

**Solution overview:** SettlementOracle ZK integrates market oracles (Chainlink, Pyth) with configurable smart contracts. When oracle data attests an event (e.g., rainfall < X mm), the contract automatically triggers fund transfers. A ZK proof guarantees that the business rule was executed exactly as agreed, with no manual intervention.

### Detailed Use Cases

- **Parametric Agricultural Insurance:** A contract monitors rainfall indices via satellite/oracle. If drought is cryptographically confirmed, payment is sent instantly to the farmer's wallet — no field inspection required.
- **Asset Price Guarantee:** Protection against sharp drops in commodities or currencies. When the strike price verified by the price oracle is hit, the contract automatically settles the difference.
- **Flight Delay Insurance:** A contract monitors aviation data APIs via oracles. If a flight exceeds a pre-set delay threshold, the contract automatically releases the payout to the passenger's wallet, eliminating claims bureaucracy.
- **Service Availability Insurance (SLA):** Automatic protection for companies against critical cloud service downtime. If uptime falls below a limit monitored by a performance oracle, the contract processes a proportional refund automatically.
- **Logistics and Cargo Insurance:** Delivery status monitoring via IoT devices integrated with oracles. If status changes to "lost" or "damaged" at a validated logistics hub, cargo payment is released immediately to the insured.

### User Stories

- As a product manager, I want to configure trigger rules based on public oracles to automate payments without manual errors.
- As an insured party, I want to receive my payout in seconds once an event is publicly verified, without bureaucracy.
- As an auditor, I want to validate via ZK Proof that each payment was triggered by intact data from the oracle.

---

## 3. Product Requirements

### Functional Requirements

- **Oracle Data Connector:** Deep integration with Chainlink Functions and Pyth for reliable off-chain data consumption.
- **Smart Contract Engine:** Factory of smart contracts managing premium escrow and payout logic.
- **ZK Validation:** Validation of business rule execution against oracle data.
- **Payout Automator:** On-chain transaction execution system with support for multiple networks (L2s).
- **ZK Generator (Prover):** Module that receives inputs and outputs, generating the ZK-SNARK.
- **API Gateway:** Secure endpoint for insurer queries.

### Example Response Payload

```json
{
  "asset_class": "agriculture_climate",
  "risk_score": 85.4,
  "scale": "0-100",
  "model_confidence": "92%",
  "timestamp": "2026-05-19T14:42:00Z",
  "zk_proof": {
    "hash": "0xABC123...",
    "verification_url": "https://api.riskoracle.com/verify/0xABC..."
  }
}
```

### Design/UX Requirements

- **Settlement Dashboard:** Real-time view of active contracts, monitored events, and settlement status.
- **Transaction and Proof Explorer:** Interface for insured parties and auditors to verify ZK hash and blockchain confirmation.

### Technical Requirements & Technology Suggestions

| Layer | Technology |
|-------|------------|
| Frontend | React / Next.js |
| Smart Contracts | Solidity / Rust (CosmWasm/Solana) |
| Oracles | Chainlink, Pyth Network, API3 |
| ZK Infrastructure | RISC Zero or Succinct for off-chain rule execution proofs integrated with contracts |
| Infrastructure | AWS (EC2/Lambda) and PostgreSQL |

---

## 4. Success Criteria

- **Time to Settlement:** Average time between oracle event confirmation and payout (target: < 2 minutes).
- **Automated Payout Rate:** Percentage of claims settled without any human intervention.

**Business Metrics:**
- **On-chain Volume:** Total premiums and payouts processed via platform contracts.
- **Trust Score:** Regulatory acceptance level of ZK proofs generated for audit.

---

## 5. Scope and Timeline

### MVP — IN Scope

- Integration with 1 third-party data oracle (e.g., Climate or Price)
- Escrow and automated Payout smart contract on Testnet
- Basic ZK Proof generation for contract trigger
- Web2 dashboard for claims monitoring

### MVP — OUT of Scope

- Order execution or direct modification of third-party systems (the system only suggests risk, it does not act on it)
- Large AI models (LLMs) inside the ZK circuit
- Continuous monitoring infrastructure

---

## 6. Risks and Mitigations

| Risk | Description | Mitigation |
|------|-------------|------------|
| Oracle Reliability | Dependency on external data that may be delayed or erroneous at source | Use multi-oracle aggregation and dispute (challenge) periods before final settlement |
| Smart Contract Security | Code vulnerabilities could lead to draining of premiums deposited in escrow | Rigorous audits and use of battle-tested OpenZeppelin libraries; on-chain Circuit Breaker patterns |
| Integration Risk | Insurers may not know how to interpret "Score 85" | Create excellent API documentation with case studies on mapping the 0-100 scale to actuarial calculations |

---

## Implementation Roadmap

1. **Solana Environment Setup:** Solana CLI setup, test suite, and local Rust development environment.
2. **Anchor Framework Development:** Escrow programs and Payout logic using Anchor for security and speed in the Solana ecosystem.
3. **Native Pyth Network Integration:** Data Connector focused on high-fidelity, low-latency feed consumption directly from Pyth on Solana.
4. **ZK Infrastructure for Solana:** Proof module using ZK libraries compatible with Solana's execution environment (Light Protocol or Succinct integration).
5. **API Gateway & RPC Development:** Endpoints integrated with Solana RPC nodes for state queries and transaction submission.
6. **Frontend with Solana Wallet Adapter:** Dashboard in React/Next.js with native Phantom and Solflare wallet integration via Wallet Adapter.
7. **QA and On-chain Verification:** Security audit of Anchor programs and stress testing on Solana Devnet/Testnet.
