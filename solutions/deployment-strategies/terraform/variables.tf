# Terraform 变量定义

variable "aws_region" {
  description = "AWS 区域"
  type        = string
  default     = "us-west-2"
}

variable "environment" {
  description = "环境名称"
  type        = string
  default     = "dev"
  
  validation {
    condition     = contains(["dev", "staging", "prod"], var.environment)
    error_message = "环境必须是 dev、staging 或 prod 之一。"
  }
}

variable "project_name" {
  description = "项目名称"
  type        = string
  default     = "distributed-compute"
}

variable "vpc_cidr" {
  description = "VPC CIDR 块"
  type        = string
  default     = "10.0.0.0/16"
}

variable "availability_zones" {
  description = "可用区列表"
  type        = list(string)
  default     = ["us-west-2a", "us-west-2b", "us-west-2c"]
}

variable "private_subnet_cidrs" {
  description = "私有子网 CIDR 块"
  type        = list(string)
  default     = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
}

variable "public_subnet_cidrs" {
  description = "公有子网 CIDR 块"
  type        = list(string)
  default     = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]
}

variable "public_access_cidrs" {
  description = "EKS API 服务器公有访问 CIDR 块"
  type        = list(string)
  default     = ["0.0.0.0/0"]
}

variable "kubernetes_version" {
  description = "Kubernetes 版本"
  type        = string
  default     = "1.28"
}

variable "node_instance_types" {
  description = "EKS 节点实例类型"
  type        = list(string)
  default     = ["t3.medium", "t3.large"]
}

variable "node_disk_size" {
  description = "EKS 节点磁盘大小（GB）"
  type        = number
  default     = 50
}

variable "node_desired_size" {
  description = "EKS 节点组期望大小"
  type        = number
  default     = 3
}

variable "node_max_size" {
  description = "EKS 节点组最大大小"
  type        = number
  default     = 10
}

variable "node_min_size" {
  description = "EKS 节点组最小大小"
  type        = number
  default     = 1
}

variable "enable_rds_clickhouse" {
  description = "是否启用 RDS ClickHouse"
  type        = bool
  default     = false
}

variable "clickhouse_instance_class" {
  description = "ClickHouse 实例类型"
  type        = string
  default     = "db.r6g.large"
}

variable "clickhouse_allocated_storage" {
  description = "ClickHouse 分配的存储大小（GB）"
  type        = number
  default     = 100
}

variable "clickhouse_password" {
  description = "ClickHouse 密码"
  type        = string
  sensitive   = true
  default     = "changeme123!"
}

variable "enable_redis" {
  description = "是否启用 Redis"
  type        = bool
  default     = true
}

variable "redis_node_type" {
  description = "Redis 节点类型"
  type        = string
  default     = "cache.t3.micro"
}

variable "redis_num_cache_nodes" {
  description = "Redis 缓存节点数量"
  type        = number
  default     = 2
}

variable "tags" {
  description = "资源标签"
  type        = map(string)
  default     = {}
}
