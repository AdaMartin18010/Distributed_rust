#!/usr/bin/env python3
"""
分布式计算栈性能测试
测试 DataFusion 服务、Vector 拓扑和整体系统的性能
"""

import time
import statistics
import concurrent.futures
import threading
import pyarrow.flight as fl
import requests
import json
import logging
from typing import List, Dict, Any, Tuple
from dataclasses import dataclass
import matplotlib.pyplot as plt
import pandas as pd

# 配置日志
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class PerformanceResult:
    """性能测试结果"""
    operation: str
    duration: float
    success: bool
    error: str = None
    metadata: Dict[str, Any] = None

class DataFusionPerformanceTest:
    """DataFusion 性能测试类"""
    
    def __init__(self, host="localhost", port=50051):
        self.host = host
        self.port = port
        self.client = None
        
    def connect(self):
        """连接到 DataFusion 服务"""
        try:
            url = f"grpc://{self.host}:{self.port}"
            self.client = fl.connect(url)
            logger.info(f"连接到 DataFusion 服务: {url}")
            return True
        except Exception as e:
            logger.error(f"连接失败: {e}")
            return False
    
    def execute_query(self, sql: str) -> PerformanceResult:
        """执行单个查询并测量性能"""
        start_time = time.time()
        try:
            ticket = fl.Ticket(sql.encode('utf-8'))
            reader = self.client.do_get(ticket)
            table = reader.read_all()
            
            end_time = time.time()
            duration = end_time - start_time
            
            return PerformanceResult(
                operation=f"query: {sql[:50]}...",
                duration=duration,
                success=True,
                metadata={"row_count": len(table)}
            )
        except Exception as e:
            end_time = time.time()
            duration = end_time - start_time
            
            return PerformanceResult(
                operation=f"query: {sql[:50]}...",
                duration=duration,
                success=False,
                error=str(e)
            )
    
    def test_single_query_performance(self, queries: List[str], iterations: int = 10) -> List[PerformanceResult]:
        """测试单个查询性能"""
        results = []
        
        for query in queries:
            logger.info(f"测试查询: {query[:50]}...")
            query_results = []
            
            for i in range(iterations):
                result = self.execute_query(query)
                query_results.append(result)
                time.sleep(0.1)  # 短暂休息
            
            results.extend(query_results)
        
        return results
    
    def test_concurrent_queries(self, queries: List[str], concurrency: int = 5) -> List[PerformanceResult]:
        """测试并发查询性能"""
        results = []
        
        def execute_query_worker(query: str) -> PerformanceResult:
            return self.execute_query(query)
        
        with concurrent.futures.ThreadPoolExecutor(max_workers=concurrency) as executor:
            # 提交所有查询任务
            future_to_query = {
                executor.submit(execute_query_worker, query): query 
                for query in queries
            }
            
            # 收集结果
            for future in concurrent.futures.as_completed(future_to_query):
                query = future_to_query[future]
                try:
                    result = future.result()
                    results.append(result)
                except Exception as e:
                    results.append(PerformanceResult(
                        operation=f"concurrent_query: {query[:50]}...",
                        duration=0,
                        success=False,
                        error=str(e)
                    ))
        
        return results
    
    def test_throughput(self, query: str, duration_seconds: int = 60) -> List[PerformanceResult]:
        """测试吞吐量"""
        results = []
        start_time = time.time()
        query_count = 0
        
        logger.info(f"开始吞吐量测试，持续 {duration_seconds} 秒...")
        
        while time.time() - start_time < duration_seconds:
            result = self.execute_query(query)
            results.append(result)
            query_count += 1
            
            if query_count % 10 == 0:
                elapsed = time.time() - start_time
                qps = query_count / elapsed
                logger.info(f"已执行 {query_count} 个查询，QPS: {qps:.2f}")
        
        return results

