# Iroha Native Permission Enforcement for d-OSPv2

## Technical Specification v1.0

---

## 1. Executive Summary

This document specifies the implementation of blockchain-level permission enforcement for d-OSPv2 using Hyperledger Iroha 2's native role-based access control (RBAC) system. The current implementation relies on application-layer permission checks, which lack blockchain-level guarantees, auditability, and trustless enforcement required for scientific data management compliance.

### 1.1 Problem Statement

| Aspect | Current State | Required State |
|--------|--------------|----------------|
| Permission Enforcement | Application-layer (Axum middleware) | Blockchain-layer (Iroha validators) |
| Enforcement Coverage | 44.4% (4/9 handlers) | 100% |
| Audit Trail | None for permissions | Immutable on-chain |
| Trust Model | Trusted application | Trustless (consensus-verified) |
| Compliance | Not verifiable | Blockchain-proven |

### 1.2 Solution Overview

Implement Iroha 2 native permission tokens and roles to enforce all d-OSPv2 operations at the blockchain level, eliminating the need for application-layer permission checks and providing immutable permission grant/revocation history.

---

## 2. Architecture

### 2.1 Current vs. Target Architecture

#### Current Architecture (Application-Layer)

```
┌─────────────┐     ┌──────────────┐     ┌─────────────────┐
│   Client    │────►│  Axum API   │────►│ Permission Check│
│             │     │              │     │ (in code)       │
└─────────────┘     └──────────────┘     └─────────────────┘
                           │                    │
                           ▼                    ▼
                    ┌──────────────┐     ┌─────────────────┐
                    │    Iroha    │◄────│  Role stored    │
                    │  (data)     │     │  as metadata    │
                    └──────────────┘     └─────────────────┘
```

**Limitations:**
- Permission checks can be bypassed
- No blockchain-verifiable permission grants
- Application must be trusted
- Gaps in enforcement coverage

#### Target Architecture (Blockchain-Layer)

```
┌─────────────┐     ┌──────────────┐     ┌─────────────────┐
│   Client    │────►│  Axum API   │────►│ Submit Transaction│
│             │     │  (passthrough)│     │ (no check)      │
└─────────────┘     └──────────────┘     └─────────────────┘
                                                  │
                                                  ▼
                                         ┌─────────────────┐
                                         │  Iroha Validators│
                                         │  (Permission    │
                                         │   Judge)        │
                                         └─────────────────┘
                                                  │
                                    ┌───────────┴───────────┐
                                    │                       │
                              ┌─────▼─────┐          ┌──────▼──────┐
                              │  ALLOW    │          │    DENY     │
                              │(validated) │          │(rejected)   │
                              └─────┬─────┘          └──────┬──────┘
                                    │                       │
                                    ▼                       ▼
                              ┌────────────┐          ┌────────────┐
                              │ Blockchain │          │  Error to  │
                              │ (execute)  │          │  client    │
                              └────────────┘          └────────────┘
```

**Advantages:**
- All operations validated by consensus
- Immutable permission audit trail
- No trust required in application
- 100% enforcement coverage

---

## 3. Iroha 2 Permission System

### 3.1 Core Concepts

#### Permission Token
A single, granular permission that grants the right to perform a specific operation.

```rust
// Example: Permission to register assets in a domain
CanRegisterAsset { asset_definition_id: Id }
```

#### Role
A named collection of PermissionTokens that can be granted to accounts.

```rust
// Example: PI role combining multiple permissions
Role::new("dosp_pi")
    .add_permission(CanRegisterAsset::new())
    .add_permission(CanUnregisterAsset::new())
    .add_permission(CanSetKeyValueInDomain::new())
```

#### Grant Instruction (ISI)
Iroha Special Instruction that assigns a role or permission token to an account.

```rust
let grant = GrantBox::new(role_id, target_account_id);
```

#### Permission Validator (Judge)
Iroha component that validates whether an operation is permitted.

| Validator | Behavior |
|-----------|----------|
| `AtLeastOneAllow` | Succeed if ANY validator allows |
| `NoDenies` | Succeed if NO validator denies |
| `NoDeniesAndAtLeastOneAllow` | Both conditions must pass |
| `AllowAll` | Allow all (testing only) |
| `DenyAll` | Deny all (testing only) |

