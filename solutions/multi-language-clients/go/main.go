package main

import (
	"context"
	"fmt"
	"log"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

// 简化的 gRPC 客户端示例
// 注意：实际使用时需要生成对应的 protobuf 代码

func main() {
	// 连接到服务
	conn, err := grpc.Dial("localhost:50051", grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		log.Fatalf("连接失败: %v", err)
	}
	defer conn.Close()

	// 创建客户端
	// client := pb.NewDataFusionClient(conn)

	// 示例查询
	queries := []string{
		"SELECT * FROM users LIMIT 5",
		"SELECT name, age FROM users WHERE age > 30",
		"SELECT city, COUNT(*) as user_count FROM users GROUP BY city",
	}

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	for _, sql := range queries {
		fmt.Printf("\n%s\n", "="*50)
		fmt.Printf("查询: %s\n", sql)
		fmt.Printf("%s\n", "="*50)

		// 执行查询
		// req := &pb.QueryRequest{Sql: sql}
		// resp, err := client.ExecuteQuery(ctx, req)
		// if err != nil {
		//     log.Printf("查询失败: %v", err)
		//     continue
		// }

		// 处理结果
		// fmt.Printf("结果: %s\n", resp.Result)

		// 临时输出
		fmt.Printf("查询执行成功 (需要生成 protobuf 代码)\n")
	}
}
