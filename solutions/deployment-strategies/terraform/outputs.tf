# Terraform 输出定义

output "vpc_id" {
  description = "VPC ID"
  value       = module.vpc.vpc_id
}

output "vpc_cidr_block" {
  description = "VPC CIDR 块"
  value       = module.vpc.vpc_cidr_block
}

output "private_subnets" {
  description = "私有子网 ID 列表"
  value       = module.vpc.private_subnets
}

output "public_subnets" {
  description = "公有子网 ID 列表"
  value       = module.vpc.public_subnets
}

output "eks_cluster_id" {
  description = "EKS 集群 ID"
  value       = aws_eks_cluster.main.id
}

output "eks_cluster_arn" {
  description = "EKS 集群 ARN"
  value       = aws_eks_cluster.main.arn
}

output "eks_cluster_endpoint" {
  description = "EKS 集群端点"
  value       = aws_eks_cluster.main.endpoint
}

output "eks_cluster_security_group_id" {
  description = "EKS 集群安全组 ID"
  value       = aws_eks_cluster.main.vpc_config[0].cluster_security_group_id
}

output "eks_node_group_arn" {
  description = "EKS 节点组 ARN"
  value       = aws_eks_node_group.main.arn
}

output "clickhouse_endpoint" {
  description = "ClickHouse 端点"
  value       = var.enable_rds_clickhouse ? aws_db_instance.clickhouse[0].endpoint : null
}

output "clickhouse_port" {
  description = "ClickHouse 端口"
  value       = var.enable_rds_clickhouse ? aws_db_instance.clickhouse[0].port : null
}

output "redis_endpoint" {
  description = "Redis 端点"
  value       = var.enable_redis ? aws_elasticache_replication_group.redis[0].primary_endpoint_address : null
}

output "redis_port" {
  description = "Redis 端口"
  value       = var.enable_redis ? aws_elasticache_replication_group.redis[0].port : null
}

output "kubectl_config_command" {
  description = "kubectl 配置命令"
  value       = "aws eks update-kubeconfig --region ${var.aws_region} --name ${aws_eks_cluster.main.name}"
}

output "helm_install_commands" {
  description = "Helm 安装命令"
  value = {
    prometheus = "helm install prometheus prometheus-community/kube-prometheus-stack --namespace monitoring --create-namespace"
    grafana    = "helm install grafana grafana/grafana --namespace monitoring"
    nats       = "helm install nats nats/nats --namespace nats --create-namespace"
  }
}