---

## 4. d-OSPv2 Permission Model

### 4.1 d-OSP Roles → Iroha Roles Mapping

| d-OSP Role | Iroha Role | Permissions |
|------------|------------|-------------|
| PI | `dosp_pi` | Full DMP control |
| DataSteward | `dosp_data_steward` | Dataset management |
| Contributor | `dosp_contributor` | Dataset creation |
| Reviewer | `dosp_reviewer` | Read access (no special permissions) |

### 4.2 Operations → Required Permissions

| Operation | Handler | Iroha Permission Token |
|-----------|---------|------------------------|
| Create Dataset NFT | `create_dataset_nft` | `CanRegisterAsset` |
| Delete Dataset NFT | `delete_dataset_nft` | `CanUnregisterAsset` |
| Transfer Dataset NFT | `transfer_dataset_nft` | `CanTransferAsset` |
| Create maDMP | `create_madmp` | `CanSetKeyValueInDomain` |
| Grant Role | `grant_role` | `can_grant` (system) |
| Revoke Role | `revoke_role` | `can_revoke` (system) |
| Invite User | `invite_user` | `CanSetKeyValueInUserAccount` |
| Delete DMP | `delete_dmp` | `CanUnregisterDomain` |
| Query Role | `get_role` | N/A (query) |

---

## 4.3 Iroha Compatibility

**Iroha Commit:** `32144e92f72b062743aeb8adedb9b7ffadf57e4b`

The current Iroha version fully supports the proposed permission model using **only generic, pre-configured permission tokens**. No custom permission tokens need to be registered.

### Verified Permission Tokens

| d-OSP Operation | Iroha Permission Token | Status |
|----------------|------------------------|--------|
| Create Dataset NFT | `can_create_asset` / `can_register_asset` | ✅ Verified |
| Delete Dataset NFT | `can_unregister_asset` | ✅ Verified |
| Transfer Dataset NFT | `can_transfer_asset` | ✅ Verified |
| Create maDMP | `can_set_key_value_in_domain` | ✅ Verified |
| Delete DMP | `can_unregister_domain` | ✅ Verified |
| Invite User | `can_set_key_value_in_user_account` | ✅ Verified |
| Grant Role | `can_grant` (system) | ✅ Verified |

### Advantages

1. **No custom permission registration** - Uses pre-built tokens
2. **Tested and stable** - Generic tokens are well-documented
3. **Compatible** - Matches current Iroha version
4. **Simple genesis** - Only role definitions needed

### Reference

