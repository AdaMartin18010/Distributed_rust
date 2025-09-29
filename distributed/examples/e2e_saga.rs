use distributed::transactions::{Saga, SagaStep};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Instant;

/// 模拟银行账户
#[derive(Debug)]
struct Account {
    balance: Arc<AtomicUsize>,
    name: String,
}

impl Account {
    fn new(name: String, initial_balance: usize) -> Self {
        Self {
            balance: Arc::new(AtomicUsize::new(initial_balance)),
            name,
        }
    }
    
    fn get_balance(&self) -> usize {
        self.balance.load(Ordering::SeqCst)
    }
}

/// 借记操作 (从账户扣钱)
struct DebitStep {
    account: Arc<Account>,
    amount: usize,
}

impl SagaStep for DebitStep {
    fn execute(&mut self) -> Result<(), distributed::DistributedError> {
        let current = self.account.balance.load(Ordering::SeqCst);
        if current >= self.amount {
            self.account.balance.fetch_sub(self.amount, Ordering::SeqCst);
            println!("  💸 借记 {} 从账户 {} (余额: {} -> {})", 
                self.amount, 
                self.account.name, 
                current, 
                current - self.amount
            );
            Ok(())
        } else {
            println!("  ❌ 借记失败: 账户 {} 余额不足 (余额: {}, 需要: {})", 
                self.account.name, current, self.amount
            );
            Err(distributed::DistributedError::Storage(
                format!("Insufficient funds: account {}, available {}, required {}", 
                    self.account.name, current, self.amount)
            ))
        }
    }
    
    fn compensate(&mut self) -> Result<(), distributed::DistributedError> {
        self.account.balance.fetch_add(self.amount, Ordering::SeqCst);
        println!("  🔄 补偿借记: 退还 {} 到账户 {} (余额: {})", 
            self.amount, 
            self.account.name, 
            self.account.balance.load(Ordering::SeqCst)
        );
        Ok(())
    }
}

/// 贷记操作 (向账户加钱)
struct CreditStep {
    account: Arc<Account>,
    amount: usize,
}

impl SagaStep for CreditStep {
    fn execute(&mut self) -> Result<(), distributed::DistributedError> {
        let current = self.account.balance.load(Ordering::SeqCst);
        self.account.balance.fetch_add(self.amount, Ordering::SeqCst);
        println!("  💰 贷记 {} 到账户 {} (余额: {} -> {})", 
            self.amount, 
            self.account.name, 
            current, 
            current + self.amount
        );
        Ok(())
    }
    
    fn compensate(&mut self) -> Result<(), distributed::DistributedError> {
        self.account.balance.fetch_sub(self.amount, Ordering::SeqCst);
        println!("  🔄 补偿贷记: 扣除 {} 从账户 {} (余额: {})", 
            self.amount, 
            self.account.name, 
            self.account.balance.load(Ordering::SeqCst)
        );
        Ok(())
    }
}

/// 转账操作 (从一个账户转到另一个账户)
struct TransferStep {
    from: Arc<Account>,
    to: Arc<Account>,
    amount: usize,
}

impl SagaStep for TransferStep {
    fn execute(&mut self) -> Result<(), distributed::DistributedError> {
        println!("  🔄 转账 {} 从 {} 到 {}", 
            self.amount, self.from.name, self.to.name
        );
        
        // 检查余额
        let from_balance = self.from.balance.load(Ordering::SeqCst);
        if from_balance < self.amount {
            return Err(distributed::DistributedError::Storage(
                format!("Transfer failed: insufficient funds: available {}, required {}", 
                    from_balance, self.amount)
            ));
        }
        
        // 执行转账
        self.from.balance.fetch_sub(self.amount, Ordering::SeqCst);
        self.to.balance.fetch_add(self.amount, Ordering::SeqCst);
        
        println!("  ✅ 转账成功: {} -> {} (余额: {} -> {})", 
            self.from.name, self.to.name, 
            from_balance, from_balance - self.amount
        );
        Ok(())
    }
    
