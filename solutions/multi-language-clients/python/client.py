#!/usr/bin/env python3
"""
Python 客户端示例 - Arrow Flight
连接到 DataFusion 服务并执行 SQL 查询
"""

import pyarrow.flight as fl
import pyarrow as pa
import pandas as pd
import sys
import logging

# 配置日志
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class DataFusionClient:
    def __init__(self, host="localhost", port=50051):
        self.host = host
        self.port = port
        self.client = None
        
    def connect(self):
        """连接到 DataFusion 服务"""
        try:
            url = f"grpc://{self.host}:{self.port}"
            logger.info(f"连接到 DataFusion 服务: {url}")
            self.client = fl.connect(url)
            logger.info("连接成功")
            return True
        except Exception as e:
            logger.error(f"连接失败: {e}")
            return False
    
    def execute_query(self, sql):
        """执行 SQL 查询"""
        if not self.client:
            raise RuntimeError("客户端未连接")
        
        try:
            logger.info(f"执行查询: {sql}")
            ticket = fl.Ticket(sql.encode('utf-8'))
            reader = self.client.do_get(ticket)
            
            # 读取所有数据
            table = reader.read_all()
            logger.info(f"查询完成，返回 {len(table)} 行数据")
            
            return table
        except Exception as e:
            logger.error(f"查询执行失败: {e}")
            raise
    
    def query_to_dataframe(self, sql):
        """执行查询并返回 pandas DataFrame"""
        table = self.execute_query(sql)
        return table.to_pandas()
    
    def close(self):
        """关闭连接"""
        if self.client:
            self.client.close()
            logger.info("连接已关闭")

def main():
    """主函数"""
    client = DataFusionClient()
    
    try:
        # 连接到服务
        if not client.connect():
            sys.exit(1)
        
        # 示例查询
        queries = [
            "SELECT * FROM users LIMIT 5",
            "SELECT name, age FROM users WHERE age > 30",
            "SELECT city, COUNT(*) as user_count FROM users GROUP BY city",
        ]
        
        for sql in queries:
            print(f"\n{'='*50}")
            print(f"查询: {sql}")
            print('='*50)
            
            try:
                df = client.query_to_dataframe(sql)
                print(df.to_string(index=False))
            except Exception as e:
                print(f"查询失败: {e}")
    
    finally:
        client.close()

if __name__ == "__main__":
    main()
