use crate::log::LogEntry;
use crate::op::Operation;
use crate::hash::Hash;

pub fn create_merge_entry(local: &LogEntry, remote: &LogEntry, lamport: u64) -> LogEntry {
    let winner = select_winner(local, remote);
    
    let operation = Operation {
        wasm_hash: Hash::zero(),
        input: Vec::new(),
        parents: vec![local.id(), remote.id()],
        lamport,
    };

    let output = winner.output.clone();
    LogEntry::new(operation, output)
}

fn select_winner<'a>(local: &'a LogEntry, remote: &'a LogEntry) -> &'a LogEntry {
    if local.operation.lamport > remote.operation.lamport {
        local
    } else if remote.operation.lamport > local.operation.lamport {
        remote
    } else {
        if local.id() > remote.id() {
            local
        } else {
            remote
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::op::Operation;
    use crate::hash::Hash;

    #[test]
    fn test_merge_commutative() {
        let op1 = Operation::new(Hash::zero(), vec![1], vec![], 1);
        let op2 = Operation::new(Hash::zero(), vec![2], vec![], 2);
        
        let e1 = LogEntry::new(op1.clone(), vec![1]);
        let e2 = LogEntry::new(op2.clone(), vec![2]);

        let m1 = create_merge_entry(&e1, &e2, 3);
        let m2 = create_merge_entry(&e2, &e1, 3);

        assert_eq!(m1.output, m2.output);
    }
}