- [Iroha 2 Permissions Documentation](https://docs.iroha.tech/blockchain/permissions.html)
- [Iroha 2 Permission Reference](https://docs.iroha.tech/reference/permissions.html)

---

## 5. Genesis Configuration

Based on the existing `genesis.json` format, here are the required additions for d-OSPv2 roles and permissions:

### 5.1 Complete Genesis Additions

```json
{
  "chain": "00000000-0000-0000-0000-000000000000",
  "executor": "executor.wasm",
  "parameters": {
    "sumeragi": {
      "block_time_ms": 2000,
      "commit_time_ms": 4000,
      "max_clock_drift_ms": 1000
    },
    "block": {
      "max_transactions": 512
    },
    "transaction": {
      "max_instructions": 4096,
      "smart_contract_size": 4194304
    },
    "executor": {
      "fuel": 55000000,
      "memory": 55000000,
      "execution_depth": 3
    },
    "smart_contract": {
      "fuel": 55000000,
      "memory": 55000000,
      "execution_depth": 3
    }
  },
  "instructions": [
    // ===== EXISTING: DOSP DOMAIN AND ACCOUNTS =====
    // {"note":"Store private_key securely - it cannot be recovered","private_key":"1300200892f6709c7460af30c5944b841bda84954d79b1c04799b139126d86083374af","public_key":"ed0120328e86ca218aaeba56909bf943ea5b2c9e28229c191e2455a725bf153de2c7d7"}

    {
      "Register": {
        "Domain": {
          "id": "dosp",
          "logo": null,
          "metadata": {}
        }
      }
    },
    {
      "Register": {
        "Account": {
          "id": "ed0120328e86ca218aaeba56909bf943ea5b2c9e28229c191e2455a725bf153de2c7d7@dosp",
          "metadata": {
            "dmp_role": "pi",
            "global_id": "https://orcid.org/0000-0001-2345-6789"
          }
        }
      }
    },
    
    // ===== NEW: REGISTER ROLES =====
    {
      "Register": {
        "Role": {
          "id": "dosp_pi",
          "permissions": [
            "can_register_asset",
            "can_unregister_asset",
            "can_transfer_asset",
            "can_set_key_value_in_domain",
            "can_remove_key_value_in_domain",
            "can_set_key_value_user_account",
            "can_remove_key_value_user_account",
            "can_unregister_domain",
            "can_unregister_account"
          ]
        }
      }
    },
    {
      "Register": {
        "Role": {
          "id": "dosp_data_steward",
          "permissions": [
            "can_register_asset",
            "can_unregister_asset",
            "can_transfer_asset",
            "can_set_key_value_in_domain",
            "can_remove_key_value_in_domain"
          ]
        }
      }
    },
    {
      "Register": {
        "Role": {
          "id": "dosp_contributor",
          "permissions": [
            "can_register_asset",
            "can_set_key_value_in_domain"
          ]
        }
      }
    },
    {
      "Register": {
        "Role": {
          "id": "dosp_reviewer",
          "permissions": []
        }
      }
    },
    
    // ===== NEW: GRANT PI ROLE TO ADMIN =====
    {
      "Grant": {
        "Role": {
          "role_id": "dosp_pi",
          "destination_id": "admin@dosp"
        }
      }
    },
    
    // ===== EXISTING: SAMPLE ACCOUNTS (wonderland) =====
    {
      "Register": {
        "Domain": {
          "id": "wonderland",
          "logo": null,
          "metadata": {}
        }
      }
    },
    {
      "Register": {
        "Account": {
          "id": "ed0120CE7FA46C9DCE7EA4B125E2E36BDB63EA33073E7590AC92816AE1E861B7048B03@wonderland",
          "metadata": {}
        }
      }
    },
    {
      "Register": {
        "AssetDefinition": {
          "id": "rose#wonderland",
          "spec": {
            "scale": null
          },
          "mintable": "Infinitely",
          "logo": null,
          "metadata": {}
        }
      }
    },
    {
      "Mint": {
        "Asset": {
          "object": "13",
          "destination": "rose##ed0120CE7FA46C9DCE7EA4B125E2E36BDB63EA33073E7590AC92816AE1E861B7048B03@wonderland"
        }
      }
    },
    
    // ===== EXISTING: GARDEN DOMAIN =====
    {
      "Register": {
        "Domain": {
          "id": "garden_of_live_flowers",
          "logo": null,
          "metadata": {}
        }
      }
    },
    {
      "Register": {
        "AssetDefinition": {
          "id": "cabbage#garden_of_live_flowers",
          "spec": {
            "scale": null
          },
          "mintable": "Infinitely",
          "logo": null,
          "metadata": {}
        }
      }
    }
  ],
  "wasm_dir": "libs",
  "wasm_triggers": [],
  "topology": []
}
```

### 5.2 Role Definitions

| Role ID | Permissions | Description |
|---------|------------|-------------|
| `dosp_pi` | can_register_asset, can_unregister_asset, can_transfer_asset, can_set_key_value_in_domain, can_remove_key_value_in_domain, can_set_key_value_user_account, can_remove_key_value_user_account, can_unregister_domain, can_unregister_account | Full DMP control - can create/delete datasets, manage users, delete domains |
| `dosp_data_steward` | can_register_asset, can_unregister_asset, can_transfer_asset, can_set_key_value_in_domain, can_remove_key_value_in_domain | Dataset management - can create/delete/transfer datasets, manage DMP metadata |
| `dosp_contributor` | can_register_asset, can_set_key_value_in_domain | Dataset creation - can create datasets and set metadata |
| `dosp_reviewer` | (none) | Read-only access |

---

## 6. Implementation Plan

### 6.1 Phase 1: Genesis Configuration

**Timeline:** Week 1  
**Objective:** Configure Iroha network with d-OSP roles

1. Update `genesis.json` with role registrations
2. Add Grant instruction to assign PI role to admin
3. Deploy updated genesis to network

### 6.2 Phase 2: Core Implementation

**Timeline:** Week 2  
**Objective:** Implement permission module and modify handlers

#### 6.2.1 Create Permission Module

New file: `src/common/iroha_permissions.rs`

```rust
use iroha::data_model::prelude::*;
use crate::common::v2_roles::DmpRole;
use crate::error::AppError;

/// Maps d-OSP roles to Iroha role IDs
pub fn get_iroha_role_id(dmp_role: &DmpRole) -> RoleId {
    match dmp_role {
        DmpRole::PI => RoleId::from_str("dosp_pi").unwrap(),
        DmpRole::DataSteward => RoleId::from_str("dosp_data_steward").unwrap(),
        DmpRole::Contributor => RoleId::from_str("dosp_contributor").unwrap(),
        DmpRole::Reviewer => RoleId::from_str("dosp_reviewer").unwrap(),
    }
}

/// Converts d-OSP role string to Iroha role ID
pub fn role_string_to_iroha(role: &str) -> Result<RoleId, AppError> {
    let dmp_role = DmpRole::from_str(role)
        .map_err(|_| AppError::ValidationError {
            field: "role".to_string(),
            message: format!("Invalid role: {}", role),
        })?;
    Ok(get_iroha_role_id(&dmp_role))
}
```

#### 6.2.2 Modify Role Grant Handler

File: `src/handlers/roles_v2.rs`

**Before (current):**
```rust
// Store role in domain metadata
let set_key = format!("role_{}", public_key);
let set_value = serde_json::json!(role);
let isi = SetKeyValue::domain(domain_id, set_key.parse()?, Json::new(set_value));
```

**After (target):**
```rust
// Grant Iroha role via blockchain
let iroha_role_id = role_string_to_iroha(&role)?;
let grant_isi = GrantBox::new(iroha_role_id.clone(), target_account_id.clone());
let tx = Transaction::new(
    domain_id.clone(),
    vec![grant_isi.into()],
    100_000,
).sign(key_pair)?;

iroha.submit_blocking(tx)?;
info!("Role '{}' granted to {} via Iroha role {}", role, target_account_id, iroha_role_id);
```

#### 6.2.3 Modify Role Revoke Handler

**Before:**
```rust
// Remove from domain metadata
let remove_key = format!("role_{}", public_key);
let isi = RemoveKeyValue::domain(domain_id, remove_key.parse()?);
```

**After:**
```rust
// Revoke Iroha role
let iroha_role_id = role_string_to_iroha(&role)?;
let revoke_isi = RevokeBox::new(iroha_role_id.clone(), target_account_id.clone());
let tx = Transaction::new(
    domain_id.clone(),
    vec![revoke_isi.into()],
    100_000,
).sign(key_pair)?;

iroha.submit_blocking(tx)?;
info!("Role '{}' revoked from {} via Iroha", role, target_account_id);
```

### 6.3 Phase 3: Handler Modifications

**Timeline:** Week 2-3  
**Objective:** Remove application-layer permission checks

#### 6.3.1 Dataset NFT Handlers

File: `src/handlers/dataset_nft_v2.rs`

| Handler | Change |
|---------|--------|
| `create_dataset_nft` | Remove role check; rely on `CanRegisterAsset` |
| `delete_dataset_nft` | Remove role check; rely on `CanUnregisterAsset` |
| `transfer_dataset_nft` | Remove role check; rely on `CanTransferAsset` |

#### 6.3.2 DMP Handlers

File: `src/handlers/dmp_v2.rs`

| Handler | Change |
|---------|--------|
| `create_dmp` | Remove role check; rely on `CanSetKeyValueInDomain` |
| `delete_dmp` | Remove role check; rely on `CanUnregisterDomain` |
| `invite_user` | Remove role check; rely on `CanSetKeyValueInUserAccount` |

#### 6.3.3 maDMP Handlers

File: `src/handlers/madmp_v2.rs`

| Handler | Change |
|---------|--------|
| `create_madmp` | Remove role check; rely on `CanSetKeyValueInDomain` |

### 6.4 Phase 4: Testing

**Timeline:** Week 3  
**Objective:** Validate implementation

| Test | Description |
|------|-------------|
| Unit Tests | Role mapping tests |
| Integration Tests | Grant/revoke flows |
| End-to-End | Full workflow with permission enforcement |

---

## 7. Error Handling

### 7.1 Permission Denied Error

```rust
match result {
    Ok(_) => (StatusCode::OK, Json(response)).into_response(),
    Err(e) if e.to_string().contains("permission") => {
        (StatusCode::FORBIDDEN, Json(ApiResponse::error(
            "permission_denied",
            "Account lacks required permission for this operation"
        ))).into_response()
    }
    Err(e) => {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error(
            "blockchain_error",
            e.to_string()
        ))).into_response()
    }
}
```

### 7.2 Error Response Format

```json
{
  "success": false,
  "error": {
    "code": "permission_denied",
    "message": "Account lacks required permission for this operation",
    "details": {
      "required_permission": "can_register_asset",
      "account_id": "ed0120...@dosp"
    },
    "hint": "Contact the DMP PI to request dataset creation permissions"
  }
}
```

---

## 8. Rollback Strategy

### 8.1 Gradual Rollout

1. **Phase A:** Deploy with both application-layer AND Iroha checks
2. **Phase B:** Remove application-layer checks (opt-in per endpoint)
3. **Phase C:** Full Iroha-only enforcement

### 8.2 Emergency Rollback

If issues occur:
1. Revert code changes
2. Re-enable application-layer checks
3. Iroha roles remain but unused (no harm)

---

## 9. Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Genesis config error | Medium | High | Test in development first |
| Permission gap | Low | High | Comprehensive test coverage |
| Migration breaks existing | Medium | High | Gradual rollout |
| Validator misconfiguration | Low | High | Use standard AtLeastOneAllow |

---

## 10. Timeline & Effort

| Phase | Duration | Effort |
|-------|----------|--------|
| Phase 1: Genesis | 1 week | 2 days |
| Phase 2: Core | 1 week | 3 days |
| Phase 3: Handlers | 1 week | 3 days |
| Phase 4: Testing | 1 week | 2 days |
| **Total** | **4 weeks** | **~10 days** |

---

## 11. Success Criteria

1. ✅ All 9 handler operations validated by Iroha
2. ✅ 100% permission enforcement coverage
3. ✅ Permission grants/revokes recorded on blockchain
4. ✅ Unauthorized operations rejected at blockchain level
5. ✅ All existing tests pass
6. ✅ Permission evaluation reports 100% coverage

---

## 12. References

- [Iroha 2 Permissions Documentation](https://docs.iroha.tech/blockchain/permissions.html)
- [Iroha 2 Permission Reference](https://docs.iroha.tech/reference/permissions.html)
- [Iroha 2 WASM Smart Contracts](https://docs.iroha.tech/blockchain/wasm.html)
- [Iroha 2 Triggers](https://docs.iroha.tech/blockchain/triggers.html)
- [d-OSPv2 Permission Evaluation Report](../reports/permission_grants_evaluation.md)
- [Iroha Commit: 32144e92f72b062743aeb8adedb9b7ffadf57e4b](https://github.com/hyperledger/iroha/commit/32144e92f72b062743aeb8adedb9b7ffadf57e4b)

---

## 13. Appendix: Permission Tokens Reference

### Generic Iroha Tokens (Sufficient for d-OSPv2)

| Token | Category | Operation |
|-------|----------|-----------|
| `can_register_asset` | Asset | Register new assets |
| `can_unregister_asset` | Asset | Remove assets |
| `can_transfer_asset` | Asset | Transfer assets |
| `can_mint_asset` | Asset | Mint new tokens |
| `can_burn_asset` | Asset | Burn tokens |
| `can_set_key_value_in_domain` | Domain | Set domain metadata |
| `can_remove_key_value_in_domain` | Domain | Remove domain metadata |
| `can_unregister_domain` | Domain | Delete domain |
| `can_set_key_value_user_account` | Account | Set account metadata |
| `can_remove_key_value_user_account` | Account | Remove account metadata |
| `can_unregister_account` | Account | Delete account |

### d-OSP Custom Tokens (If Needed Later)

| Token | Parameters | Purpose |
|-------|------------|---------|
| `dosp_grant_role` | domain_id | Grant role in specific domain |

---

**Document Version:** 1.0  
**Status:** Ready for Implementation  
**Created:** 2026-02-18  
**Author:** d-OSPv2 Engineering Team
