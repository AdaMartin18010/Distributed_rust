# 3.10.7 部署和运维 (Deployment and Operations)

## 目录

- [3.10.7 部署和运维 (Deployment and Operations)](#3107-部署和运维-deployment-and-operations)
  - [目录](#目录)
  - [核心概念](#核心概念)
    - [部署模型](#部署模型)
    - [运维成熟度](#运维成熟度)
  - [部署策略](#部署策略)
    - [蓝绿部署](#蓝绿部署)
    - [金丝雀部署](#金丝雀部署)
    - [滚动更新](#滚动更新)
    - [A/B测试部署](#ab测试部署)
  - [容器化](#容器化)
    - [Docker镜像](#docker镜像)
    - [容器编排](#容器编排)
  - [编排和调度](#编排和调度)
    - [Kubernetes部署](#kubernetes部署)
    - [StatefulSet（有状态应用）](#statefulset有状态应用)
  - [基础设施即代码](#基础设施即代码)
    - [Terraform](#terraform)
    - [Pulumi（Rust IaC）](#pulumirust-iac)
  - [配置管理](#配置管理)
    - [环境配置](#环境配置)
    - [Kubernetes ConfigMap和Secret](#kubernetes-configmap和secret)
    - [密钥管理](#密钥管理)
  - [灾难恢复](#灾难恢复)
    - [备份策略](#备份策略)
    - [RTO和RPO](#rto和rpo)
    - [数据库备份](#数据库备份)
    - [灾难恢复演练](#灾难恢复演练)
  - [运维实践](#运维实践)
    - [CI/CD流水线](#cicd流水线)
    - [GitOps（Flux/ArgoCD）](#gitopsfluxargocd)
    - [零停机部署检查清单](#零停机部署检查清单)
  - [实现示例](#实现示例)
    - [Rust健康检查端点](#rust健康检查端点)
    - [优雅关闭](#优雅关闭)
  - [最佳实践](#最佳实践)
    - [部署检查清单](#部署检查清单)
    - [SRE原则](#sre原则)
    - [运维成熟度评估](#运维成熟度评估)
  - [相关文档](#相关文档)
  - [参考资料](#参考资料)
    - [工具](#工具)
    - [最佳实践1](#最佳实践1)

---

## 核心概念

部署和运维是保证分布式系统稳定运行的关键环节，包括持续交付、自动化运维、故障恢复等。

### 部署模型

```text
传统部署 vs 现代部署：

传统部署：
- 物理服务器
- 手动配置
- 长周期发布
- 停机部署

现代部署：
- 容器化
- 自动化流程
- 持续交付
- 零停机部署
```

### 运维成熟度

```text
Level 0: 手动运维
  - 手动部署
  - 手动配置
  - 响应式故障处理

Level 1: 脚本化
  - 部署脚本
  - 配置脚本
  - 基础监控

Level 2: 自动化
  - CI/CD流水线
  - 自动化测试
  - 自动化监控和告警

Level 3: 自服务
  - 平台即服务
  - 自助部署
  - 自动扩缩容

Level 4: 自愈
  - 自动故障检测
  - 自动故障修复
  - 预测性维护
```

---

## 部署策略

### 蓝绿部署

```text
Blue-Green Deployment：

┌─────────────┐      ┌─────────────┐
│   Blue      │      │   Green     │
│ (当前版本)  │      │  (新版本)   │
│   v1.0      │      │   v2.0      │
└──────┬──────┘      └──────┬──────┘
       │                    │
       └────────┬───────────┘
                │
          ┌─────▼─────┐
          │  Router   │
          └───────────┘

流程：
1. 部署新版本到Green环境
2. 测试Green环境
3. 切换流量到Green
4. 保留Blue作为回滚备份
```

**实现示例**：

```yaml
# Kubernetes Blue-Green部署
apiVersion: v1
kind: Service
metadata:
  name: myapp-service
spec:
  selector:
    app: myapp
    version: blue  # 切换为 'green' 进行部署
  ports:
    - port: 80
      targetPort: 8080

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp-blue
spec:
  replicas: 3
  selector:
    matchLabels:
      app: myapp
      version: blue
  template:
    metadata:
      labels:
        app: myapp
        version: blue
    spec:
      containers:
      - name: myapp
        image: myapp:v1.0

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp-green
spec:
  replicas: 3
  selector:
    matchLabels:
      app: myapp
      version: green
  template:
    metadata:
      labels:
        app: myapp
        version: green
    spec:
      containers:
      - name: myapp
        image: myapp:v2.0
```

### 金丝雀部署

```text
Canary Deployment：

逐步将流量切换到新版本

Stage 1: 5% traffic
┌─────┐  95%  ┌─────┐
│ v1  │◄──────┤     │
└─────┘       │Load │
┌─────┐  5%   │Balancer│
│ v2  │◄──────┤     │
└─────┘       └─────┘

Stage 2: 50% traffic
┌─────┐  50%  ┌─────┐
│ v1  │◄──────┤     │
└─────┘       │Load │
┌─────┐  50%  │Balancer│
│ v2  │◄──────┤     │
└─────┘       └─────┘

Stage 3: 100% traffic
┌─────┐       ┌─────┐
│ v1  │       │     │
└─────┘       │Load │
┌─────┐ 100%  │Balancer│
│ v2  │◄──────┤     │
└─────┘       └─────┘
```

**Istio金丝雀部署**：

```yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: myapp
spec:
  hosts:
  - myapp.example.com
  http:
  - match:
    - headers:
        canary:
          exact: "true"
    route:
    - destination:
        host: myapp
        subset: v2
  - route:
    - destination:
        host: myapp
        subset: v1
      weight: 90
    - destination:
        host: myapp
        subset: v2
      weight: 10

---
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: myapp
spec:
  host: myapp
  subsets:
  - name: v1
    labels:
      version: v1
  - name: v2
    labels:
      version: v2
```

### 滚动更新

```text
Rolling Update：

逐个替换实例

┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐
│ v1  │ │ v1  │ │ v1  │ │ v1  │  初始状态
└─────┘ └─────┘ └─────┘ └─────┘

┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐
│ v2  │ │ v1  │ │ v1  │ │ v1  │  Step 1
└─────┘ └─────┘ └─────┘ └─────┘

┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐
│ v2  │ │ v2  │ │ v1  │ │ v1  │  Step 2
└─────┘ └─────┘ └─────┘ └─────┘

┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐
│ v2  │ │ v2  │ │ v2  │ │ v1  │  Step 3
└─────┘ └─────┘ └─────┘ └─────┘

┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐
│ v2  │ │ v2  │ │ v2  │ │ v2  │  完成
└─────┘ └─────┘ └─────┘ └─────┘
```

**Kubernetes滚动更新**：

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
spec:
  replicas: 4
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1        # 最多超出期望副本数
      maxUnavailable: 1  # 最多不可用副本数
  selector:
    matchLabels:
      app: myapp
  template:
    metadata:
      labels:
        app: myapp
    spec:
      containers:
      - name: myapp
        image: myapp:v2.0
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

### A/B测试部署

```text
A/B Testing：

基于用户属性路由到不同版本

User Group A  ──→  Version A
User Group B  ──→  Version B

用于：
- 功能对比测试
- UI/UX测试
- 性能对比
```

---

## 容器化

### Docker镜像

**多阶段构建**：

```dockerfile
# Rust应用多阶段构建
# 阶段1：构建
FROM rust:1.90 as builder
WORKDIR /app

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 预构建依赖（缓存优化）
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制源代码
COPY src ./src

# 构建应用
RUN cargo build --release

# 阶段2：运行
FROM debian:bookworm-slim
WORKDIR /app

# 安装运行时依赖
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/myapp /app/myapp

# 创建非root用户
RUN useradd -m -u 1000 appuser && \
    chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

CMD ["/app/myapp"]
```

**镜像优化**：

```dockerfile
# 使用distroless基础镜像（最小化）
FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/myapp /myapp

EXPOSE 8080

CMD ["/myapp"]
```

### 容器编排

**Docker Compose示例**：

```yaml
version: '3.8'

services:
  app:
    image: myapp:latest
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://postgres:password@db:5432/mydb
      - REDIS_URL=redis://redis:6379
    depends_on:
      - db
      - redis
    networks:
      - app-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  db:
    image: postgres:16
    environment:
      - POSTGRES_DB=mydb
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres-data:/var/lib/postgresql/data
    networks:
      - app-network
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    volumes:
      - redis-data:/data
    networks:
      - app-network
    restart: unless-stopped

  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    ports:
      - "9090:9090"
    networks:
      - app-network

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-data:/var/lib/grafana
    networks:
      - app-network
    depends_on:
      - prometheus

volumes:
  postgres-data:
  redis-data:
  prometheus-data:
  grafana-data:

networks:
  app-network:
    driver: bridge
```

---

## 编排和调度

### Kubernetes部署

**完整应用部署**：

```yaml
# Namespace
apiVersion: v1
kind: Namespace
metadata:
  name: myapp

---
# ConfigMap
apiVersion: v1
kind: ConfigMap
metadata:
  name: myapp-config
  namespace: myapp
data:
  app.yaml: |
    server:
      port: 8080
    logging:
      level: info

---
# Secret
apiVersion: v1
kind: Secret
metadata:
  name: myapp-secrets
  namespace: myapp
type: Opaque
stringData:
  database-url: "postgresql://user:pass@db:5432/mydb"
  api-key: "secret-api-key"

---
# Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
  namespace: myapp
spec:
  replicas: 3
  selector:
    matchLabels:
      app: myapp
  template:
    metadata:
      labels:
        app: myapp
        version: v1.0
    spec:
      containers:
      - name: myapp
        image: myapp:v1.0
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: myapp-secrets
              key: database-url
        - name: API_KEY
          valueFrom:
            secretKeyRef:
              name: myapp-secrets
              key: api-key
        volumeMounts:
        - name: config
          mountPath: /etc/config
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 500m
            memory: 512Mi
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: myapp-config

---
# Service
apiVersion: v1
kind: Service
metadata:
  name: myapp-service
  namespace: myapp
spec:
  selector:
    app: myapp
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: ClusterIP

---
# Ingress
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: myapp-ingress
  namespace: myapp
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/rate-limit: "100"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - myapp.example.com
    secretName: myapp-tls
  rules:
  - host: myapp.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: myapp-service
            port:
              number: 80

---
# HorizontalPodAutoscaler
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: myapp-hpa
  namespace: myapp
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: myapp
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
      - type: Percent
        value: 100
        periodSeconds: 15
      - type: Pods
        value: 2
        periodSeconds: 15

---
# PodDisruptionBudget
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: myapp-pdb
  namespace: myapp
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: myapp
```

### StatefulSet（有状态应用）

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: myapp-stateful
spec:
  serviceName: myapp-headless
  replicas: 3
  selector:
    matchLabels:
      app: myapp-stateful
  template:
    metadata:
      labels:
        app: myapp-stateful
    spec:
      containers:
      - name: myapp
        image: myapp:v1.0
        ports:
        - containerPort: 8080
        volumeMounts:
        - name: data
          mountPath: /data
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: fast-ssd
      resources:
        requests:
          storage: 10Gi

---
apiVersion: v1
kind: Service
metadata:
  name: myapp-headless
spec:
  clusterIP: None
  selector:
    app: myapp-stateful
  ports:
  - port: 8080
```

---

## 基础设施即代码

### Terraform

**AWS基础设施示例**：

```hcl
# providers.tf
terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
  
  backend "s3" {
    bucket = "myapp-terraform-state"
    key    = "prod/terraform.tfstate"
    region = "us-east-1"
  }
}

provider "aws" {
  region = var.aws_region
}

# variables.tf
variable "aws_region" {
  description = "AWS region"
  default     = "us-east-1"
}

variable "environment" {
  description = "Environment name"
  default     = "production"
}

variable "vpc_cidr" {
  description = "VPC CIDR block"
  default     = "10.0.0.0/16"
}

# vpc.tf
resource "aws_vpc" "main" {
  cidr_block           = var.vpc_cidr
  enable_dns_hostnames = true
  enable_dns_support   = true
  
  tags = {
    Name        = "${var.environment}-vpc"
    Environment = var.environment
  }
}

resource "aws_subnet" "public" {
  count             = 3
  vpc_id            = aws_vpc.main.id
  cidr_block        = cidrsubnet(var.vpc_cidr, 4, count.index)
  availability_zone = data.aws_availability_zones.available.names[count.index]
  
  map_public_ip_on_launch = true
  
  tags = {
    Name        = "${var.environment}-public-${count.index + 1}"
    Environment = var.environment
  }
}

resource "aws_subnet" "private" {
  count             = 3
  vpc_id            = aws_vpc.main.id
  cidr_block        = cidrsubnet(var.vpc_cidr, 4, count.index + 3)
  availability_zone = data.aws_availability_zones.available.names[count.index]
  
  tags = {
    Name        = "${var.environment}-private-${count.index + 1}"
    Environment = var.environment
  }
}

# eks.tf
resource "aws_eks_cluster" "main" {
  name     = "${var.environment}-cluster"
  role_arn = aws_iam_role.eks_cluster.arn
  version  = "1.28"
  
  vpc_config {
    subnet_ids = concat(
      aws_subnet.public[*].id,
      aws_subnet.private[*].id
    )
    endpoint_private_access = true
    endpoint_public_access  = true
  }
  
  depends_on = [
    aws_iam_role_policy_attachment.eks_cluster_policy
  ]
}

resource "aws_eks_node_group" "main" {
  cluster_name    = aws_eks_cluster.main.name
  node_group_name = "${var.environment}-node-group"
  node_role_arn   = aws_iam_role.eks_node.arn
  subnet_ids      = aws_subnet.private[*].id
  
  scaling_config {
    desired_size = 3
    max_size     = 10
    min_size     = 2
  }
  
  instance_types = ["t3.medium"]
  
  depends_on = [
    aws_iam_role_policy_attachment.eks_node_policy
  ]
}

# rds.tf
resource "aws_db_instance" "main" {
  identifier           = "${var.environment}-db"
  engine               = "postgres"
  engine_version       = "16"
  instance_class       = "db.t3.medium"
  allocated_storage    = 100
  storage_encrypted    = true
  
  db_name  = "myapp"
  username = "admin"
  password = random_password.db_password.result
  
  vpc_security_group_ids = [aws_security_group.rds.id]
  db_subnet_group_name   = aws_db_subnet_group.main.name
  
  backup_retention_period = 7
  backup_window          = "03:00-04:00"
  maintenance_window     = "mon:04:00-mon:05:00"
  
  skip_final_snapshot = false
  final_snapshot_identifier = "${var.environment}-db-final-snapshot"
  
  tags = {
    Name        = "${var.environment}-database"
    Environment = var.environment
  }
}

# outputs.tf
output "eks_cluster_endpoint" {
  value = aws_eks_cluster.main.endpoint
}

output "rds_endpoint" {
  value     = aws_db_instance.main.endpoint
  sensitive = true
}
```

### Pulumi（Rust IaC）

```rust
use pulumi_wasm_rust::Output;
use pulumi_aws::eks;
use pulumi_kubernetes as k8s;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pulumi_wasm_rust::init("myapp");
    
    // 创建EKS集群
    let cluster = eks::Cluster::create(
        "my-cluster",
        eks::ClusterArgs::builder()
            .version("1.28")
            .build(),
    );
    
    // 创建Kubernetes provider
    let k8s_provider = k8s::Provider::create(
        "k8s-provider",
        k8s::ProviderArgs::builder()
            .kubeconfig(cluster.kubeconfig)
            .build(),
    );
    
    // 部署应用
    let deployment = k8s::apps::v1::Deployment::create(
        "myapp",
        k8s::apps::v1::DeploymentArgs::builder()
            .spec(k8s::apps::v1::DeploymentSpecArgs::builder()
                .replicas(3)
                .selector(/* ... */)
                .template(/* ... */)
                .build())
            .build(),
        Some(k8s_provider),
    );
    
    Ok(())
}
```

---

## 配置管理

### 环境配置

**12-Factor App配置原则**：

```text
1. 配置与代码分离
2. 使用环境变量
3. 不同环境使用不同配置
4. 不在代码中硬编码敏感信息
```

**配置层级**：

```text
优先级从高到低：
1. 环境变量
2. 命令行参数
3. 配置文件
4. 默认值

示例：
DATABASE_URL (env)
  ↓ 覆盖
config.yaml (file)
  ↓ 覆盖
默认配置 (code)
```

### Kubernetes ConfigMap和Secret

```yaml
# 从文件创建ConfigMap
kubectl create configmap myapp-config \
  --from-file=config.yaml

# 从字面值创建Secret
kubectl create secret generic myapp-secret \
  --from-literal=db-password=supersecret

# 在Pod中使用
apiVersion: v1
kind: Pod
metadata:
  name: myapp
spec:
  containers:
  - name: myapp
    image: myapp:latest
    envFrom:
    - configMapRef:
        name: myapp-config
    env:
    - name: DB_PASSWORD
      valueFrom:
        secretKeyRef:
          name: myapp-secret
          key: db-password
```

### 密钥管理

**Vault集成**：

```rust
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use vaultrs::kv2;

pub async fn get_secret_from_vault(
    vault_addr: &str,
    vault_token: &str,
    secret_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address(vault_addr)
            .token(vault_token)
            .build()?
    )?;
    
    let secret: std::collections::HashMap<String, String> = 
        kv2::read(&client, "secret", secret_path).await?;
    
    Ok(secret.get("password")
        .ok_or("Password not found")?
        .clone())
}
```

---

## 灾难恢复

### 备份策略

**3-2-1备份规则**：

```text
3: 至少3份数据副本
2: 2种不同的存储介质
1: 1份异地备份

示例：
- 生产数据库 (主)
- 本地备份 (SSD)
- 远程备份 (S3)
```

**备份类型**：

```text
1. 全量备份
   - 完整数据副本
   - 恢复最快
   - 存储开销大

2. 增量备份
   - 只备份变化数据
   - 存储开销小
   - 恢复较慢

3. 差异备份
   - 备份自上次全量备份后的变化
   - 折中方案
```

### RTO和RPO

```text
RTO (Recovery Time Objective)：
  恢复时间目标 - 系统可容忍的最大停机时间

RPO (Recovery Point Objective)：
  恢复点目标 - 可容忍的最大数据丢失量

示例：
  RTO = 4小时 → 系统必须在4小时内恢复
  RPO = 1小时 → 最多丢失1小时的数据
```

### 数据库备份

**PostgreSQL备份脚本**：

```bash
#!/bin/bash

# 配置
DB_NAME="mydb"
DB_USER="postgres"
BACKUP_DIR="/backups"
RETENTION_DAYS=7
S3_BUCKET="s3://myapp-backups"

# 创建备份目录
mkdir -p $BACKUP_DIR

# 生成备份文件名
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/${DB_NAME}_${TIMESTAMP}.sql.gz"

# 执行备份
pg_dump -U $DB_USER -Fc $DB_NAME | gzip > $BACKUP_FILE

# 上传到S3
aws s3 cp $BACKUP_FILE $S3_BUCKET/

# 清理旧备份
find $BACKUP_DIR -name "*.sql.gz" -mtime +$RETENTION_DAYS -delete

# 验证备份
if [ -f "$BACKUP_FILE" ]; then
    echo "Backup successful: $BACKUP_FILE"
else
    echo "Backup failed!"
    exit 1
fi
```

### 灾难恢复演练

```text
DR演练流程：

1. 计划阶段
   - 定义场景
   - 准备检查清单
   - 通知相关人员

2. 执行阶段
   - 模拟故障
   - 执行恢复流程
   - 记录时间和问题

3. 验证阶段
   - 验证数据完整性
   - 验证系统功能
   - 测试性能

4. 总结阶段
   - 记录问题和改进点
   - 更新DR文档
   - 制定改进计划

频率：
- 全面演练：每年2次
- 部分演练：每季度1次
- 备份恢复测试：每月1次
```

---

## 运维实践

### CI/CD流水线

**GitHub Actions示例**：

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    
    - name: Cache cargo registry
      uses: actions/cache@v3
      with:
        path: ~/.cargo/registry
        key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Run tests
      run: cargo test --all-features
    
    - name: Run clippy
      run: cargo clippy -- -D warnings
    
    - name: Check formatting
      run: cargo fmt -- --check

  build:
    needs: test
    runs-on: ubuntu-latest
    if: github.event_name == 'push'
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Docker Buildx
      uses: docker/setup-buildx-action@v3
    
    - name: Log in to GitHub Container Registry
      uses: docker/login-action@v3
      with:
        registry: ${{ env.REGISTRY }}
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}
    
    - name: Extract metadata
      id: meta
      uses: docker/metadata-action@v5
      with:
        images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
        tags: |
          type=ref,event=branch
          type=sha,prefix={{branch}}-
          type=semver,pattern={{version}}
    
    - name: Build and push
      uses: docker/build-push-action@v5
      with:
        context: .
        push: true
        tags: ${{ steps.meta.outputs.tags }}
        labels: ${{ steps.meta.outputs.labels }}
        cache-from: type=gha
        cache-to: type=gha,mode=max

  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
    - uses: actions/checkout@v4
    
    - name: Configure AWS credentials
      uses: aws-actions/configure-aws-credentials@v4
      with:
        aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
        aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
        aws-region: us-east-1
    
    - name: Update kubeconfig
      run: |
        aws eks update-kubeconfig --name production-cluster
    
    - name: Deploy to Kubernetes
      run: |
        kubectl set image deployment/myapp \
          myapp=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:main-${{ github.sha }}
        kubectl rollout status deployment/myapp
    
    - name: Run smoke tests
      run: |
        kubectl run smoke-test --rm -i --restart=Never \
          --image=curlimages/curl:latest \
          -- curl -f http://myapp-service/health
```

### GitOps（Flux/ArgoCD）

**ArgoCD应用定义**：

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: myapp
  namespace: argocd
spec:
  project: default
  
  source:
    repoURL: https://github.com/myorg/myapp-config
    targetRevision: HEAD
    path: k8s/overlays/production
    kustomize:
      version: v5.0.0
  
  destination:
    server: https://kubernetes.default.svc
    namespace: myapp
  
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
      allowEmpty: false
    syncOptions:
    - CreateNamespace=true
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
```

### 零停机部署检查清单

```text
□ 健康检查
  □ Liveness probe配置正确
  □ Readiness probe配置正确
  □ 启动探针（startup probe）

□ 滚动更新配置
  □ maxSurge和maxUnavailable设置合理
  □ 滚动更新速度适中

□ 数据库迁移
  □ 向后兼容的schema变更
  □ 先部署兼容旧schema的代码
  □ 再执行schema迁移

□ 回滚准备
  □ 保留旧版本镜像
  □ 快速回滚流程
  □ 回滚测试

□ 监控和告警
  □ 部署前确认监控正常
  □ 部署后密切观察指标
  □ 设置部署告警

□ 流量管理
  □ 金丝雀部署或蓝绿部署
  □ 逐步增加流量
  □ 流量回退机制
```

---

## 实现示例

### Rust健康检查端点

```rust
use axum::{
    extract::State,
    http::StatusCode,
    Json, Router, routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

#[derive(Clone)]
pub struct AppState {
    pub start_time: std::time::Instant,
    pub version: String,
}

pub fn health_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health_handler))
        .route("/health/live", get(liveness_handler))
        .route("/health/ready", get(readiness_handler))
}

async fn health_handler(
    State(state): State<Arc<AppState>>,
) -> Json<HealthCheck> {
    Json(HealthCheck {
        status: "healthy".to_string(),
        version: state.version.clone(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
    })
}

async fn liveness_handler() -> StatusCode {
    // 检查应用是否存活
    // 简单返回200即可
    StatusCode::OK
}

async fn readiness_handler(
    State(state): State<Arc<AppState>>,
) -> StatusCode {
    // 检查应用是否准备好接收流量
    // 检查依赖服务（数据库、缓存等）
    
    // 示例：检查数据库连接
    // if !check_database_connection().await {
    //     return StatusCode::SERVICE_UNAVAILABLE;
    // }
    
    StatusCode::OK
}
```

### 优雅关闭

```rust
use tokio::signal;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct GracefulShutdown {
    is_shutting_down: Arc<AtomicBool>,
}

impl GracefulShutdown {
    pub fn new() -> Self {
        Self {
            is_shutting_down: Arc::new(AtomicBool::new(false)),
        }
    }
    
    pub fn is_shutting_down(&self) -> bool {
        self.is_shutting_down.load(Ordering::Relaxed)
    }
    
    pub async fn wait_for_signal(&self) {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                tracing::info!("Received SIGINT");
            }
            _ = terminate => {
                tracing::info!("Received SIGTERM");
            }
        }
        
        self.is_shutting_down.store(true, Ordering::Relaxed);
    }
}

// 主函数中使用
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shutdown = Arc::new(GracefulShutdown::new());
    let shutdown_clone = shutdown.clone();
    
    // 启动HTTP服务器
    let server = axum::Server::bind(&"0.0.0.0:8080".parse()?)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async move {
            shutdown_clone.wait_for_signal().await;
            tracing::info!("Starting graceful shutdown...");
        });
    
    server.await?;
    
    tracing::info!("Server shut down gracefully");
    Ok(())
}
```

---

## 最佳实践

### 部署检查清单

```text
□ 代码质量
  □ 所有测试通过
  □ 代码审查完成
  □ 静态分析无问题

□ 配置
  □ 环境变量正确配置
  □ 密钥安全管理
  □ 配置文件版本控制

□ 依赖
  □ 依赖版本锁定
  □ 安全漏洞扫描
  □ License合规检查

□ 基础设施
  □ 资源配额足够
  □ 网络配置正确
  □ 存储容量充足

□ 监控
  □ 监控指标配置
  □ 日志聚合工作
  □ 告警规则设置

□ 备份
  □ 数据备份完成
  □ 恢复流程测试
  □ 备份验证

□ 文档
  □ 部署文档更新
  □ Runbook更新
  □ 变更记录
```

### SRE原则

```text
1. 拥抱风险
   - 设定合理的SLO
   - 使用错误预算
   - 平衡可靠性和速度

2. 服务级别目标
   - 定义SLI（指标）
   - 设定SLO（目标）
   - 制定SLA（协议）

3. 消除琐事
   - 自动化重复任务
   - 减少手动操作
   - 提高运维效率

4. 监控和可观测性
   - 全面的监控覆盖
   - 有效的告警策略
   - 可操作的仪表盘

5. 自动化
   - CI/CD自动化
   - 基础设施自动化
   - 故障响应自动化

6. 发布工程
   - 小批量发布
   - 快速回滚
   - 金丝雀部署

7. 简单性
   - 简化架构
   - 减少复杂性
   - 清晰的文档
```

### 运维成熟度评估

```text
Level 1: 基础
□ 手动部署流程
□ 基本监控
□ 被动响应

Level 2: 标准化
□ 脚本化部署
□ 完善的监控
□ 文档化流程

Level 3: 自动化
□ CI/CD流水线
□ 自动化测试
□ 主动监控

Level 4: 自服务
□ 开发者自助部署
□ 自动扩缩容
□ 自动故障恢复

Level 5: 优化
□ AI辅助运维
□ 预测性维护
□ 持续优化
```

---

## 相关文档

- [3.10.3 配置管理](configuration.md)
- [3.10.6 监控和可观测性](monitoring.md)
- [3.10.4 安全设计](security.md)
- [部署策略示例](../../solutions/deployment-strategies/)

## 参考资料

### 工具

- **容器**: Docker, Podman
- **编排**: Kubernetes, Docker Swarm
- **IaC**: Terraform, Pulumi
- **CI/CD**: GitHub Actions, GitLab CI, Jenkins
- **GitOps**: ArgoCD, Flux

### 最佳实践1

- Google SRE Book
- Kubernetes Best Practices
- The DevOps Handbook
- Accelerate (DevOps Research)