class VectorPerformanceTest:
    """Vector 性能测试类"""
    
    def __init__(self, edge_url="http://localhost:9598", agg_url="http://localhost:9599"):
        self.edge_url = edge_url
        self.agg_url = agg_url
    
    def get_metrics(self, url: str) -> Dict[str, Any]:
        """获取 Vector 指标"""
        try:
            response = requests.get(f"{url}/metrics", timeout=5)
            if response.status_code == 200:
                return self._parse_metrics(response.text)
            else:
                return {"error": f"HTTP {response.status_code}"}
        except Exception as e:
            return {"error": str(e)}
    
    def _parse_metrics(self, metrics_text: str) -> Dict[str, Any]:
        """解析 Prometheus 格式的指标"""
        metrics = {}
        for line in metrics_text.split('\n'):
            if line.startswith('#') or not line.strip():
                continue
            
            if ' ' in line:
                name, value = line.split(' ', 1)
                try:
                    metrics[name] = float(value)
                except ValueError:
                    metrics[name] = value
        
        return metrics
    
    def test_edge_performance(self) -> Dict[str, Any]:
        """测试 Vector Edge 性能"""
        return self.get_metrics(self.edge_url)
    
    def test_aggregator_performance(self) -> Dict[str, Any]:
        """测试 Vector Aggregator 性能"""
        return self.get_metrics(self.agg_url)
    
    def test_log_processing_rate(self, duration_seconds: int = 60) -> Dict[str, Any]:
        """测试日志处理速率"""
        logger.info(f"开始日志处理速率测试，持续 {duration_seconds} 秒...")
        
        # 获取初始指标
        initial_edge = self.test_edge_performance()
        initial_agg = self.test_aggregator_performance()
        
        time.sleep(duration_seconds)
        
        # 获取最终指标
        final_edge = self.test_edge_performance()
        final_agg = self.test_aggregator_performance()
        
        # 计算处理速率
        edge_processed = final_edge.get('vector_events_processed_total', 0) - initial_edge.get('vector_events_processed_total', 0)
        agg_processed = final_agg.get('vector_events_processed_total', 0) - initial_agg.get('vector_events_processed_total', 0)
        
        return {
            "duration_seconds": duration_seconds,
            "edge_events_processed": edge_processed,
            "agg_events_processed": agg_processed,
            "edge_events_per_second": edge_processed / duration_seconds,
            "agg_events_per_second": agg_processed / duration_seconds
        }

