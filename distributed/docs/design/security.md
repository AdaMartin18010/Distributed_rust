# 3.10.4 安全设计 (Security Design)

## 目录

- [3.10.4 安全设计 (Security Design)](#3104-安全设计-security-design)
  - [目录](#目录)
  - [核心概念](#核心概念)
    - [安全目标](#安全目标)
    - [安全原则](#安全原则)
  - [安全威胁模型](#安全威胁模型)
    - [威胁分类](#威胁分类)
    - [攻击面分析](#攻击面分析)
  - [认证与授权](#认证与授权)
    - [认证机制](#认证机制)
    - [授权模型](#授权模型)
  - [加密与密钥管理](#加密与密钥管理)
    - [加密类型](#加密类型)
    - [密钥管理](#密钥管理)
  - [网络安全](#网络安全)
    - [传输层安全](#传输层安全)
    - [网络隔离](#网络隔离)
    - [API安全](#api安全)
  - [数据安全](#数据安全)
    - [静态数据加密](#静态数据加密)
    - [数据脱敏](#数据脱敏)
    - [数据保留与删除](#数据保留与删除)
  - [审计与合规](#审计与合规)
    - [审计日志](#审计日志)
    - [合规框架](#合规框架)
  - [Rust实现示例](#rust实现示例)
    - [认证系统](#认证系统)
    - [授权系统](#授权系统)
    - [加密服务](#加密服务)
    - [审计日志1](#审计日志1)
    - [安全中间件](#安全中间件)
  - [测试策略](#测试策略)
    - [安全测试类型](#安全测试类型)
    - [渗透测试](#渗透测试)
    - [模糊测试](#模糊测试)
  - [性能优化](#性能优化)
    - [加密性能优化](#加密性能优化)
    - [会话管理优化](#会话管理优化)
    - [密钥轮换策略](#密钥轮换策略)
  - [最佳实践](#最佳实践)
    - [安全编码原则](#安全编码原则)
    - [安全配置](#安全配置)
    - [安全检查清单](#安全检查清单)
    - [安全事件响应](#安全事件响应)
  - [相关文档](#相关文档)
  - [参考资料](#参考资料)
    - [标准与规范](#标准与规范)
    - [工具与库](#工具与库)
    - [最佳实践1](#最佳实践1)

---

## 核心概念

安全设计是分布式系统中的关键组成部分，涵盖认证、授权、加密、审计等多个方面。

### 安全目标

**CIA三元组**：

- **机密性 (Confidentiality)**：保护数据不被未授权访问
- **完整性 (Integrity)**：确保数据不被篡改
- **可用性 (Availability)**：确保系统持续可用

**扩展目标**：

- **不可否认性 (Non-repudiation)**：防止操作者否认已执行的操作
- **认证 (Authentication)**：验证身份的真实性
- **授权 (Authorization)**：控制访问权限
- **审计 (Auditing)**：记录和监控安全事件

### 安全原则

```text
1. 最小权限原则 (Principle of Least Privilege)
   - 仅授予完成任务所需的最小权限
   
2. 深度防御 (Defense in Depth)
   - 多层安全机制，防止单点失效
   
3. 默认拒绝 (Fail Secure)
   - 发生错误时默认拒绝访问
   
4. 职责分离 (Separation of Duties)
   - 关键操作需要多方协作
   
5. 安全透明 (Security Transparency)
   - 安全机制应该对用户透明
```

---

## 安全威胁模型

### 威胁分类

```text
STRIDE威胁模型：
┌─────────────────────────────────────────────┐
│ S - Spoofing (身份欺骗)                     │
│   攻击者冒充其他用户或系统                  │
├─────────────────────────────────────────────┤
│ T - Tampering (篡改)                        │
│   未授权修改数据或代码                      │
├─────────────────────────────────────────────┤
│ R - Repudiation (抵赖)                      │
│   否认已执行的操作                          │
├─────────────────────────────────────────────┤
│ I - Information Disclosure (信息泄露)       │
│   未授权访问敏感信息                        │
├─────────────────────────────────────────────┤
│ D - Denial of Service (拒绝服务)            │
│   使系统无法正常提供服务                    │
├─────────────────────────────────────────────┤
│ E - Elevation of Privilege (权限提升)       │
│   获得超出授权范围的权限                    │
└─────────────────────────────────────────────┘
```

### 攻击面分析

**网络层攻击**：

- Man-in-the-Middle (MITM)
- DDoS攻击
- 数据包嗅探
- DNS劫持

**应用层攻击**：

- SQL注入
- XSS (跨站脚本)
- CSRF (跨站请求伪造)
- API滥用

**系统层攻击**：

- 权限提升
- 恶意软件
- 零日漏洞
- 供应链攻击

---

## 认证与授权

### 认证机制

**多因素认证 (MFA)**：

```text
认证因素：
1. 知识因素 (Something You Know)
   - 密码、PIN
   
2. 持有因素 (Something You Have)
   - Token、智能卡、手机
   
3. 生物因素 (Something You Are)
   - 指纹、面部识别、虹膜
```

**认证协议**：

- OAuth 2.0：授权框架
- OpenID Connect：身份认证层
- SAML：企业级单点登录
- JWT：无状态令牌
- mTLS：双向TLS认证

### 授权模型

**基于角色的访问控制 (RBAC)**：

```text
用户 → 角色 → 权限 → 资源

示例：
User: alice
  └─ Role: admin
      ├─ Permission: read:*
      ├─ Permission: write:*
      └─ Permission: delete:*
```

**基于属性的访问控制 (ABAC)**：

```text
策略决策基于：
- 主体属性 (Subject Attributes)
- 资源属性 (Resource Attributes)
- 环境属性 (Environment Attributes)
- 操作属性 (Action Attributes)

示例策略：
IF (user.department == "finance" AND 
    resource.type == "financial_report" AND
    time.hour >= 9 AND time.hour <= 17)
THEN ALLOW
```

**基于策略的访问控制 (Policy-Based)**：

```text
使用策略语言定义复杂的访问规则：
- Rego (Open Policy Agent)
- Cedar (AWS)
- XACML
```

---

## 加密与密钥管理

### 加密类型

**对称加密**：

- AES-256-GCM：数据加密
- ChaCha20-Poly1305：流加密
- 用途：大量数据加密

**非对称加密**：

- RSA-4096：密钥交换、数字签名
- ECDSA (P-256)：高效签名
- Ed25519：现代签名算法
- 用途：身份验证、密钥交换

**哈希函数**：

- SHA-256/SHA-512：完整性校验
- BLAKE3：高性能哈希
- Argon2：密码哈希
- 用途：数据完整性、密码存储

### 密钥管理

**密钥生命周期**：

```text
1. 生成 (Generation)
   - 使用CSPRNG生成高熵密钥
   
2. 分发 (Distribution)
   - 通过安全信道分发密钥
   
3. 存储 (Storage)
   - HSM、KMS、加密存储
   
4. 轮换 (Rotation)
   - 定期更换密钥
   
5. 撤销 (Revocation)
   - 密钥泄露时立即撤销
   
6. 销毁 (Destruction)
   - 安全删除密钥
```

**密钥派生**：

```text
HKDF (HMAC-based Key Derivation Function)：
  master_key
      ↓
  [HKDF-Extract]
      ↓
  pseudorandom_key
      ↓
  [HKDF-Expand]
      ↓
  ├─ encryption_key
  ├─ signing_key
  └─ mac_key
```

**密钥交换协议**：

- Diffie-Hellman (DH)
- Elliptic Curve DH (ECDH)
- X25519

---

## 网络安全

### 传输层安全

**TLS 1.3**：

```text
TLS握手流程：
Client                                Server
  │                                      │
  ├──── ClientHello ──────────────────→ │
  │     (支持的密码套件)                 │
  │                                      │
  │ ←──── ServerHello ──────────────────┤
  │     (选择的密码套件、证书)           │
  │                                      │
  ├──── [Change Cipher Spec] ─────────→ │
  │     [Finished]                       │
  │                                      │
  │ ←──── [Change Cipher Spec] ─────────┤
  │     [Finished]                       │
  │                                      │
  ├──── Application Data ←────────────→ │
```

**推荐密码套件**：

```text
TLS_AES_256_GCM_SHA384
TLS_CHACHA20_POLY1305_SHA256
TLS_AES_128_GCM_SHA256
```

### 网络隔离

**网络分段**：

```text
┌──────────────────────────────────────┐
│ DMZ (Demilitarized Zone)             │
│  - 面向公网的服务                    │
├──────────────────────────────────────┤
│ Application Tier                     │
│  - 应用服务器                        │
├──────────────────────────────────────┤
│ Data Tier                            │
│  - 数据库、存储                      │
└──────────────────────────────────────┘
```

**防火墙规则**：

- 白名单策略
- 最小化开放端口
- 地域限制
- 速率限制

### API安全

**API认证**：

- API密钥
- Bearer Token
- HMAC签名
- mTLS

**API速率限制**：

```text
令牌桶算法：
  Bucket (容量: 100)
     │
     ├─ 令牌生成速率: 10/秒
     │
     └─ 请求消耗令牌
        - 有令牌 → 允许请求
        - 无令牌 → 拒绝请求 (429)
```

---

## 数据安全

### 静态数据加密

**加密层级**：

```text
1. 应用层加密
   - 在应用中加密敏感字段
   - 完全控制密钥
   
2. 数据库加密
   - 透明数据加密 (TDE)
   - 列级加密
   
3. 存储层加密
   - 磁盘加密 (LUKS, BitLocker)
   - 对象存储加密 (S3 SSE)
```

**加密策略**：

```rust
// 字段级加密示例
pub struct EncryptedField<T> {
    ciphertext: Vec<u8>,
    key_id: String,
    algorithm: EncryptionAlgorithm,
    _phantom: PhantomData<T>,
}

// 选择性加密
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: EncryptedField<String>,      // 加密
    pub ssn: EncryptedField<String>,        // 加密
    pub address: EncryptedField<Address>,   // 加密
    pub created_at: DateTime<Utc>,
}
```

### 数据脱敏

**脱敏技术**：

```text
1. 遮蔽 (Masking)
   - 信用卡: **** **** **** 1234
   - 邮箱: u***@example.com
   
2. 泛化 (Generalization)
   - 年龄: 25 → 20-30岁
   - 地址: 具体门牌号 → 街道
   
3. 替换 (Substitution)
   - 使用假数据替换真实数据
   
4. 噪声添加 (Noise Addition)
   - 添加随机噪声保护隐私
   
5. K-匿名性
   - 确保每条记录至少有k-1条相似记录
```

### 数据保留与删除

**数据生命周期**：

```text
创建 → 使用 → 归档 → 删除

策略示例：
- 日志数据: 保留90天
- 用户数据: 账户删除后保留30天
- 备份数据: 保留7年（合规要求）
```

**安全删除**：

- 逻辑删除 + 定期物理删除
- 多次覆写（DoD 5220.22-M）
- 加密删除：删除加密密钥

---

## 审计与合规

### 审计日志

**审计事件**：

```text
关键事件记录：
- 认证事件 (登录、登出、认证失败)
- 授权事件 (权限检查、访问拒绝)
- 数据访问 (读取、修改、删除敏感数据)
- 配置变更 (系统配置、安全策略)
- 安全事件 (攻击检测、异常行为)
```

**审计日志格式**：

```json
{
  "timestamp": "2025-10-17T10:30:00Z",
  "event_id": "evt_abc123",
  "event_type": "data_access",
  "severity": "info",
  "user": {
    "id": "usr_123",
    "username": "alice",
    "ip": "192.168.1.100"
  },
  "resource": {
    "type": "user_profile",
    "id": "prof_456",
    "action": "read"
  },
  "outcome": "success",
  "metadata": {
    "request_id": "req_xyz789",
    "session_id": "sess_abc"
  }
}
```

**审计日志保护**：

- 仅追加 (Append-only)
- 不可篡改 (使用区块链或签名)
- 访问控制严格
- 定期备份

### 合规框架

**常见标准**：

```text
1. GDPR (通用数据保护条例)
   - 数据主体权利
   - 数据处理合法性
   - 数据泄露通知
   
2. PCI DSS (支付卡行业数据安全标准)
   - 保护持卡人数据
   - 加密传输
   - 访问控制
   
3. HIPAA (健康保险可移植性和责任法案)
   - 保护健康信息
   - 隐私规则
   - 安全规则
   
4. SOC 2 (Service Organization Control 2)
   - 安全性
   - 可用性
   - 处理完整性
   - 机密性
   - 隐私
```

---

## Rust实现示例

### 认证系统

```rust
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

/// JWT声明
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // 用户ID
    pub username: String,
    pub roles: Vec<String>,
    pub exp: i64,          // 过期时间
    pub iat: i64,          // 签发时间
    pub jti: String,       // JWT ID
}

/// 认证服务
pub struct AuthService {
    jwt_secret: Vec<u8>,
    token_expiry: Duration,
}

impl AuthService {
    pub fn new(jwt_secret: Vec<u8>, token_expiry_hours: i64) -> Self {
        Self {
            jwt_secret,
            token_expiry: Duration::hours(token_expiry_hours),
        }
    }
    
    /// 哈希密码
    pub fn hash_password(&self, password: &str) -> Result<String, anyhow::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
        
        Ok(password_hash)
    }
    
    /// 验证密码
    pub fn verify_password(
        &self,
        password: &str,
        password_hash: &str,
    ) -> Result<bool, anyhow::Error> {
        let parsed_hash = PasswordHash::new(password_hash)?;
        let argon2 = Argon2::default();
        
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
    
    /// 生成JWT
    pub fn generate_token(
        &self,
        user_id: String,
        username: String,
        roles: Vec<String>,
    ) -> Result<String, anyhow::Error> {
        let now = Utc::now();
        let exp = now + self.token_expiry;
        
        let claims = Claims {
            sub: user_id,
            username,
            roles,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
        };
        
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&self.jwt_secret),
        )?;
        
        Ok(token)
    }
    
    /// 验证JWT
    pub fn verify_token(&self, token: &str) -> Result<Claims, anyhow::Error> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(&self.jwt_secret),
            &Validation::default(),
        )?;
        
        Ok(token_data.claims)
    }
}

/// 多因素认证
pub struct MFAService {
    issuer: String,
}

impl MFAService {
    pub fn new(issuer: String) -> Self {
        Self { issuer }
    }
    
    /// 生成TOTP密钥
    pub fn generate_totp_secret(&self) -> String {
        use totp_rs::{Secret, TOTP};
        
        let secret = Secret::generate_secret();
        secret.to_encoded().to_string()
    }
    
    /// 验证TOTP码
    pub fn verify_totp(
        &self,
        secret: &str,
        code: &str,
        username: &str,
    ) -> Result<bool, anyhow::Error> {
        use totp_rs::{Secret, TOTP, Algorithm};
        
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(secret.to_string()).to_bytes()?,
            Some(self.issuer.clone()),
            username.to_string(),
        )?;
        
        Ok(totp.check_current(code)?)
    }
}
```

### 授权系统

```rust
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// 权限
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub action: String,
}

impl Permission {
    pub fn new(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
        }
    }
    
    /// 通配符匹配
    pub fn matches(&self, other: &Permission) -> bool {
        let resource_match = self.resource == "*" || self.resource == other.resource;
        let action_match = self.action == "*" || self.action == other.action;
        resource_match && action_match
    }
}

/// 角色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: HashSet<Permission>,
}

/// RBAC授权服务
pub struct RBACService {
    roles: HashMap<String, Role>,
    user_roles: HashMap<String, HashSet<String>>,
}

impl RBACService {
    pub fn new() -> Self {
        Self {
            roles: HashMap::new(),
            user_roles: HashMap::new(),
        }
    }
    
    /// 添加角色
    pub fn add_role(&mut self, role: Role) {
        self.roles.insert(role.name.clone(), role);
    }
    
    /// 为用户分配角色
    pub fn assign_role(&mut self, user_id: String, role_name: String) {
        self.user_roles
            .entry(user_id)
            .or_insert_with(HashSet::new)
            .insert(role_name);
    }
    
    /// 检查权限
    pub fn check_permission(
        &self,
        user_id: &str,
        required_permission: &Permission,
    ) -> bool {
        let user_roles = match self.user_roles.get(user_id) {
            Some(roles) => roles,
            None => return false,
        };
        
        for role_name in user_roles {
            if let Some(role) = self.roles.get(role_name) {
                for perm in &role.permissions {
                    if perm.matches(required_permission) {
                        return true;
                    }
                }
            }
        }
        
        false
    }
}

/// ABAC策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABACPolicy {
    pub name: String,
    pub effect: Effect,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub attribute: String,
    pub operator: Operator,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    In,
    Contains,
}

/// ABAC授权服务
pub struct ABACService {
    policies: Vec<ABACPolicy>,
}

impl ABACService {
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }
    
    pub fn add_policy(&mut self, policy: ABACPolicy) {
        self.policies.push(policy);
    }
    
    /// 评估策略
    pub fn evaluate(
        &self,
        attributes: &HashMap<String, String>,
    ) -> bool {
        let mut allow = false;
        
        for policy in &self.policies {
            if self.evaluate_conditions(&policy.conditions, attributes) {
                match policy.effect {
                    Effect::Allow => allow = true,
                    Effect::Deny => return false, // Deny优先
                }
            }
        }
        
        allow
    }
    
    fn evaluate_conditions(
        &self,
        conditions: &[Condition],
        attributes: &HashMap<String, String>,
    ) -> bool {
        conditions.iter().all(|cond| {
            let attr_value = attributes.get(&cond.attribute);
            match attr_value {
                Some(value) => self.evaluate_condition(cond, value),
                None => false,
            }
        })
    }
    
    fn evaluate_condition(&self, condition: &Condition, value: &str) -> bool {
        match condition.operator {
            Operator::Equals => value == condition.value,
            Operator::NotEquals => value != condition.value,
            Operator::Contains => value.contains(&condition.value),
            // 其他操作符...
            _ => false,
        }
    }
}
```

### 加密服务

```rust
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use ed25519_dalek::{Signer, Verifier, Signature, SigningKey, VerifyingKey};
use rand::RngCore;

/// 加密服务
pub struct CryptoService {
    master_key: Vec<u8>,
}

impl CryptoService {
    pub fn new(master_key: Vec<u8>) -> Self {
        assert_eq!(master_key.len(), 32, "Master key must be 32 bytes");
        Self { master_key }
    }
    
    /// AES-256-GCM加密
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        let cipher = Aes256Gcm::new_from_slice(&self.master_key)?;
        
        // 生成随机nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // 加密
        let ciphertext = cipher.encrypt(nonce, plaintext)?;
        
        // 格式: nonce + ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    /// AES-256-GCM解密
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        if encrypted.len() < 12 {
            anyhow::bail!("Invalid encrypted data");
        }
        
        let cipher = Aes256Gcm::new_from_slice(&self.master_key)?;
        
        // 分离nonce和ciphertext
        let (nonce_bytes, ciphertext) = encrypted.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        // 解密
        let plaintext = cipher.decrypt(nonce, ciphertext)?;
        
        Ok(plaintext)
    }
    
    /// 派生密钥 (HKDF)
    pub fn derive_key(&self, info: &[u8], length: usize) -> Result<Vec<u8>, anyhow::Error> {
        use hkdf::Hkdf;
        use sha2::Sha256;
        
        let hkdf = Hkdf::<Sha256>::new(None, &self.master_key);
        let mut output = vec![0u8; length];
        hkdf.expand(info, &mut output)?;
        
        Ok(output)
    }
}

/// 数字签名服务
pub struct SignatureService {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl SignatureService {
    pub fn new() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        
        Self {
            signing_key,
            verifying_key,
        }
    }
    
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        let verifying_key = signing_key.verifying_key();
        
        Self {
            signing_key,
            verifying_key,
        }
    }
    
    /// 签名
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }
    
    /// 验证签名
    pub fn verify(
        &self,
        message: &[u8],
        signature: &Signature,
    ) -> Result<(), anyhow::Error> {
        self.verifying_key
            .verify(message, signature)
            .map_err(|e| anyhow::anyhow!("Signature verification failed: {}", e))
    }
    
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }
}

/// 密钥派生函数
pub fn derive_key_from_password(
    password: &str,
    salt: &[u8],
) -> Result<Vec<u8>, anyhow::Error> {
    use argon2::Argon2;
    
    let argon2 = Argon2::default();
    let mut output = vec![0u8; 32];
    
    argon2.hash_password_into(password.as_bytes(), salt, &mut output)?;
    
    Ok(output)
}
```

### 审计日志1

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// 审计事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub event_id: String,
    pub event_type: EventType,
    pub severity: Severity,
    pub user: Option<UserContext>,
    pub resource: Option<ResourceContext>,
    pub outcome: Outcome,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Authentication,
    Authorization,
    DataAccess,
    DataModification,
    ConfigChange,
    SecurityIncident,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub id: String,
    pub username: String,
    pub ip: String,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContext {
    pub resource_type: String,
    pub resource_id: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Outcome {
    Success,
    Failure { reason: String },
}

/// 审计服务
pub struct AuditService {
    signature_service: SignatureService,
}

impl AuditService {
    pub fn new(signature_service: SignatureService) -> Self {
        Self { signature_service }
    }
    
    /// 记录审计事件
    pub fn log_event(&self, event: AuditEvent) -> Result<(), anyhow::Error> {
        // 序列化事件
        let event_json = serde_json::to_string(&event)?;
        
        // 签名事件（确保不可篡改）
        let signature = self.signature_service.sign(event_json.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());
        
        // 记录到日志系统
        match event.severity {
            Severity::Info => {
                info!(
                    event_type = ?event.event_type,
                    event_id = %event.event_id,
                    signature = %signature_hex,
                    "Audit event"
                );
            }
            Severity::Warning => {
                warn!(
                    event_type = ?event.event_type,
                    event_id = %event.event_id,
                    signature = %signature_hex,
                    "Audit event"
                );
            }
            _ => {
                tracing::error!(
                    event_type = ?event.event_type,
                    event_id = %event.event_id,
                    signature = %signature_hex,
                    "Audit event"
                );
            }
        }
        
        // 可以同时写入专门的审计日志存储
        self.persist_audit_event(&event, &signature_hex)?;
        
        Ok(())
    }
    
    /// 持久化审计事件
    fn persist_audit_event(
        &self,
        event: &AuditEvent,
        signature: &str,
    ) -> Result<(), anyhow::Error> {
        // 实现：写入数据库、文件或专门的审计日志系统
        // 这里只是示例
        Ok(())
    }
    
    /// 验证审计日志
    pub fn verify_audit_log(
        &self,
        event_json: &str,
        signature_hex: &str,
    ) -> Result<bool, anyhow::Error> {
        let signature_bytes = hex::decode(signature_hex)?;
        let signature = Signature::from_bytes(
            signature_bytes.as_slice().try_into()?
        );
        
        self.signature_service
            .verify(event_json.as_bytes(), &signature)
            .map(|_| true)
            .or(Ok(false))
    }
}

/// 审计事件构建器
pub struct AuditEventBuilder {
    event_type: EventType,
    severity: Severity,
    user: Option<UserContext>,
    resource: Option<ResourceContext>,
    outcome: Outcome,
    metadata: serde_json::Value,
}

impl AuditEventBuilder {
    pub fn new(event_type: EventType) -> Self {
        Self {
            event_type,
            severity: Severity::Info,
            user: None,
            resource: None,
            outcome: Outcome::Success,
            metadata: serde_json::json!({}),
        }
    }
    
    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }
    
    pub fn user(mut self, user: UserContext) -> Self {
        self.user = Some(user);
        self
    }
    
    pub fn resource(mut self, resource: ResourceContext) -> Self {
        self.resource = Some(resource);
        self
    }
    
    pub fn outcome(mut self, outcome: Outcome) -> Self {
        self.outcome = outcome;
        self
    }
    
    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
    
    pub fn build(self) -> AuditEvent {
        AuditEvent {
            timestamp: Utc::now(),
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: self.event_type,
            severity: self.severity,
            user: self.user,
            resource: self.resource,
            outcome: self.outcome,
            metadata: self.metadata,
        }
    }
}
```

### 安全中间件

```rust
use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// 认证中间件
pub async fn auth_middleware(
    State(auth_service): State<Arc<AuthService>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 提取Bearer token
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // 验证token
    let claims = auth_service
        .verify_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // 将claims附加到请求扩展中
    request.extensions_mut().insert(claims);
    
    Ok(next.run(request).await)
}

/// 授权中间件
pub async fn authz_middleware(
    State(rbac_service): State<Arc<RBACService>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 从请求扩展中获取claims
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // 获取请求的资源和操作
    let path = request.uri().path();
    let method = request.method().as_str();
    
    let permission = Permission::new(path, method);
    
    // 检查权限
    if !rbac_service.check_permission(&claims.sub, &permission) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(next.run(request).await)
}

/// 速率限制中间件
pub async fn rate_limit_middleware(
    State(rate_limiter): State<Arc<RateLimiter>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 提取客户端标识（IP或用户ID）
    let client_id = headers
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    
    // 检查速率限制
    if !rate_limiter.check(client_id).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    Ok(next.run(request).await)
}

/// 令牌桶速率限制器
use tokio::sync::Mutex;
use std::collections::HashMap;

pub struct RateLimiter {
    buckets: Mutex<HashMap<String, TokenBucket>>,
    capacity: u32,
    refill_rate: u32,
}

struct TokenBucket {
    tokens: f64,
    last_refill: DateTime<Utc>,
}

impl RateLimiter {
    pub fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
            capacity,
            refill_rate,
        }
    }
    
    pub async fn check(&self, client_id: &str) -> bool {
        let mut buckets = self.buckets.lock().await;
        let bucket = buckets.entry(client_id.to_string()).or_insert(TokenBucket {
            tokens: self.capacity as f64,
            last_refill: Utc::now(),
        });
        
        // 补充令牌
        let now = Utc::now();
        let elapsed = (now - bucket.last_refill).num_seconds() as f64;
        let new_tokens = elapsed * (self.refill_rate as f64 / 60.0);
        bucket.tokens = (bucket.tokens + new_tokens).min(self.capacity as f64);
        bucket.last_refill = now;
        
        // 消耗令牌
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}
```

---

## 测试策略

### 安全测试类型

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 认证测试
    #[tokio::test]
    async fn test_password_hashing() {
        let auth_service = AuthService::new(
            b"test_secret_key_32_bytes_long!!".to_vec(),
            24,
        );
        
        let password = "StrongPassword123!";
        let hash = auth_service.hash_password(password).unwrap();
        
        // 验证正确密码
        assert!(auth_service.verify_password(password, &hash).unwrap());
        
        // 验证错误密码
        assert!(!auth_service.verify_password("WrongPassword", &hash).unwrap());
    }
    
    #[tokio::test]
    async fn test_jwt_lifecycle() {
        let auth_service = AuthService::new(
            b"test_secret_key_32_bytes_long!!".to_vec(),
            24,
        );
        
        // 生成token
        let token = auth_service
            .generate_token(
                "user123".to_string(),
                "alice".to_string(),
                vec!["admin".to_string()],
            )
            .unwrap();
        
        // 验证token
        let claims = auth_service.verify_token(&token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.username, "alice");
        assert!(claims.roles.contains(&"admin".to_string()));
    }
    
    /// 授权测试
    #[tokio::test]
    async fn test_rbac_authorization() {
        let mut rbac = RBACService::new();
        
        // 创建角色
        let admin_role = Role {
            name: "admin".to_string(),
            permissions: vec![
                Permission::new("*", "*"),
            ].into_iter().collect(),
        };
        
        let reader_role = Role {
            name: "reader".to_string(),
            permissions: vec![
                Permission::new("*", "read"),
            ].into_iter().collect(),
        };
        
        rbac.add_role(admin_role);
        rbac.add_role(reader_role);
        
        // 分配角色
        rbac.assign_role("user1".to_string(), "admin".to_string());
        rbac.assign_role("user2".to_string(), "reader".to_string());
        
        // 测试权限
        assert!(rbac.check_permission(
            "user1",
            &Permission::new("users", "delete"),
        ));
        
        assert!(rbac.check_permission(
            "user2",
            &Permission::new("users", "read"),
        ));
        
        assert!(!rbac.check_permission(
            "user2",
            &Permission::new("users", "delete"),
        ));
    }
    
    /// 加密测试
    #[tokio::test]
    async fn test_encryption_decryption() {
        let key = vec![0u8; 32];
        let crypto = CryptoService::new(key);
        
        let plaintext = b"Sensitive data";
        
        // 加密
        let encrypted = crypto.encrypt(plaintext).unwrap();
        
        // 解密
        let decrypted = crypto.decrypt(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }
    
    #[tokio::test]
    async fn test_digital_signature() {
        let sig_service = SignatureService::new();
        
        let message = b"Important message";
        
        // 签名
        let signature = sig_service.sign(message);
        
        // 验证
        assert!(sig_service.verify(message, &signature).is_ok());
        
        // 验证篡改的消息
        let tampered = b"Tampered message";
        assert!(sig_service.verify(tampered, &signature).is_err());
    }
    
    /// 审计日志测试
    #[tokio::test]
    async fn test_audit_logging() {
        let sig_service = SignatureService::new();
        let audit_service = AuditService::new(sig_service);
        
        let event = AuditEventBuilder::new(EventType::DataAccess)
            .severity(Severity::Info)
            .user(UserContext {
                id: "user123".to_string(),
                username: "alice".to_string(),
                ip: "192.168.1.100".to_string(),
                user_agent: None,
            })
            .resource(ResourceContext {
                resource_type: "user_profile".to_string(),
                resource_id: "prof_456".to_string(),
                action: "read".to_string(),
            })
            .outcome(Outcome::Success)
            .build();
        
        // 记录事件
        assert!(audit_service.log_event(event).is_ok());
    }
}
```

### 渗透测试

**常见测试场景**：

```text
1. 注入攻击测试
   - SQL注入
   - 命令注入
   - LDAP注入
   
2. 认证测试
   - 弱密码测试
   - 暴力破解测试
   - Session劫持
   
3. 授权测试
   - 权限提升测试
   - 水平权限测试
   - 垂直权限测试
   
4. 加密测试
   - 弱加密算法
   - 密钥管理问题
   - 传输层安全
   
5. API安全测试
   - 参数篡改
   - 速率限制绕过
   - CORS配置错误
```

### 模糊测试

```rust
#[cfg(test)]
mod fuzz_tests {
    use super::*;
    
    /// 模糊测试加密/解密
    #[test]
    fn fuzz_encrypt_decrypt() {
        use quickcheck::{quickcheck, TestResult};
        
        fn prop(data: Vec<u8>) -> TestResult {
            if data.is_empty() {
                return TestResult::discard();
            }
            
            let key = vec![0u8; 32];
            let crypto = CryptoService::new(key);
            
            let encrypted = crypto.encrypt(&data).unwrap();
            let decrypted = crypto.decrypt(&encrypted).unwrap();
            
            TestResult::from_bool(data == decrypted)
        }
        
        quickcheck(prop as fn(Vec<u8>) -> TestResult);
    }
}
```

---

## 性能优化

### 加密性能优化

**硬件加速**：

```text
- AES-NI指令集
- Intel QuickAssist
- AWS Nitro加密加速
```

**算法选择**：

```text
对称加密：
- 小数据: AES-256-GCM
- 大数据: ChaCha20-Poly1305 (更好的软件性能)

非对称加密：
- RSA: 密钥交换
- ECDSA/Ed25519: 数字签名 (更快)

哈希：
- BLAKE3: 最快的密码学哈希
- SHA-256: 广泛支持
```

**批量操作**：

```rust
// 批量加密多个字段
pub async fn encrypt_batch(
    crypto: &CryptoService,
    fields: Vec<&[u8]>,
) -> Result<Vec<Vec<u8>>, anyhow::Error> {
    // 使用并行处理
    use rayon::prelude::*;
    
    fields
        .par_iter()
        .map(|field| crypto.encrypt(field))
        .collect()
}
```

### 会话管理优化

**会话缓存**：

```rust
use moka::sync::Cache;
use std::time::Duration;

pub struct SessionCache {
    cache: Cache<String, Claims>,
}

impl SessionCache {
    pub fn new(max_capacity: u64, ttl: Duration) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(ttl)
            .build();
        
        Self { cache }
    }
    
    pub fn get(&self, token: &str) -> Option<Claims> {
        self.cache.get(&token.to_string())
    }
    
    pub fn insert(&self, token: String, claims: Claims) {
        self.cache.insert(token, claims);
    }
}
```

### 密钥轮换策略

```rust
pub struct KeyRotationManager {
    current_key_id: String,
    keys: HashMap<String, Vec<u8>>,
    rotation_interval: Duration,
}

impl KeyRotationManager {
    /// 自动密钥轮换
    pub async fn start_rotation(&mut self) {
        let mut interval = tokio::time::interval(self.rotation_interval);
        
        loop {
            interval.tick().await;
            self.rotate_key().await;
        }
    }
    
    async fn rotate_key(&mut self) {
        // 生成新密钥
        let new_key_id = uuid::Uuid::new_v4().to_string();
        let new_key = Self::generate_key();
        
        // 保留旧密钥用于解密
        self.keys.insert(new_key_id.clone(), new_key);
        self.current_key_id = new_key_id;
        
        // 清理过期密钥（保留最近N个）
        if self.keys.len() > 5 {
            // 移除最旧的密钥
        }
    }
    
    fn generate_key() -> Vec<u8> {
        let mut key = vec![0u8; 32];
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut key);
        key
    }
}
```

---

## 最佳实践

### 安全编码原则

```text
1. 输入验证
   - 永远不要信任用户输入
   - 使用白名单而非黑名单
   - 验证数据类型、范围、格式
   
2. 输出编码
   - 对输出进行适当编码
   - 防止XSS、注入攻击
   
3. 错误处理
   - 不要泄露敏感信息
   - 使用通用错误消息
   - 详细错误记录到日志
   
4. 密码学
   - 使用标准库和算法
   - 不要自己实现加密
   - 定期更新依赖
   
5. 认证授权
   - 使用多因素认证
   - 实施最小权限原则
   - 定期审查权限
```

### 安全配置

```toml
[security]
# 密码策略
password_min_length = 12
password_require_uppercase = true
password_require_lowercase = true
password_require_digit = true
password_require_special = true

# 会话管理
session_timeout_minutes = 30
max_concurrent_sessions = 3
session_absolute_timeout_hours = 8

# MFA
mfa_required_for_admin = true
mfa_totp_window = 1

# 速率限制
rate_limit_requests_per_minute = 60
rate_limit_burst = 10

# 审计
audit_log_retention_days = 90
audit_log_signing = true

# TLS
tls_min_version = "1.3"
tls_cipher_suites = [
    "TLS_AES_256_GCM_SHA384",
    "TLS_CHACHA20_POLY1305_SHA256",
]

# 密钥轮换
key_rotation_days = 90
key_retention_count = 5
```

### 安全检查清单

```text
□ 认证
  □ 实施强密码策略
  □ 启用多因素认证
  □ 使用安全的会话管理
  □ 实施账户锁定机制

□ 授权
  □ 实施最小权限原则
  □ 定期审查权限
  □ 使用基于角色或策略的访问控制

□ 加密
  □ 传输层使用TLS 1.3
  □ 静态数据加密
  □ 使用强加密算法
  □ 安全的密钥管理

□ 审计
  □ 记录所有安全相关事件
  □ 保护审计日志不被篡改
  □ 定期审查审计日志

□ 网络安全
  □ 实施防火墙规则
  □ 使用网络分段
  □ DDoS防护
  □ API速率限制

□ 数据保护
  □ 数据分类
  □ 敏感数据脱敏
  □ 安全的数据删除
  □ 备份加密

□ 合规
  □ 满足GDPR/PCI DSS等要求
  □ 定期安全评估
  □ 漏洞管理
  □ 事件响应计划

□ 开发安全
  □ 代码审查
  □ 静态代码分析
  □ 依赖安全扫描
  □ 渗透测试
```

### 安全事件响应

```text
事件响应流程：

1. 检测 (Detection)
   - 监控告警
   - 异常检测
   - 用户报告
   
2. 分析 (Analysis)
   - 确定事件类型
   - 评估影响范围
   - 识别受影响资源
   
3. 遏制 (Containment)
   - 隔离受影响系统
   - 阻止攻击扩散
   - 保护证据
   
4. 根除 (Eradication)
   - 移除威胁
   - 修复漏洞
   - 更新系统
   
5. 恢复 (Recovery)
   - 恢复服务
   - 验证安全性
   - 监控复发
   
6. 总结 (Lessons Learned)
   - 事后分析
   - 更新流程
   - 培训团队
```

---

## 相关文档

- [3.10.1 架构模式](architecture_patterns.md)
- [3.10.2 错误处理](error_handling.md)
- [3.10.3 配置管理](configuration.md)
- [3.8 可观测性](../observability/README.md)
- [3.6 事务处理](../transactions/README.md)

## 参考资料

### 标准与规范

- NIST Cybersecurity Framework
- OWASP Top 10
- CIS Controls
- ISO 27001/27002

### 工具与库

- **认证**: `argon2`, `jsonwebtoken`, `totp-rs`
- **加密**: `aes-gcm`, `ed25519-dalek`, `ring`
- **TLS**: `rustls`, `tokio-rustls`
- **审计**: `tracing`, `serde_json`

### 最佳实践1

- OWASP Secure Coding Practices
- NIST Special Publications (800系列)
- Cloud Security Alliance Guidelines
