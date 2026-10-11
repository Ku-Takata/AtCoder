// C_01: exact interval validation and bounded compact-state search.
#[derive(Default, Clone, Copy)]
struct PatchStats {
    trials: usize, accepted: usize, saved_ops: usize, replay_steps: usize,
    rejoin_accepts: usize, compact_nodes: usize, budget_stops: usize,
}
thread_local! {
    static PATCH_STATS: std::cell::RefCell<PatchStats> = std::cell::RefCell::new(PatchStats::default());
}
fn stat(f: impl FnOnce(&mut PatchStats)) { PATCH_STATS.with(|s| f(&mut s.borrow_mut())); }
fn stats_snapshot() -> PatchStats { PATCH_STATS.with(|s| *s.borrow()) }
pub fn post_mode() -> String {
    std::env::args().find_map(|s| s.strip_prefix("--post=").map(str::to_owned))
        .unwrap_or_else(|| "local".into())
}
pub fn print_patch_stats() {
    PATCH_STATS.with(|s| {
        let s = s.borrow();
        eprintln!("PATCH: trials={} accepted={} saved_ops={} replay_steps={} rejoin_accepts={} compact_nodes={} budget_stops={}",
            s.trials, s.accepted, s.saved_ops, s.replay_steps, s.rejoin_accepts, s.compact_nodes, s.budget_stops);
    });
}
fn phase_log(name: &str, begin: Instant, before: usize, after: usize) {
    eprintln!("PHASE: name={} elapsed_ms={:.3} before={} after={} saved_ops={}",
        name, begin.elapsed().as_secs_f64()*1000.0, before, after, before.saturating_sub(after));
}
fn phase_stats(name: &str,before: PatchStats) {
    let after=stats_snapshot();
    eprintln!("PHASE_STATS: name={} trials={} accepted={} saved_ops={} replay_steps={} rejoin_accepts={} compact_nodes={} budget_stops={}",
        name,after.trials-before.trials,after.accepted-before.accepted,after.saved_ops-before.saved_ops,
        after.replay_steps-before.replay_steps,after.rejoin_accepts-before.rejoin_accepts,
        after.compact_nodes-before.compact_nodes,after.budget_stops-before.budget_stops);
}
fn endpoint(op: Op, grid: &Grid) -> (usize, usize) {
    let (dr, dc) = DIR[op.d];
    (op.r * grid.n + op.c,
     (op.r as isize + dr * op.l as isize) as usize * grid.n
        + (op.c as isize + dc * op.l as isize) as usize)
}
fn deadline_at(start: Instant, ms: u128) -> Instant {
    start + std::time::Duration::from_millis(ms as u64)
}
struct TraceCache {
    boards: Vec<BitBoard>, cleared: Vec<usize>, endpoints: Vec<(usize, usize)>,
}
impl TraceCache {
    fn build(initial: &BitBoard, ops: &[Op], grid: &Grid, deadline: Instant) -> Option<Self> {
        let mut trace = Self { boards: vec![*initial], cleared: vec![0], endpoints: Vec::with_capacity(ops.len()) };
        trace.rebuild_from(0, ops, grid, deadline).then_some(trace)
    }
    fn rebuild_from(&mut self, a: usize, ops: &[Op], grid: &Grid, deadline: Instant) -> bool {
        self.boards.truncate(a+1); self.cleared.truncate(a+1); self.endpoints.truncate(a);
        let mut board = self.boards[a]; let mut cleared = self.cleared[a];
        for (t, &op) in ops.iter().enumerate().skip(a) {
            if t % 32 == 0 && Instant::now() >= deadline { stat(|s| s.budget_stops += 1); return false; }
            let Some(c) = apply_op(&mut board, op, grid) else { return false; };
            cleared += c;
            self.boards.push(board); self.cleared.push(cleared); self.endpoints.push(endpoint(op, grid));
        }
        Instant::now() < deadline
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CheckResult { Accept, Reject, BudgetExpired }
#[inline]
fn same_cell(a: &BitBoard, b: &BitBoard, p: usize) -> bool {
    a.lens[p] == b.lens[p] && a.colors[p] == b.colors[p]
}
fn validate_patch(ops: &[Op], trace: &TraceCache, a: usize, b: usize,
    replacement: &[Op], grid: &Grid, target: usize, deadline: Instant) -> CheckResult {
    stat(|s| s.trials += 1);
    if Instant::now() >= deadline { stat(|s| s.budget_stops += 1); return CheckResult::BudgetExpired; }
    let mut board = trace.boards[a]; let mut cleared = trace.cleared[a];
    let mut touched = [0u64; 7];
    for &(p, q) in &trace.endpoints[a..b] {
        touched[p/64] |= 1 << (p%64); touched[q/64] |= 1 << (q%64);
    }
    for (t, &op) in replacement.iter().enumerate() {
        if t % 32 == 0 && Instant::now() >= deadline { stat(|s| s.budget_stops += 1); return CheckResult::BudgetExpired; }
        let Some(c) = apply_op(&mut board, op, grid) else { return CheckResult::Reject; };
        cleared += c;
        let (p,q) = endpoint(op, grid);
        touched[p/64] |= 1 << (p%64); touched[q/64] |= 1 << (q%64);
    }
    let mut diff = [0u64; 7]; let mut count = 0usize;
    for w in 0..7 {
        let mut bits = touched[w];
        while bits != 0 {
            let p = w*64 + bits.trailing_zeros() as usize; bits &= bits-1;
            if !same_cell(&board, &trace.boards[b], p) { diff[w] |= 1 << (p%64); count += 1; }
        }
    }
    let total = *trace.cleared.last().unwrap(); let mut replay_steps = 0;
    for t in b..=ops.len() {
        if count == 0 {
            if Instant::now() >= deadline { stat(|s| s.budget_stops += 1); return CheckResult::BudgetExpired; }
            let accepted = cleared + total - trace.cleared[t] >= target;
            stat(|s| { s.replay_steps += replay_steps; if accepted { s.rejoin_accepts += 1; } });
            return if accepted { CheckResult::Accept } else { CheckResult::Reject };
        }
        if t == ops.len() { break; }
        if replay_steps % 32 == 0 && Instant::now() >= deadline {
            stat(|s| { s.replay_steps += replay_steps; s.budget_stops += 1; }); return CheckResult::BudgetExpired;
        }
        let Some(c) = apply_op(&mut board, ops[t], grid) else {
            stat(|s| s.replay_steps += replay_steps); return CheckResult::Reject;
        };
        cleared += c; replay_steps += 1;
        let (p,q) = trace.endpoints[t];
        for x in [p,q] {
            let mask = 1u64 << (x%64); let was = diff[x/64] & mask != 0;
            let now = !same_cell(&board, &trace.boards[t+1], x);
            if was != now { if now { count += 1; diff[x/64] |= mask; } else { count -= 1; diff[x/64] &= !mask; } }
        }
    }
    stat(|s| s.replay_steps += replay_steps);
    if Instant::now() >= deadline { stat(|s| s.budget_stops += 1); return CheckResult::BudgetExpired; }
    if cleared >= target { CheckResult::Accept } else { CheckResult::Reject }
}
fn record_patch(old: usize, new: usize) { stat(|s| { s.accepted += 1; s.saved_ops += old-new; }); }
fn ray(p: usize, q: usize, grid: &Grid) -> Option<(usize, usize)> {
    if p == q || !grid.is_floor[p] || !grid.is_floor[q] { return None; }
    let (pr,pc,qr,qc) = (p/grid.n,p%grid.n,q/grid.n,q%grid.n);
    let (d,l) = if pr == qr { if pc < qc { (3,qc-pc) } else { (2,pc-qc) } }
        else if pc == qc { if pr < qr { (1,qr-pr) } else { (0,pr-qr) } } else { return None; };
    (l <= grid.max_reach[p][d]).then_some((d,l))
}
fn bridges(p: usize, q: usize, grid: &Grid) -> Vec<usize> {
    let (pr,pc,qr,qc) = (p/grid.n,p%grid.n,q/grid.n,q%grid.n);
    let xs: Vec<usize> = if pr == qr { ((pc.min(qc)+1)..pc.max(qc)).map(|c| pr*grid.n+c).collect() }
        else if pc == qc { ((pr.min(qr)+1)..pr.max(qr)).map(|r| r*grid.n+pc).collect() }
        else { vec![pr*grid.n+qc, qr*grid.n+pc] };
    xs.into_iter().filter(|&x| ray(p,x,grid).is_some() && ray(x,q,grid).is_some()).collect()
}
pub fn complete_2hop_jumps(mut ops: Vec<Op>, initial: &BitBoard, grid: &Grid,
    target: usize, deadline: Instant) -> Vec<Op> {
    let begin = Instant::now(); let before = ops.len(); let stats_before=stats_snapshot();
    let Some(mut trace) = TraceCache::build(initial,&ops,grid,deadline) else { return ops; };
    let mut a = 0;
    'search: while a+3 <= ops.len() && Instant::now() < deadline {
        let p = trace.endpoints[a].0;
        for len in 3..=8 {
            let b = a+len; if b > ops.len() { break; }
            let q = trace.endpoints[b-1].1;
            let mut fallback = Vec::new();
            for x in bridges(p,q,grid) {
                let (d1,l1) = ray(p,x,grid).unwrap(); let (d2,l2) = ray(x,q,grid).unwrap();
                for k1 in (l1-1)..trace.boards[a].lens[p] as usize {
                    if Instant::now() >= deadline { break 'search; }
                    let op1 = Op { r:p/grid.n,c:p%grid.n,k:k1,d:d1,l:l1 };
                    let mut after1 = trace.boards[a];
                    if apply_op(&mut after1,op1,grid).is_none() { continue; }
                    for k2 in (l2-1)..after1.lens[x] as usize {
                        let op2 = Op { r:x/grid.n,c:x%grid.n,k:k2,d:d2,l:l2 };
                        let mut after2 = after1;
                        if apply_op(&mut after2,op2,grid).is_none() { continue; }
                        let patch = [op1,op2];
                        if after2 != trace.boards[b] { fallback.push(patch); continue; }
                        if validate_patch(&ops,&trace,a,b,&patch,grid,target,deadline) == CheckResult::Accept {
                            ops.splice(a..b,patch); record_patch(len,2);
                            if !trace.rebuild_from(a,&ops,grid,deadline) { break 'search; }
                            a = a.saturating_sub(1); continue 'search;
                        }
                    }
                }
            }
            for patch in fallback {
                match validate_patch(&ops,&trace,a,b,&patch,grid,target,deadline) {
                    CheckResult::Accept => {
                        ops.splice(a..b,patch); record_patch(len,2);
                        if !trace.rebuild_from(a,&ops,grid,deadline) { break 'search; }
                        a = a.saturating_sub(1); continue 'search;
                    }
                    CheckResult::BudgetExpired => break 'search,
                    CheckResult::Reject => {}
                }
            }
        }
        a += 1;
    }
    phase_log("bridge",begin,before,ops.len()); phase_stats("bridge",stats_before); ops
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct LocalState { cells: [u64;12] }
fn pack_cell(board: &BitBoard,p: usize) -> u64 { board.colors[p] as u64 | ((board.lens[p] as u64)<<32) }
fn local_state(board: &BitBoard,region: &[usize]) -> LocalState {
    let mut s = LocalState { cells:[0;12] };
    for (i,&p) in region.iter().enumerate() { s.cells[i] = pack_cell(board,p); } s
}
#[inline]
fn local_clean(cell: &mut u64,nest: Option<usize>) {
    let Some(c) = nest else { return; };
    let mut h = (*cell>>32) as usize; let mut bits = *cell as u32;
    while h>0 && ((bits>>((h-1)*4))&15) as usize == c {
        h -= 1; bits &= if h==0 {0} else {(1u32<<(h*4))-1};
    }
    *cell = bits as u64 | ((h as u64)<<32);
}
fn local_move(s: LocalState,i: usize,j: usize,k: usize,l: usize,nests: &[Option<usize>;12]) -> Option<LocalState> {
    let h = (s.cells[i]>>32) as usize; let g = (s.cells[j]>>32) as usize;
    if k>=h || l==0 || l>k+1 || g+h-k>8 { return None; }
    let mut next = s; let mut bits = (s.cells[i] as u32)>>(k*4); let mut rev=0u32;
    for _ in k..h { rev = (rev<<4)|(bits&15); bits >>= 4; }
    let mask = if k==0 {0} else {(1u32<<(k*4))-1};
    next.cells[i] = ((s.cells[i] as u32 & mask) as u64)|((k as u64)<<32);
    next.cells[j] = ((s.cells[j] as u32 | (rev<<(g*4))) as u64)|(((g+h-k) as u64)<<32);
    local_clean(&mut next.cells[i],nests[i]); local_clean(&mut next.cells[j],nests[j]); Some(next)
}
fn color_counts(s: &LocalState) -> [u8;16] {
    let mut counts=[0;16];
    for &cell in &s.cells {
        let mut bits=cell as u32;
        for _ in 0..(cell>>32) { counts[(bits&15) as usize]+=1; bits>>=4; }
    } counts
}
struct LocalSearch<'a> {
    region: &'a [usize], n: usize, nests: [Option<usize>;12], rays: [[Option<(usize,usize)>;12];12],
    goal: LocalState, required: [u8;16], seen: FastHashMap<LocalState,usize>,
    deadline: Instant, nodes: usize, stopped: bool,
}
impl LocalSearch<'_> {
    fn dfs(&mut self,s: LocalState,depth: usize,limit: usize,path: &mut Vec<Op>) -> bool {
        if self.nodes >= 2000 || (self.nodes%32==0 && Instant::now()>=self.deadline) {
            self.stopped=true; return false;
        }
        self.nodes+=1;
        if s==self.goal { return true; }
        let diff=s.cells.iter().zip(self.goal.cells.iter()).filter(|(a,b)|a!=b).count();
        if diff>2*(limit-depth) || depth==limit { return false; }
        let counts=color_counts(&s);
        if (0..16).any(|c|counts[c]<self.required[c]) { return false; }
        if self.seen.get(&s).is_some_and(|&d|d<=depth) { return false; }
        self.seen.insert(s,depth);
        let mut moves=Vec::new(); let mut generated=0;
        for i in 0..self.region.len() {
            let h=(s.cells[i]>>32) as usize;
            for j in 0..self.region.len() {
                let Some((d,l))=self.rays[i][j] else {continue;};
                for k in (l-1)..h {
                    generated+=1;
                    if generated%32==0 && Instant::now()>=self.deadline { self.stopped=true; return false; }
                    let Some(next)=local_move(s,i,j,k,l,&self.nests) else {continue;};
                    if next==s {continue;}
                    let nd=next.cells.iter().zip(self.goal.cells.iter()).filter(|(a,b)|a!=b).count();
                    let p=self.region[i];
                    moves.push((nd,next,Op{r:p/self.n,c:p%self.n,k,d,l}));
                }
            }
        }
        moves.sort_by_key(|x|x.0);
        for (_,next,op) in moves {
            path.push(op);
            if self.dfs(next,depth+1,limit,path) { return true; }
            path.pop(); if self.stopped { return false; }
        }
        false
    }
}
fn compact_window(start: &BitBoard,goal: &BitBoard,region: &[usize],original_len: usize,grid: &Grid,deadline: Instant) -> Option<Vec<Op>> {
    let mut nests=[None;12]; let mut rays=[[None;12];12];
    for (i,&p) in region.iter().enumerate() {
        nests[i]=grid.nest_color_at[p];
        for (j,&q) in region.iter().enumerate() { rays[i][j]=ray(p,q,grid); }
    }
    let initial=local_state(start,region); let goal=local_state(goal,region);
    let mut search=LocalSearch{region,n:grid.n,nests,rays,goal,required:color_counts(&goal),seen:FastHashMap::default(),
        deadline,nodes:0,stopped:false};
    let mut answer=None;
    for limit in 0..=4.min(original_len-1) {
        search.seen.clear(); let mut path=Vec::new();
        if search.dfs(initial,0,limit,&mut path) {
            answer=Some(path); break;
        }
        if search.stopped {break;}
    }
    stat(|s| {s.compact_nodes+=search.nodes; if search.stopped {s.budget_stops+=1;} }); answer
}
fn closure_region(endpoints: &[(usize,usize)],a: usize,b: usize) -> (Vec<usize>,Vec<usize>) {
    let (p,q)=endpoints[a]; let mut region=vec![p,q];
    loop {
        let old=region.len();
        for &(p,q) in &endpoints[a..b] {
            if region.contains(&p)||region.contains(&q) {
                if !region.contains(&p) {region.push(p);}
                if !region.contains(&q) {region.push(q);}
            }
        }
        if old==region.len() {break;}
    }
    let chosen=(a..b).filter(|&t|region.contains(&endpoints[t].0)||region.contains(&endpoints[t].1)).collect();
    (region,chosen)
}
fn add_auxiliary(region: &mut Vec<usize>,start: &BitBoard,endpoints: &[(usize,usize)],grid: &Grid) {
    let mandatory=region.len(); let mut xs=Vec::new();
    for i in 0..mandatory { for j in (i+1)..mandatory {
        for x in bridges(region[i],region[j],grid) {
            if start.lens[x]==0 {xs.push(x);}
        }
    }}
    for &p in &region[..mandatory] {
        for d in 0..4 { if grid.max_reach[p][d]>0 {
            let (dr,dc)=DIR[d]; let x=(p/grid.n) as isize*grid.n as isize + (p%grid.n) as isize + dr*grid.n as isize+dc;
            if start.lens[x as usize]>0 {xs.push(x as usize);}
        }}
    }
    for x in xs {
        if region.len()>=12.min(mandatory+2) {break;}
        if !region.contains(&x) && !endpoints.iter().any(|&(p,q)|p==x||q==x) {region.push(x);}
    }
}
pub fn compact_optimize(mut ops: Vec<Op>,initial: &BitBoard,grid: &Grid,deadline: Instant,independent: bool) -> Vec<Op> {
    let begin=Instant::now(); let before=ops.len(); let stats_before=stats_snapshot();
    let Some(mut trace)=TraceCache::build(initial,&ops,grid,deadline) else {return ops;};
    let mut a=0;
    'search: while a+3<=ops.len() && Instant::now()<deadline {
        let mut windows: Vec<(usize,bool)>=(3..=6).filter(|&l|a+l<=ops.len()).map(|l|(a+l,false)).collect();
        if independent {
            let mut next_counts=vec![3,4,6];
            for b in (a+3)..=(a+24).min(ops.len()) {
                let (region,chosen)=closure_region(&trace.endpoints,a,b);
                if region.len()>12 || chosen.len()>6 {break;}
                if next_counts.contains(&chosen.len()) {
                    if chosen.len()<b-a {windows.push((b,true));}
                    next_counts.retain(|&c|c!=chosen.len());
                }
            }
        }
        for (b,filtered) in windows {
            if Instant::now()>=deadline {break 'search;}
            let (mut region,chosen)=if filtered {closure_region(&trace.endpoints,a,b)} else {
                let mut cells=Vec::new();
                for &(p,q) in &trace.endpoints[a..b] {for x in [p,q] {if !cells.contains(&x) {cells.push(x);}}}
                (cells,(a..b).collect::<Vec<_>>())
            };
            if region.len()>12 || !(3..=6).contains(&chosen.len()) {continue;}
            add_auxiliary(&mut region,&trace.boards[a],&trace.endpoints[a..b],grid);
            let window_deadline=deadline.min(Instant::now()+std::time::Duration::from_millis(1));
            let Some(mut patch)=compact_window(&trace.boards[a],&trace.boards[b],&region,chosen.len(),grid,window_deadline) else {continue;};
            for t in a..b {if !chosen.contains(&t) {patch.push(ops[t]);}}
            // Mandatory full interval equality check, including preserved independent moves.
            let mut check=trace.boards[a]; let mut legal=true;
            for &op in &patch {if apply_op(&mut check,op,grid).is_none() {legal=false;break;}}
            if legal && check==trace.boards[b] && Instant::now()<deadline {
                record_patch(b-a,patch.len()); ops.splice(a..b,patch);
                if !trace.rebuild_from(a,&ops,grid,deadline) {break 'search;}
                a=a.saturating_sub(1); continue 'search;
            }
        }
        a+=1;
    }
    phase_log("compact",begin,before,ops.len()); phase_stats("compact",stats_before); ops
}

pub fn finish_new_pipeline(mut ops: Vec<Op>,initial: &BitBoard,grid: &Grid,start: Instant,mode: &str) -> Vec<Op> {
    let bridge_deadline=deadline_at(start,1680).min(Instant::now()+std::time::Duration::from_millis(30));
    if Instant::now()<bridge_deadline { ops=complete_2hop_jumps(ops,initial,grid,grid.initial_slimes,bridge_deadline); }
    if mode=="local"||mode=="local-contiguous" {
        let local_deadline=deadline_at(start,1730).min(Instant::now()+std::time::Duration::from_millis(50));
        if Instant::now()<local_deadline {ops=compact_optimize(ops,initial,grid,local_deadline,mode=="local");}
    } else if start.elapsed().as_millis()+150<1730 {
        ops=lns_optimize(ops,initial,grid,start,1730);
    }
    let begin=Instant::now(); let before=ops.len();
    ops=prune_ops(ops,initial,grid,grid.initial_slimes,start,1750);
    ops=prune_cycle_and_uturn_ops(ops,initial,grid,grid.initial_slimes,start,1750);
    phase_log("final",begin,before,ops.len()); ops
}
