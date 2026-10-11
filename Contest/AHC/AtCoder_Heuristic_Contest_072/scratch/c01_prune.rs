pub fn prune_ops(mut ops: Vec<Op>,initial: &BitBoard,grid: &Grid,target: usize,start: Instant,ms: u128) -> Vec<Op> {
    let deadline=deadline_at(start,ms);
    let Some(mut trace)=TraceCache::build(initial,&ops,grid,deadline) else {return ops;};
    for _ in 0..2 {
        let mut improved=false;
        for i in (0..ops.len()).rev() {
            if Instant::now()>=deadline {return ops;}
            match validate_patch(&ops,&trace,i,i+1,&[],grid,target,deadline) {
                CheckResult::Accept => {
                    ops.remove(i); record_patch(1,0); improved=true;
                    if !trace.rebuild_from(i,&ops,grid,deadline) {return ops;}
                }
                CheckResult::BudgetExpired => return ops,
                CheckResult::Reject => {}
            }
        }
        if !improved {break;}
    }
    ops
}
pub fn prune_cycle_and_uturn_ops(mut ops: Vec<Op>,initial: &BitBoard,grid: &Grid,target: usize,start: Instant,ms: u128) -> Vec<Op> {
    let deadline=deadline_at(start,ms);
    let Some(mut trace)=TraceCache::build(initial,&ops,grid,deadline) else {return ops;};
    for _ in 0..2 {
        let mut accepted=false;
        'scan: for i in 0..ops.len() {
            if Instant::now()>=deadline {return ops;}
            let (p,q)=trace.endpoints[i];
            for j in i+1..(i+12).min(ops.len()) {
                let (jp,jq)=trace.endpoints[j];
                if p==jq {
                    match validate_patch(&ops,&trace,i,j+1,&[],grid,target,deadline) {
                        CheckResult::Accept => {
                            record_patch(j+1-i,0); ops.drain(i..j+1); accepted=true;
                            if !trace.rebuild_from(i,&ops,grid,deadline) {return ops;}
                            break 'scan;
                        }
                        CheckResult::BudgetExpired => return ops,
                        CheckResult::Reject => {}
                    }
                }
                if jp==q && jq==p {
                    match validate_patch(&ops,&trace,i,j+1,&ops[i+1..j],grid,target,deadline) {
                        CheckResult::Accept => {
                            record_patch(2,0); ops.remove(j); ops.remove(i); accepted=true;
                            if !trace.rebuild_from(i,&ops,grid,deadline) {return ops;}
                            break 'scan;
                        }
                        CheckResult::BudgetExpired => return ops,
                        CheckResult::Reject => {}
                    }
                }
            }
        }
        if !accepted {break;}
    }
    ops
}