class SystemPerformanceTest:
    """系统整体性能测试类"""
    
    def __init__(self):
        self.datafusion_test = DataFusionPerformanceTest()
        self.vector_test = VectorPerformanceTest()
    
    def run_comprehensive_test(self) -> Dict[str, Any]:
        """运行综合性能测试"""
        logger.info("开始综合性能测试...")
        
        results = {
            "timestamp": time.time(),
            "datafusion": {},
            "vector": {},
            "system": {}
        }
        
        # 1. DataFusion 性能测试
        if self.datafusion_test.connect():
            logger.info("运行 DataFusion 性能测试...")
            
            # 测试查询
            test_queries = [
                "SELECT * FROM users LIMIT 100",
                "SELECT COUNT(*) FROM users",
                "SELECT city, COUNT(*) FROM users GROUP BY city",
                "SELECT AVG(age) FROM users",
                "SELECT * FROM users WHERE age > 30"
            ]
            
            # 单查询性能
            single_results = self.datafusion_test.test_single_query_performance(test_queries, 5)
            results["datafusion"]["single_query"] = self._analyze_results(single_results)
            
            # 并发查询性能
            concurrent_results = self.datafusion_test.test_concurrent_queries(test_queries, 3)
            results["datafusion"]["concurrent_query"] = self._analyze_results(concurrent_results)
            
            # 吞吐量测试
            throughput_results = self.datafusion_test.test_throughput("SELECT * FROM users LIMIT 10", 30)
            results["datafusion"]["throughput"] = self._analyze_results(throughput_results)
        
        # 2. Vector 性能测试
        logger.info("运行 Vector 性能测试...")
        results["vector"]["edge_metrics"] = self.vector_test.test_edge_performance()
        results["vector"]["agg_metrics"] = self.vector_test.test_aggregator_performance()
        results["vector"]["processing_rate"] = self.vector_test.test_log_processing_rate(30)
        
        # 3. 系统资源监控
        logger.info("收集系统资源信息...")
        results["system"]["resource_usage"] = self._get_system_resources()
        
        return results
    
    def _analyze_results(self, results: List[PerformanceResult]) -> Dict[str, Any]:
        """分析性能测试结果"""
        if not results:
            return {"error": "无测试结果"}
        
        successful_results = [r for r in results if r.success]
        failed_results = [r for r in results if not r.success]
        
        if not successful_results:
            return {
                "total_tests": len(results),
                "successful_tests": 0,
                "failed_tests": len(failed_results),
                "error": "所有测试都失败了"
            }
        
        durations = [r.duration for r in successful_results]
        
        return {
            "total_tests": len(results),
            "successful_tests": len(successful_results),
            "failed_tests": len(failed_results),
            "success_rate": len(successful_results) / len(results),
            "duration_stats": {
                "min": min(durations),
                "max": max(durations),
                "mean": statistics.mean(durations),
                "median": statistics.median(durations),
                "stdev": statistics.stdev(durations) if len(durations) > 1 else 0
            },
            "throughput_qps": len(successful_results) / sum(durations) if sum(durations) > 0 else 0
        }
    
    def _get_system_resources(self) -> Dict[str, Any]:
        """获取系统资源使用情况"""
        try:
            # 这里可以添加系统资源监控代码
            # 例如使用 psutil 库获取 CPU、内存使用率
            return {
                "cpu_usage": "N/A",
                "memory_usage": "N/A",
                "disk_usage": "N/A"
            }
        except Exception as e:
            return {"error": str(e)}
    
    def generate_report(self, results: Dict[str, Any]) -> str:
        """生成性能测试报告"""
        report = []
        report.append("=" * 80)
        report.append("分布式计算栈性能测试报告")
        report.append("=" * 80)
        report.append(f"测试时间: {time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(results['timestamp']))}")
        report.append("")
        
        # DataFusion 性能
        if "datafusion" in results:
            report.append("📊 DataFusion 性能测试结果:")
            for test_type, test_results in results["datafusion"].items():
                report.append(f"  {test_type}:")
                if "error" in test_results:
                    report.append(f"    错误: {test_results['error']}")
                else:
                    report.append(f"    总测试数: {test_results['total_tests']}")
                    report.append(f"    成功率: {test_results['success_rate']:.2%}")
                    if "duration_stats" in test_results:
                        stats = test_results["duration_stats"]
                        report.append(f"    平均延迟: {stats['mean']:.3f}s")
                        report.append(f"    中位数延迟: {stats['median']:.3f}s")
                        report.append(f"    最大延迟: {stats['max']:.3f}s")
                    if "throughput_qps" in test_results:
                        report.append(f"    吞吐量: {test_results['throughput_qps']:.2f} QPS")
            report.append("")
        
        # Vector 性能
        if "vector" in results:
            report.append("📈 Vector 性能测试结果:")
            for test_type, test_results in results["vector"].items():
                report.append(f"  {test_type}:")
                if isinstance(test_results, dict):
                    for key, value in test_results.items():
                        report.append(f"    {key}: {value}")
                else:
                    report.append(f"    {test_results}")
            report.append("")
        
        # 系统资源
        if "system" in results:
            report.append("💻 系统资源使用情况:")
            for key, value in results["system"].get("resource_usage", {}).items():
                report.append(f"  {key}: {value}")
            report.append("")
        
        report.append("=" * 80)
        return "\n".join(report)
    
    def save_results(self, results: Dict[str, Any], filename: str = "performance_results.json"):
        """保存测试结果到文件"""
        with open(filename, "w", encoding="utf-8") as f:
            json.dump(results, f, indent=2, ensure_ascii=False)
        logger.info(f"测试结果已保存到 {filename}")
    
    def plot_results(self, results: Dict[str, Any]):
        """绘制性能测试图表"""
        try:
            # 这里可以添加图表绘制代码
            # 例如使用 matplotlib 绘制延迟分布、吞吐量趋势等
            logger.info("图表绘制功能待实现")
        except Exception as e:
            logger.error(f"图表绘制失败: {e}")

def main():
    """主函数"""
    logger.info("开始分布式计算栈性能测试")
    
    # 创建性能测试实例
    system_test = SystemPerformanceTest()
    
    # 运行综合测试
    results = system_test.run_comprehensive_test()
    
    # 生成报告
    report = system_test.generate_report(results)
    print(report)
    
    # 保存结果
    system_test.save_results(results)
    
    # 绘制图表
    system_test.plot_results(results)
    
    logger.info("性能测试完成")

if __name__ == "__main__":
    main()
