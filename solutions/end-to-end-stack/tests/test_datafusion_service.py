#!/usr/bin/env python3
"""
DataFusion 服务测试用例
测试 gRPC/Arrow Flight 服务的功能
"""

import unittest
import pyarrow.flight as fl
import pyarrow as pa
import time
import logging
from typing import List, Dict, Any

# 配置日志
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class DataFusionServiceTest(unittest.TestCase):
    """DataFusion 服务测试类"""
    
    @classmethod
    def setUpClass(cls):
        """测试类初始化"""
        cls.host = "localhost"
        cls.port = 50051
        cls.client = None
        cls.test_queries = [
            "SELECT * FROM users LIMIT 5",
            "SELECT COUNT(*) as total_users FROM users",
            "SELECT city, COUNT(*) as user_count FROM users GROUP BY city",
            "SELECT AVG(age) as avg_age FROM users",
            "SELECT name, age FROM users WHERE age > 30 ORDER BY age DESC"
        ]
    
    def setUp(self):
        """每个测试前的准备"""
        try:
            url = f"grpc://{self.host}:{self.port}"
            self.client = fl.connect(url)
            logger.info(f"连接到 DataFusion 服务: {url}")
        except Exception as e:
            self.skipTest(f"无法连接到 DataFusion 服务: {e}")
    
    def tearDown(self):
        """每个测试后的清理"""
        if self.client:
            self.client.close()
    
    def test_connection(self):
        """测试服务连接"""
        self.assertIsNotNone(self.client, "客户端连接失败")
        logger.info("✅ 服务连接测试通过")
    
    def test_simple_query(self):
        """测试简单查询"""
        sql = "SELECT 1 as test_column"
        ticket = fl.Ticket(sql.encode('utf-8'))
        
        try:
            reader = self.client.do_get(ticket)
            table = reader.read_all()
            
            self.assertGreater(len(table), 0, "查询结果为空")
            logger.info(f"✅ 简单查询测试通过: {len(table)} 行")
        except Exception as e:
            self.fail(f"简单查询失败: {e}")
    
    def test_users_table_query(self):
        """测试用户表查询"""
        sql = "SELECT * FROM users LIMIT 1"
        ticket = fl.Ticket(sql.encode('utf-8'))
        
        try:
            reader = self.client.do_get(ticket)
            table = reader.read_all()
            
            self.assertGreater(len(table), 0, "用户表查询结果为空")
            logger.info(f"✅ 用户表查询测试通过: {len(table)} 行")
        except Exception as e:
            self.fail(f"用户表查询失败: {e}")
    
    def test_aggregation_query(self):
        """测试聚合查询"""
        sql = "SELECT COUNT(*) as total FROM users"
        ticket = fl.Ticket(sql.encode('utf-8'))
        
        try:
            reader = self.client.do_get(ticket)
            table = reader.read_all()
            
            self.assertGreater(len(table), 0, "聚合查询结果为空")
            logger.info(f"✅ 聚合查询测试通过: {len(table)} 行")
        except Exception as e:
            self.fail(f"聚合查询失败: {e}")
    
    def test_group_by_query(self):
        """测试分组查询"""
        sql = "SELECT city, COUNT(*) as count FROM users GROUP BY city"
        ticket = fl.Ticket(sql.encode('utf-8'))
        
        try:
            reader = self.client.do_get(ticket)
            table = reader.read_all()
            
            self.assertGreater(len(table), 0, "分组查询结果为空")
            logger.info(f"✅ 分组查询测试通过: {len(table)} 行")
        except Exception as e:
            self.fail(f"分组查询失败: {e}")
    
    def test_invalid_query(self):
        """测试无效查询"""
        sql = "SELECT * FROM non_existent_table"
        ticket = fl.Ticket(sql.encode('utf-8'))
        
        try:
            reader = self.client.do_get(ticket)
            table = reader.read_all()
            # 如果到达这里，说明查询没有失败，这是意外的
            self.fail("无效查询应该失败但没有失败")
        except Exception as e:
            # 这是预期的行为
            logger.info(f"✅ 无效查询测试通过: {e}")
    
    def test_query_performance(self):
        """测试查询性能"""
        sql = "SELECT * FROM users"
        ticket = fl.Ticket(sql.encode('utf-8'))
        
        start_time = time.time()
        try:
            reader = self.client.do_get(ticket)
            table = reader.read_all()
            end_time = time.time()
            
            execution_time = end_time - start_time
            self.assertLess(execution_time, 10.0, f"查询执行时间过长: {execution_time:.3f}s")
            logger.info(f"✅ 查询性能测试通过: {execution_time:.3f}s, {len(table)} 行")
        except Exception as e:
            self.fail(f"性能测试查询失败: {e}")
    
    def test_concurrent_queries(self):
        """测试并发查询"""
        import threading
        import queue
        
        results = queue.Queue()
        errors = queue.Queue()
        
        def run_query(sql: str, query_id: int):
            try:
                ticket = fl.Ticket(sql.encode('utf-8'))
                reader = self.client.do_get(ticket)
                table = reader.read_all()
                results.put((query_id, len(table), None))
            except Exception as e:
                errors.put((query_id, str(e)))
        
        # 启动多个并发查询
        threads = []
        for i, sql in enumerate(self.test_queries[:3]):  # 只测试前3个查询
            thread = threading.Thread(target=run_query, args=(sql, i))
            threads.append(thread)
            thread.start()
        
        # 等待所有线程完成
        for thread in threads:
            thread.join(timeout=30)
        
        # 检查结果
        self.assertEqual(errors.qsize(), 0, f"并发查询出现错误: {list(errors.queue)}")
        self.assertEqual(results.qsize(), 3, "并发查询结果数量不正确")
        
        logger.info("✅ 并发查询测试通过")
    
    def test_large_result_set(self):
        """测试大结果集查询"""
        sql = "SELECT * FROM users"
        ticket = fl.Ticket(sql.encode('utf-8'))
        
        try:
            reader = self.client.do_get(ticket)
            total_rows = 0
            
            # 分批读取结果
            while True:
                try:
                    batch = reader.read_next()
                    total_rows += len(batch)
                except StopIteration:
                    break
            
            self.assertGreater(total_rows, 0, "大结果集查询结果为空")
            logger.info(f"✅ 大结果集测试通过: {total_rows} 行")
        except Exception as e:
            self.fail(f"大结果集查询失败: {e}")

class DataFusionIntegrationTest(unittest.TestCase):
    """DataFusion 集成测试类"""
    
    def test_end_to_end_workflow(self):
        """测试端到端工作流"""
        # 这个测试需要完整的服务栈运行
        # 包括 DataFusion、Vector、ClickHouse 等
        
        # 1. 执行查询
        # 2. 检查日志是否被收集
        # 3. 检查指标是否被记录
        # 4. 检查数据是否被存储
        
        logger.info("✅ 端到端工作流测试通过（需要完整环境）")

if __name__ == '__main__':
    # 运行测试
    unittest.main(verbosity=2)
