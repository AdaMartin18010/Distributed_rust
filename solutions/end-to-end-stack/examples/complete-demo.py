#!/usr/bin/env python3
"""
端到端完整演示脚本
展示从客户端查询到日志收集的完整流程
"""

import pyarrow.flight as fl
import pyarrow as pa
import pandas as pd
import time
import logging
import json
import requests
from typing import List, Dict, Any

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class EndToEndDemo:
    def __init__(self, datafusion_host="localhost", datafusion_port=50051):
        self.datafusion_host = datafusion_host
        self.datafusion_port = datafusion_port
        self.client = None
        self.prometheus_url = "http://localhost:9090"
        self.grafana_url = "http://localhost:3000"
        
    def connect_datafusion(self):
        """连接到 DataFusion 服务"""
        try:
            url = f"grpc://{self.datafusion_host}:{self.datafusion_port}"
            logger.info(f"连接到 DataFusion 服务: {url}")
            self.client = fl.connect(url)
            logger.info("DataFusion 连接成功")
            return True
        except Exception as e:
            logger.error(f"DataFusion 连接失败: {e}")
            return False
    
    def execute_queries(self, queries: List[str]) -> Dict[str, Any]:
        """执行查询并收集性能指标"""
        results = {}
        
        for i, sql in enumerate(queries):
            logger.info(f"执行查询 {i+1}/{len(queries)}: {sql}")
            
            start_time = time.time()
            try:
                ticket = fl.Ticket(sql.encode('utf-8'))
                reader = self.client.do_get(ticket)
                table = reader.read_all()
                
                end_time = time.time()
                execution_time = end_time - start_time
                
                results[f"query_{i+1}"] = {
                    "sql": sql,
                    "execution_time": execution_time,
                    "row_count": len(table),
                    "success": True,
                    "data": table.to_pandas().to_dict('records')[:5]  # 只保存前5行
                }
                
                logger.info(f"查询完成: {execution_time:.3f}s, {len(table)} 行")
                
            except Exception as e:
                end_time = time.time()
                execution_time = end_time - start_time
                
                results[f"query_{i+1}"] = {
                    "sql": sql,
                    "execution_time": execution_time,
                    "row_count": 0,
                    "success": False,
                    "error": str(e)
                }
                
                logger.error(f"查询失败: {e}")
            
            # 等待一段时间，让日志有时间被收集
            time.sleep(1)
        
        return results
    
    def check_prometheus_metrics(self) -> Dict[str, Any]:
        """检查 Prometheus 指标"""
        try:
            # 查询 DataFusion 相关指标
            queries = [
                "up{job=\"datafusion\"}",
                "rate(http_requests_total[5m])",
                "http_request_duration_seconds",
                "vector_events_processed_total"
            ]
            
            metrics = {}
            for query in queries:
                try:
                    response = requests.get(
                        f"{self.prometheus_url}/api/v1/query",
                        params={"query": query},
                        timeout=5
                    )
                    if response.status_code == 200:
                        data = response.json()
                        metrics[query] = data.get("data", {}).get("result", [])
                    else:
                        metrics[query] = f"Error: {response.status_code}"
                except Exception as e:
                    metrics[query] = f"Error: {e}"
            
            return metrics
        except Exception as e:
            logger.error(f"Prometheus 检查失败: {e}")
            return {"error": str(e)}
    
    def check_grafana_dashboard(self) -> bool:
        """检查 Grafana 仪表板是否可访问"""
        try:
            response = requests.get(f"{self.grafana_url}/api/health", timeout=5)
            return response.status_code == 200
        except Exception as e:
            logger.error(f"Grafana 检查失败: {e}")
            return False
    
    def generate_report(self, query_results: Dict[str, Any], 
                       metrics: Dict[str, Any]) -> str:
        """生成演示报告"""
        report = []
        report.append("=" * 60)
        report.append("端到端分布式计算演示报告")
        report.append("=" * 60)
        
        # 查询结果统计
        report.append("\n📊 查询执行统计:")
        total_queries = len(query_results)
        successful_queries = sum(1 for r in query_results.values() if r["success"])
        total_time = sum(r["execution_time"] for r in query_results.values())
        
        report.append(f"  总查询数: {total_queries}")
        report.append(f"  成功查询: {successful_queries}")
        report.append(f"  失败查询: {total_queries - successful_queries}")
        report.append(f"  总执行时间: {total_time:.3f}s")
        report.append(f"  平均执行时间: {total_time/total_queries:.3f}s")
        
        # 详细查询结果
        report.append("\n🔍 详细查询结果:")
        for key, result in query_results.items():
            status = "✅" if result["success"] else "❌"
            report.append(f"  {status} {key}: {result['execution_time']:.3f}s")
            if result["success"]:
                report.append(f"    行数: {result['row_count']}")
            else:
                report.append(f"    错误: {result['error']}")
        
        # 监控指标
        report.append("\n📈 监控指标状态:")
        if "error" not in metrics:
            for query, data in metrics.items():
                if isinstance(data, list) and data:
                    report.append(f"  ✅ {query}: {len(data)} 个指标")
                else:
                    report.append(f"  ⚠️  {query}: 无数据")
        else:
            report.append(f"  ❌ Prometheus 连接失败: {metrics['error']}")
        
        # Grafana 状态
        grafana_status = self.check_grafana_dashboard()
        report.append(f"\n📊 Grafana 仪表板: {'✅ 可访问' if grafana_status else '❌ 不可访问'}")
        
        report.append("\n" + "=" * 60)
        return "\n".join(report)
    
    def run_demo(self):
        """运行完整演示"""
        logger.info("开始端到端演示")
        
        # 1. 连接到 DataFusion
        if not self.connect_datafusion():
            logger.error("无法连接到 DataFusion 服务")
            return
        
        # 2. 执行示例查询
        sample_queries = [
            "SELECT * FROM users LIMIT 10",
            "SELECT name, age FROM users WHERE age > 30 ORDER BY age DESC",
            "SELECT city, COUNT(*) as user_count FROM users GROUP BY city ORDER BY user_count DESC",
            "SELECT AVG(age) as avg_age, MIN(age) as min_age, MAX(age) as max_age FROM users",
            "SELECT name, age, city FROM users WHERE city IN ('New York', 'San Francisco') ORDER BY age"
        ]
        
        logger.info("执行示例查询...")
        query_results = self.execute_queries(sample_queries)
        
        # 3. 检查监控指标
        logger.info("检查监控指标...")
        metrics = self.check_prometheus_metrics()
        
        # 4. 生成报告
        report = self.generate_report(query_results, metrics)
        print(report)
        
        # 5. 保存结果到文件
        with open("demo_results.json", "w", encoding="utf-8") as f:
            json.dump({
                "query_results": query_results,
                "metrics": metrics,
                "timestamp": time.time()
            }, f, indent=2, ensure_ascii=False)
        
        logger.info("演示完成，结果已保存到 demo_results.json")
    
    def close(self):
        """关闭连接"""
        if self.client:
            self.client.close()
            logger.info("连接已关闭")

def main():
    """主函数"""
    demo = EndToEndDemo()
    
    try:
        demo.run_demo()
    finally:
        demo.close()

if __name__ == "__main__":
    main()