    fn compensate(&mut self) -> Result<(), distributed::DistributedError> {
        println!("  🔄 补偿转账: 退回 {} 从 {} 到 {}", 
            self.amount, self.to.name, self.from.name
        );
        
        self.to.balance.fetch_sub(self.amount, Ordering::SeqCst);
        self.from.balance.fetch_add(self.amount, Ordering::SeqCst);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Saga 分布式事务演示开始");
    
    // 1. 创建测试账户
    println!("\n🏦 创建测试账户...");
    let alice = Arc::new(Account::new("Alice".to_string(), 1000));
    let bob = Arc::new(Account::new("Bob".to_string(), 500));
    let charlie = Arc::new(Account::new("Charlie".to_string(), 200));
    
    println!("  👤 Alice: ${}", alice.get_balance());
    println!("  👤 Bob: ${}", bob.get_balance());
    println!("  👤 Charlie: ${}", charlie.get_balance());
    
    // 2. 成功转账场景
    println!("\n✅ 场景1: 成功转账 (Alice -> Bob -> Charlie)");
    let start = Instant::now();
    
    let saga = Saga::new()
        .then(Box::new(TransferStep {
            from: alice.clone(),
            to: bob.clone(),
            amount: 100,
        }))
        .then(Box::new(TransferStep {
            from: bob.clone(),
            to: charlie.clone(),
            amount: 50,
        }));
    
    match saga.run() {
        Ok(_) => {
            let duration = start.elapsed();
            println!("  🎉 Saga 执行成功! 耗时: {:?}", duration);
        }
        Err(e) => {
            println!("  ❌ Saga 执行失败: {:?}", e);
        }
    }
    
    println!("  📊 转账后余额:");
    println!("    👤 Alice: ${}", alice.get_balance());
    println!("    👤 Bob: ${}", bob.get_balance());
    println!("    👤 Charlie: ${}", charlie.get_balance());
    
    // 3. 失败回滚场景
    println!("\n❌ 场景2: 失败回滚 (余额不足)");
    
    // 重置账户余额
    alice.balance.store(50, Ordering::SeqCst);
    bob.balance.store(500, Ordering::SeqCst);
    charlie.balance.store(200, Ordering::SeqCst);
    
    println!("  📊 重置后余额:");
    println!("    👤 Alice: ${}", alice.get_balance());
    println!("    👤 Bob: ${}", bob.get_balance());
    println!("    👤 Charlie: ${}", charlie.get_balance());
    
    let saga = Saga::new()
        .then(Box::new(TransferStep {
            from: alice.clone(),
            to: bob.clone(),
            amount: 100, // Alice 只有 50，会失败
        }))
        .then(Box::new(TransferStep {
            from: bob.clone(),
            to: charlie.clone(),
            amount: 50,
        }));
    
    match saga.run() {
        Ok(_) => {
            println!("  🎉 Saga 执行成功!");
        }
        Err(e) => {
            println!("  ❌ Saga 执行失败，自动回滚: {:?}", e);
        }
    }
    
    println!("  📊 回滚后余额:");
    println!("    👤 Alice: ${}", alice.get_balance());
    println!("    👤 Bob: ${}", bob.get_balance());
    println!("    👤 Charlie: ${}", charlie.get_balance());
    
    // 4. 复杂业务场景
    println!("\n🏢 场景3: 复杂业务场景 (工资发放)");
    
    // 重置账户
    alice.balance.store(1000, Ordering::SeqCst);
    bob.balance.store(500, Ordering::SeqCst);
    charlie.balance.store(200, Ordering::SeqCst);
    
    let company = Arc::new(Account::new("Company".to_string(), 5000));
    
    println!("  🏢 公司账户: ${}", company.get_balance());
    
    let payroll_saga = Saga::new()
        .then(Box::new(DebitStep {
            account: company.clone(),
            amount: 300, // 总工资
        }))
        .then(Box::new(CreditStep {
            account: alice.clone(),
            amount: 100, // Alice 工资
        }))
        .then(Box::new(CreditStep {
            account: bob.clone(),
            amount: 100, // Bob 工资
        }))
        .then(Box::new(CreditStep {
            account: charlie.clone(),
            amount: 100, // Charlie 工资
        }));
    
    match payroll_saga.run() {
        Ok(_) => {
            println!("  🎉 工资发放成功!");
        }
        Err(e) => {
            println!("  ❌ 工资发放失败: {:?}", e);
        }
    }
    
    println!("  📊 工资发放后余额:");
    println!("    🏢 公司: ${}", company.get_balance());
    println!("    👤 Alice: ${}", alice.get_balance());
    println!("    👤 Bob: ${}", bob.get_balance());
    println!("    👤 Charlie: ${}", charlie.get_balance());
    
    println!("\n✅ Saga 分布式事务演示完成！");
    println!("\n📚 学习要点:");
    println!("  🔄 Saga 模式: 长事务分解为多个短事务");
    println!("  🛡️  补偿机制: 每个步骤都有对应的补偿操作");
    println!("  ⚡ 高性能: 避免长时间锁定资源");
    println!("  🔧 灵活性: 支持复杂的业务流程编排");
    
    Ok(())
}
