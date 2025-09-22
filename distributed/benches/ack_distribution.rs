#![allow(unused)]
use c20_distributed::consistency::ConsistencyLevel;
use c20_distributed::replication::MajorityQuorum;
use c20_distributed::replication::QuorumPolicy;

#[allow(dead_code)]
fn simulate_ack(total: usize, ok: usize) -> (usize, usize) {
    let need = MajorityQuorum::required_acks(total, ConsistencyLevel::Quorum);
    (ok, need)
}

// 注意：标准基准框架在稳定版上不可用。此文件仅保留计算逻辑，
// 便于后续接入 Criterion 或自定义驱动。
pub fn ack_table(total: usize) -> Vec<(usize, usize, bool)> {
    let mut v = Vec::new();
    for ok in 0..=total {
        let (o, need) = simulate_ack(total, ok);
        v.push((o, need, o >= need));
    }
    v
}

fn main() {
    // 提供一个可执行入口，便于在稳定版上直接运行该 bench 逻辑
    let tbl = ack_table(5);
    for (ok, need, pass) in tbl {
        println!("ok={ok} need={need} pass={pass}");
    }
}
