package com.example.datafusion;

import io.grpc.ManagedChannel;
import io.grpc.ManagedChannelBuilder;
import io.grpc.StatusRuntimeException;

import java.util.concurrent.TimeUnit;
import java.util.logging.Logger;

/**
 * Java 客户端示例 - gRPC
 * 连接到 DataFusion 服务并执行 SQL 查询
 */
public class DataFusionClient {
    private static final Logger logger = Logger.getLogger(DataFusionClient.class.getName());

    private final ManagedChannel channel;
    // private final DataFusionGrpc.DataFusionBlockingStub blockingStub;

    public DataFusionClient(String host, int port) {
        this(ManagedChannelBuilder.forAddress(host, port)
                .usePlaintext()
                .build());
    }

    public DataFusionClient(ManagedChannel channel) {
        this.channel = channel;
        // this.blockingStub = DataFusionGrpc.newBlockingStub(channel);
    }

    public void shutdown() throws InterruptedException {
        channel.shutdown().awaitTermination(5, TimeUnit.SECONDS);
    }

    /**
     * 执行 SQL 查询
     */
    public void executeQuery(String sql) {
        logger.info("执行查询: " + sql);
        
        try {
            // QueryRequest request = QueryRequest.newBuilder()
            //     .setSql(sql)
            //     .build();
            
            // QueryResponse response = blockingStub.executeQuery(request);
            // logger.info("查询结果: " + response.getResult());
            
            // 临时输出
            logger.info("查询执行成功 (需要生成 protobuf 代码)");
            
        } catch (StatusRuntimeException e) {
            logger.warning("RPC 失败: " + e.getStatus());
        }
    }

    public static void main(String[] args) throws Exception {
        DataFusionClient client = new DataFusionClient("localhost", 50051);
        
        try {
            String[] queries = {
                "SELECT * FROM users LIMIT 5",
                "SELECT name, age FROM users WHERE age > 30",
                "SELECT city, COUNT(*) as user_count FROM users GROUP BY city"
            };

            for (String sql : queries) {
                System.out.println("\n" + "=".repeat(50));
                System.out.println("查询: " + sql);
                System.out.println("=".repeat(50));
                
                client.executeQuery(sql);
            }
        } finally {
            client.shutdown();
        }
    }
}
