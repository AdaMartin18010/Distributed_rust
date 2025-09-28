#!/bin/bash

# 端到端部署脚本
# 一键部署完整的分布式计算栈

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    log_info "检查依赖..."
    
    local deps=("docker" "docker-compose" "kubectl" "helm")
    local missing=()
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            missing+=("$dep")
        fi
    done
    
    if [ ${#missing[@]} -ne 0 ]; then
        log_error "缺少依赖: ${missing[*]}"
        log_info "请安装缺少的依赖后重试"
        exit 1
    fi
    
    log_success "所有依赖检查通过"
}

# 检查 Docker 是否运行
check_docker() {
    log_info "检查 Docker 状态..."
    
    if ! docker info &> /dev/null; then
        log_error "Docker 未运行，请启动 Docker 后重试"
        exit 1
    fi
    
    log_success "Docker 运行正常"
}

# 构建镜像
build_images() {
    log_info "构建 Docker 镜像..."
    
    # 构建 DataFusion 服务镜像
    log_info "构建 DataFusion 服务镜像..."
    cd ../foundations-datafusion
    docker build -f ../deployment-strategies/docker/Dockerfile -t datafusion:latest .
    cd - > /dev/null
    
    log_success "镜像构建完成"
}

# 启动开发环境
deploy_dev() {
    log_info "部署开发环境..."
    
    cd ../deployment-strategies/docker
    
    # 停止现有容器
    log_info "停止现有容器..."
    docker-compose down --remove-orphans
    
    # 启动服务
    log_info "启动服务..."
    docker-compose up -d
    
    # 等待服务启动
    log_info "等待服务启动..."
    sleep 30
    
    # 检查服务状态
    log_info "检查服务状态..."
    docker-compose ps
    
    cd - > /dev/null
    
    log_success "开发环境部署完成"
}

# 部署生产环境
deploy_prod() {
    log_info "部署生产环境..."
    
    # 检查 Kubernetes 集群
    if ! kubectl cluster-info &> /dev/null; then
        log_error "无法连接到 Kubernetes 集群"
        exit 1
    fi
    
    # 创建命名空间
    log_info "创建命名空间..."
    kubectl create namespace datafusion --dry-run=client -o yaml | kubectl apply -f -
    
    # 部署 DataFusion 服务
    log_info "部署 DataFusion 服务..."
    kubectl apply -f ../deployment-strategies/kubernetes/datafusion-deployment.yaml
    
    # 等待部署完成
    log_info "等待部署完成..."
    kubectl wait --for=condition=available --timeout=300s deployment/datafusion -n datafusion
    
    # 检查服务状态
    log_info "检查服务状态..."
    kubectl get pods -n datafusion
    kubectl get services -n datafusion
    
    log_success "生产环境部署完成"
}

# 运行测试
run_tests() {
    log_info "运行测试..."
    
    # 等待服务完全启动
    log_info "等待服务启动..."
    sleep 60
    
    # 运行端到端测试
    log_info "运行端到端测试..."
    cd ../end-to-end-stack/examples
    
    if command -v python3 &> /dev/null; then
        python3 complete-demo.py
    elif command -v python &> /dev/null; then
        python complete-demo.py
    else
        log_warning "未找到 Python，跳过端到端测试"
    fi
    
    cd - > /dev/null
    
    log_success "测试完成"
}

# 显示访问信息
show_access_info() {
    log_info "服务访问信息:"
    echo ""
    echo "🌐 Web 界面:"
    echo "  - Grafana: http://localhost:3000 (admin/admin)"
    echo "  - Prometheus: http://localhost:9090"
    echo ""
    echo "🔧 服务端点:"
    echo "  - DataFusion gRPC: localhost:50051"
    echo "  - DataFusion HTTP: localhost:8080"
    echo "  - NATS: localhost:4222"
    echo "  - ClickHouse: localhost:8123"
    echo ""
    echo "📊 监控端点:"
    echo "  - Vector Edge Metrics: localhost:9598"
    echo "  - Vector Aggregator Metrics: localhost:9599"
    echo ""
    echo "📝 日志查看:"
    echo "  docker-compose logs -f datafusion"
    echo "  docker-compose logs -f vector-edge"
    echo "  docker-compose logs -f vector-aggregator"
}

# 清理环境
cleanup() {
    log_info "清理环境..."
    
    cd ../deployment-strategies/docker
    docker-compose down --volumes --remove-orphans
    cd - > /dev/null
    
    log_success "环境清理完成"
}

# 主函数
main() {
    echo "=========================================="
    echo "    分布式计算栈部署脚本"
    echo "=========================================="
    echo ""
    
    # 解析参数
    case "${1:-dev}" in
        "dev")
            log_info "部署模式: 开发环境"
            check_dependencies
            check_docker
            build_images
            deploy_dev
            run_tests
            show_access_info
            ;;
        "prod")
            log_info "部署模式: 生产环境"
            check_dependencies
            build_images
            deploy_prod
            ;;
        "test")
            log_info "运行测试模式"
            run_tests
            ;;
        "cleanup")
            log_info "清理模式"
            cleanup
            ;;
        "help"|"-h"|"--help")
            echo "用法: $0 [dev|prod|test|cleanup|help]"
            echo ""
            echo "选项:"
            echo "  dev     部署开发环境 (默认)"
            echo "  prod    部署生产环境"
            echo "  test    运行测试"
            echo "  cleanup 清理环境"
            echo "  help    显示帮助信息"
            ;;
        *)
            log_error "未知选项: $1"
            echo "使用 '$0 help' 查看帮助信息"
            exit 1
            ;;
    esac
}

# 错误处理
trap 'log_error "脚本执行失败，退出码: $?"' ERR

# 执行主函数
main "$@"
