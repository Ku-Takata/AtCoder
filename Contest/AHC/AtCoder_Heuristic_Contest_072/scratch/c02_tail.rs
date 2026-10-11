// C_02: tail-only search. C_01's candidate pipeline and local physics stay intact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TailMode { Off, Legacy, CompactFast }
fn tail_mode() -> TailMode {
    let value = std::env::args().find_map(|s| s.strip_prefix("--tail=").map(str::to_owned))
        .unwrap_or_else(|| "compact-fast".to_owned());
    match value.as_str() {
        "off" => TailMode::Off, "legacy" => TailMode::Legacy,
        "compact-fast" => TailMode::CompactFast,
        _ => panic!("unsupported --tail value: {} (off, legacy, compact-fast)", value),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SearchOutcome {
    Found(Vec<Op>),
    Exhausted { max_depth: usize },
    NodeLimit { completed_depth: Option<usize> },
    TimeLimit { completed_depth: Option<usize> },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchStop { Nodes, Time }
#[derive(Default, Clone, Copy)]
struct SearchCounters { generated: usize, calls: usize, expanded: usize, terminal_checks: usize }
struct SearchBudget {
    deadline: Instant, node_limit: usize, counters: SearchCounters, stop: Option<SearchStop>,
}
impl SearchBudget {
    fn check_time(&mut self) -> bool {
        if Instant::now() >= self.deadline { self.stop = Some(SearchStop::Time); false } else { true }
    }
    fn enter(&mut self) -> bool {
        if self.counters.calls >= self.node_limit { self.stop = Some(SearchStop::Nodes); return false; }
        if self.counters.calls % 32 == 0 && !self.check_time() { return false; }
        self.counters.calls += 1; true
    }
    fn generated(&mut self) -> bool {
        self.counters.generated += 1;
        self.counters.generated % 32 != 0 || self.check_time()
    }
}
struct LocalGeometry {
    region: Vec<usize>, n: usize, nests: [Option<usize>;12],
    rays: [[Option<(usize,usize)>;12];12],
}
impl LocalGeometry {
    fn new(region: &[usize],grid: &Grid) -> Self {
        let mut geometry = Self {region:region.to_vec(),n:grid.n,nests:[None;12],rays:[[None;12];12]};
        for (i,&p) in region.iter().enumerate() {
            geometry.nests[i] = grid.nest_color_at[p];
            for (j,&q) in region.iter().enumerate() { geometry.rays[i][j] = ray(p,q,grid); }
        }
        geometry
    }
    fn op(&self,i: usize,k: usize,d: usize,l: usize) -> Op {
        let p=self.region[i]; Op {r:p/self.n,c:p%self.n,k,d,l}
    }
}
#[derive(Clone, Copy, Debug)]
struct LocalCandidate {
    src: u8, dst: u8, k: u8, d: u8, l: u8, next_diff: u8,
    next_src: u64, next_dst: u64, removed: u64, removed_len: u8,
}
fn fast_clean(cell: &mut u64,nest: Option<usize>,removed: &mut u64,removed_len: &mut u8) {
    let Some(c)=nest else {return;};
    let mut h=(*cell>>32) as usize; let mut bits=*cell as u32;
    while h>0 && ((bits>>((h-1)*4))&15) as usize==c {
        *removed |= (c as u64)<<(*removed_len as usize*4); *removed_len+=1;
        h-=1; bits &= if h==0 {0} else {(1u32<<(h*4))-1};
    }
    *cell=bits as u64|((h as u64)<<32);
}
fn fast_transition(s: &LocalState,i: usize,j: usize,k: usize,d: usize,l: usize,
    nests: &[Option<usize>;12]) -> Option<LocalCandidate> {
    let h=(s.cells[i]>>32) as usize; let g=(s.cells[j]>>32) as usize;
    if i==j || k>=h || l==0 || l>k+1 || g+h-k>8 {return None;}
    let mut bits=(s.cells[i] as u32)>>(k*4); let mut rev=0u32;
    for _ in k..h {rev=(rev<<4)|(bits&15);bits>>=4;}
    let mask=if k==0 {0} else {(1u32<<(k*4))-1};
    let mut src=(s.cells[i] as u32&mask) as u64|((k as u64)<<32);
    let mut dst=((s.cells[j] as u32)|(rev<<(g*4))) as u64|(((g+h-k) as u64)<<32);
    let mut removed=0; let mut removed_len=0;
    fast_clean(&mut src,nests[i],&mut removed,&mut removed_len);
    fast_clean(&mut dst,nests[j],&mut removed,&mut removed_len);
    Some(LocalCandidate {src:i as u8,dst:j as u8,k:k as u8,d:d as u8,l:l as u8,
        next_src:src,next_dst:dst,next_diff:0,removed,removed_len})
}
fn counts_after(mut counts: [u8;16],candidate: &LocalCandidate) -> [u8;16] {
    let mut bits=candidate.removed;
    for _ in 0..candidate.removed_len {counts[(bits&15) as usize]-=1;bits>>=4;}
    counts
}
fn apply_candidate(mut state: LocalState,candidate: &LocalCandidate) -> LocalState {
    state.cells[candidate.src as usize]=candidate.next_src;
    state.cells[candidate.dst as usize]=candidate.next_dst; state
}
fn diff_count(s: &LocalState,g: &LocalState) -> usize {
    s.cells.iter().zip(g.cells.iter()).filter(|(a,b)|a!=b).count()
}
fn candidate_diff(s: &LocalState,g: &LocalState,diff: usize,c: &LocalCandidate) -> usize {
    let (i,j)=(c.src as usize,c.dst as usize);
    diff - usize::from(s.cells[i]!=g.cells[i]) - usize::from(s.cells[j]!=g.cells[j])
        + usize::from(c.next_src!=g.cells[i]) + usize::from(c.next_dst!=g.cells[j])
}
// Exact one-move check: k is enumerated even when source-nest cleanup lowers
// its final height below k. With one mismatch, the unchanged destination is a nest.
fn solve_one_move(s: &LocalState,goal: &LocalState,geometry: &LocalGeometry,budget: &mut SearchBudget) -> Option<Vec<Op>> {
    budget.counters.terminal_checks+=1;
    if !budget.check_time() {return None;}
    let mut differences=[0;2]; let mut mismatch_count=0;
    for i in 0..geometry.region.len() {
        if s.cells[i]!=goal.cells[i] {
            if mismatch_count==2 {return None;}
            differences[mismatch_count]=i; mismatch_count+=1;
        }
    }
    if mismatch_count==0 {return Some(Vec::new());}
    for &i in &differences[..mismatch_count] {
        let h=(s.cells[i]>>32) as usize;
        if h <= (goal.cells[i]>>32) as usize {continue;}
        for j in 0..geometry.region.len() {
            if j==i {continue;}
            if mismatch_count==2 && !differences.contains(&j) {continue;}
            if mismatch_count==1 && geometry.nests[j].is_none() {continue;}
            let Some((d,l))=geometry.rays[i][j] else {continue;};
            for k in (l-1)..h {
                if !budget.generated() {return None;}
                let Some(c)=fast_transition(s,i,j,k,d,l,&geometry.nests) else {continue;};
                if c.next_src==goal.cells[i] && c.next_dst==goal.cells[j] {
                    return Some(vec![geometry.op(i,k,d,l)]);
                }
            }
        }
    }
    None
}
struct CompactSearchFast<'a> {
    geometry: &'a LocalGeometry, goal: LocalState, required: [u8;16], budget: SearchBudget,
    seen: FastHashMap<LocalState,usize>, buffers: [Vec<LocalCandidate>;4],
}
impl CompactSearchFast<'_> {
    fn dfs(&mut self,s: LocalState,counts: [u8;16],diff: usize,depth: usize,limit: usize,path: &mut Vec<Op>) -> bool {
        if !self.budget.enter() {return false;}
        if diff==0 {return true;}
        let remaining=limit-depth;
        if remaining==0 || diff>2*remaining || (0..16).any(|c|counts[c]<self.required[c]) {return false;}
        if remaining==1 {
            if let Some(last)=solve_one_move(&s,&self.goal,self.geometry,&mut self.budget) {path.extend(last);return true;}
            return false;
        }
        if self.seen.get(&s).is_some_and(|&d|d<=depth) {return false;}
        self.seen.insert(s,depth); self.budget.counters.expanded+=1;
        self.buffers[depth].clear();
        for i in 0..self.geometry.region.len() {
            let h=(s.cells[i]>>32) as usize;
            for j in 0..self.geometry.region.len() {
                let Some((d,l))=self.geometry.rays[i][j] else {continue;};
                for k in (l-1)..h {
                    if !self.budget.generated() {return false;}
                    let Some(mut candidate)=fast_transition(&s,i,j,k,d,l,&self.geometry.nests) else {continue;};
                    let next_diff=candidate_diff(&s,&self.goal,diff,&candidate);
                    if next_diff>2*(remaining-1) {continue;}
                    if next_diff==0 {path.push(self.geometry.op(i,k,d,l));return true;}
                    if candidate.next_src==s.cells[i] && candidate.next_dst==s.cells[j] {continue;}
                    if candidate.removed_len>0 {
                        let next_counts=counts_after(counts,&candidate);
                        if (0..16).any(|c|next_counts[c]<self.required[c]) {continue;}
                    }
                    candidate.next_diff=next_diff as u8;
                    self.buffers[depth].push(candidate);
                }
            }
        }
        self.buffers[depth].sort_by_key(|c|c.next_diff);
        for index in 0..self.buffers[depth].len() {
            let candidate=self.buffers[depth][index];
            let next=apply_candidate(s,&candidate); let next_counts=counts_after(counts,&candidate);
            path.push(self.geometry.op(candidate.src as usize,candidate.k as usize,candidate.d as usize,candidate.l as usize));
            if self.dfs(next,next_counts,candidate.next_diff as usize,depth+1,limit,path) {return true;}
            path.pop(); if self.budget.stop.is_some() {return false;}
        }
        false
    }
}
fn search_fast(initial: LocalState,goal: LocalState,geometry: &LocalGeometry,max_depth: usize,
    proven: Option<usize>,deadline: Instant,node_limit: usize) -> (SearchOutcome,SearchCounters) {
    let required=color_counts(&goal); let initial_counts=color_counts(&initial);
    let diff=diff_count(&initial,&goal);
    let mut search=CompactSearchFast {geometry,goal,required,seen:FastHashMap::default(),
        buffers:std::array::from_fn(|_|Vec::with_capacity(128)),
        budget:SearchBudget{deadline,node_limit,counters:SearchCounters::default(),stop:None}};
    if !search.budget.check_time() {return (SearchOutcome::TimeLimit{completed_depth:proven},search.budget.counters);}
    if diff==0 {return (SearchOutcome::Found(Vec::new()),search.budget.counters);}
    if (0..16).any(|c|initial_counts[c]<required[c]) {return (SearchOutcome::Exhausted{max_depth},search.budget.counters);}
    let lower=diff.div_ceil(2);
    let mut completed=Some(proven.unwrap_or(0).max(lower-1));
    let first=lower.max(proven.map_or(0,|d|d+1));
    for limit in first..=max_depth {
        search.seen.clear(); let mut path=Vec::with_capacity(4);
        if search.dfs(initial,initial_counts,diff,0,limit,&mut path) {
            if search.budget.check_time() {return (SearchOutcome::Found(path),search.budget.counters);}
        }
        if let Some(stop)=search.budget.stop {
            let result=match stop {SearchStop::Nodes=>SearchOutcome::NodeLimit{completed_depth:completed},
                SearchStop::Time=>SearchOutcome::TimeLimit{completed_depth:completed}};
            return (result,search.budget.counters);
        }
        completed=Some(limit);
    }
    (SearchOutcome::Exhausted{max_depth},search.budget.counters)
}
fn search_legacy(start: &BitBoard,goal: &BitBoard,region: &[usize],max_depth: usize,grid: &Grid,deadline: Instant) -> (SearchOutcome,SearchCounters) {
    let geometry=LocalGeometry::new(region,grid);
    let initial=local_state(start,region);let goal=local_state(goal,region);
    let mut search=LocalSearch{region,n:grid.n,nests:geometry.nests,rays:geometry.rays,goal,
        required:color_counts(&goal),seen:FastHashMap::default(),deadline,nodes:0,stopped:false};
    let mut completed=None;
    for limit in 0..=max_depth {
        if Instant::now()>=deadline {return (SearchOutcome::TimeLimit{completed_depth:completed},SearchCounters{calls:search.nodes,..SearchCounters::default()});}
        search.seen.clear();let mut path=Vec::new();
        if search.dfs(initial,0,limit,&mut path) {
            if Instant::now()<deadline {return (SearchOutcome::Found(path),SearchCounters{calls:search.nodes,..SearchCounters::default()});}
            return (SearchOutcome::TimeLimit{completed_depth:completed},SearchCounters{calls:search.nodes,..SearchCounters::default()});
        }
        if search.stopped {
            let outcome=if search.nodes>=2000 {SearchOutcome::NodeLimit{completed_depth:completed}}
                else {SearchOutcome::TimeLimit{completed_depth:completed}};
            return (outcome,SearchCounters{calls:search.nodes,..SearchCounters::default()});
        }
        completed=Some(limit);
    }
    (SearchOutcome::Exhausted{max_depth},SearchCounters{calls:search.nodes,..SearchCounters::default()})
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct WindowKey {
    region: [u16;12], len: u8, rules: u8, start: LocalState, goal: LocalState,
}
impl WindowKey {
    fn new(mut region: Vec<usize>,start: &BitBoard,goal: &BitBoard) -> (Self,Vec<usize>) {
        region.sort_unstable(); region.dedup();
        let mut coords=[0;12]; for (i,&p) in region.iter().enumerate() {coords[i]=p as u16;}
        (Self{region:coords,len:region.len() as u8,rules:0,start:local_state(start,&region),goal:local_state(goal,&region)},region)
    }
}
#[derive(Default)]
struct WindowEntry {
    found: Option<Vec<Op>>, exhausted_depth: Option<usize>,
    // This is an attempt marker, never a proof of unreachability.
    transient: Option<(usize,usize,usize,u64)>,
}
#[derive(Default)]
struct TailStats {
    windows: usize, generated: usize, calls: usize, expanded: usize, terminal_checks: usize,
    found: usize, exhausted: usize, node_limit: usize, time_limit: usize,
    hits_found: usize, hits_exhausted: usize, transient_skips: usize, cache_ns: u128,
    first_half: usize, second_half: usize, reverse_windows: usize, forward_windows: usize,
    accepted: usize, local_saved: usize, cleanup_saved: usize,
}
impl TailStats {
    fn search_result(&mut self,outcome: &SearchOutcome,counters: SearchCounters) {
        self.generated+=counters.generated;self.calls+=counters.calls;self.expanded+=counters.expanded;
        self.terminal_checks+=counters.terminal_checks;
        match outcome {SearchOutcome::Found(_)=>self.found+=1,SearchOutcome::Exhausted{..}=>self.exhausted+=1,
            SearchOutcome::NodeLimit{..}=>self.node_limit+=1,SearchOutcome::TimeLimit{..}=>self.time_limit+=1};
    }
}
fn update_entry(entry: &mut WindowEntry,outcome: &SearchOutcome,version: usize,depth: usize) {
    let completed=match outcome {
        SearchOutcome::Found(path)=>{
            if entry.found.as_ref().is_none_or(|p|path.len()<p.len()) {entry.found=Some(path.clone());}
            path.len().checked_sub(1)
        }
        SearchOutcome::Exhausted{max_depth}=>Some(*max_depth),
        SearchOutcome::NodeLimit{completed_depth}|SearchOutcome::TimeLimit{completed_depth}=>{
            entry.transient=Some((version,depth,2000,1000)); *completed_depth
        }
    };
    if let Some(d)=completed {entry.exhausted_depth=Some(entry.exhausted_depth.map_or(d,|old|old.max(d)));}
}
fn tail_windows(trace: &TraceCache,a: usize,len: usize) -> Vec<(usize,bool)> {
    let mut windows: Vec<_>=(3..=6).filter(|&l|a+l<=len).map(|l|(a+l,false)).collect();
    let mut requested=vec![3,4,6];
    for b in a+3..=(a+24).min(len) {
        let (r,chosen)=closure_region(&trace.endpoints,a,b);
        if r.len()>12 || chosen.len()>6 {break;}
        if requested.contains(&chosen.len()) {
            if chosen.len()<b-a {windows.push((b,true));}
            requested.retain(|&c|c!=chosen.len());
        }
    }
    windows
}
fn window_region(trace: &TraceCache,a: usize,b: usize,filtered: bool,grid: &Grid) -> Option<(Vec<usize>,Vec<usize>)> {
    let (mut region,chosen)=if filtered {closure_region(&trace.endpoints,a,b)} else {
        let mut cells=Vec::new();
        for &(p,q) in &trace.endpoints[a..b] {for x in [p,q] {if !cells.contains(&x) {cells.push(x);}}}
        (cells,(a..b).collect::<Vec<_>>())
    };
    if region.len()>12 || !(3..=6).contains(&chosen.len()) {return None;}
    add_auxiliary(&mut region,&trace.boards[a],&trace.endpoints[a..b],grid);
    Some((region,chosen))
}

// Small exact, independently checked cleanup blocks. They never consume the
// entire tail budget before the reverse compact scan gets a chance to run.
fn tail_cleanup(mut ops: Vec<Op>,initial: &BitBoard,grid: &Grid,deadline: Instant,geometry: bool,name: &str) -> Vec<Op> {
    let begin=Instant::now(); let before=ops.len(); let snapshot=stats_snapshot();
    let ms=deadline.saturating_duration_since(begin).as_millis();
    type Opt=fn(Vec<Op>,&BitBoard,&Grid,usize,Instant,u128)->Vec<Op>;
    if geometry {
        for opt in [merge_straight_jumps as Opt,shortcut_ray_jumps,orthogonal_2hop_jumps,coriding_leap_jumps,crank_3hop_jumps] {
            if Instant::now()>=deadline {break;}
            ops=opt(ops,initial,grid,grid.initial_slimes,begin,ms);
        }
    }
    if Instant::now()<deadline {ops=complete_2hop_jumps(ops,initial,grid,grid.initial_slimes,deadline);}
    if Instant::now()<deadline {ops=prune_ops(ops,initial,grid,grid.initial_slimes,begin,ms);}
    if Instant::now()<deadline {ops=prune_cycle_and_uturn_ops(ops,initial,grid,grid.initial_slimes,begin,ms);}
    let after=stats_snapshot();
    eprintln!("TAIL_PHASE: name={} trials={} accepted={} saved_ops={} elapsed_ms={:.3}",name,
        after.trials-snapshot.trials,after.accepted-snapshot.accepted,before-ops.len(),begin.elapsed().as_secs_f64()*1000.0);
    ops
}
struct TailSession {
    ops: Vec<Op>, trace: TraceCache, cache: FastHashMap<WindowKey,WindowEntry>,
    reverse: bool, cursor: usize, version: usize, stats: TailStats,
}
impl TailSession {
    fn patch_window(&mut self,a: usize,b: usize,filtered: bool,mode: TailMode,grid: &Grid,deadline: Instant) -> Option<usize> {
        let (region,chosen)=window_region(&self.trace,a,b,filtered,grid)?;
        self.stats.windows+=1;
        if a*2<self.ops.len() {self.stats.first_half+=1;} else {self.stats.second_half+=1;}
        if self.reverse {self.stats.reverse_windows+=1;} else {self.stats.forward_windows+=1;}
        let (key,region)=WindowKey::new(region,&self.trace.boards[a],&self.trace.boards[b]);
        let depth=4.min(chosen.len()-1); let mut result=None;
        let cache_start=Instant::now();
        let mut proven=None;
        if let Some(entry)=self.cache.get(&key) {
            if let Some(path)=&entry.found {
                if path.len()<=depth && path.len()<chosen.len() {self.stats.hits_found+=1;result=Some(path.clone());}
            }
            proven=entry.exhausted_depth;
            if result.is_none() && proven.is_some_and(|d|d>=depth) {self.stats.hits_exhausted+=1;self.stats.cache_ns+=cache_start.elapsed().as_nanos();return None;}
            if result.is_none() && entry.transient==Some((self.version,depth,2000,1000)) {self.stats.transient_skips+=1;self.stats.cache_ns+=cache_start.elapsed().as_nanos();return None;}
        }
        self.stats.cache_ns+=cache_start.elapsed().as_nanos();
        if result.is_none() {
            let local_deadline=deadline.min(Instant::now()+std::time::Duration::from_millis(1));
            let outcome=if mode==TailMode::Legacy {
                let (outcome,counters)=search_legacy(&self.trace.boards[a],&self.trace.boards[b],&region,depth,grid,local_deadline);
                self.stats.search_result(&outcome,counters);outcome
            } else {
                let geometry=LocalGeometry::new(&region,grid);
                let (outcome,counters)=search_fast(key.start,key.goal,&geometry,depth,proven,local_deadline,2000);
                self.stats.search_result(&outcome,counters);outcome
            };
            let cache_start=Instant::now();
            if self.cache.contains_key(&key)||self.cache.len()<4096 {
                let entry=self.cache.entry(key).or_default();
                update_entry(entry,&outcome,self.version,depth);
            }
            self.stats.cache_ns+=cache_start.elapsed().as_nanos();
            if let SearchOutcome::Found(path)=outcome {result=Some(path);}
        }
        let mut patch=result?;
        if patch.len()>=chosen.len() {return None;}
        for t in a..b {if !chosen.contains(&t) {patch.push(self.ops[t]);}}
        let mut board=self.trace.boards[a];
        for (t,&op) in patch.iter().enumerate() {
            if t%32==0 && Instant::now()>=deadline {return None;}
            apply_op(&mut board,op,grid)?;
        }
        if board!=self.trace.boards[b] || Instant::now()>=deadline {return None;}
        let saved=b-a-patch.len();
        self.ops.splice(a..b,patch);self.version+=1;self.stats.accepted+=1;self.stats.local_saved+=saved;
        Some(a)
    }
    fn scan(&mut self,mode: TailMode,grid: &Grid,initial: &BitBoard,deadline: Instant) -> &'static str {
        let mut pass_improved=false; let mut quiet_passes=0;
        loop {
            if Instant::now()>=deadline {return "deadline";}
            if self.ops.len()<3 {return "no_windows";}
            let max_a=self.ops.len()-3;
            self.cursor=self.cursor.min(max_a);
            let a=self.cursor;
            let windows=tail_windows(&self.trace,a,self.ops.len());
            // The queue is local to this version. Any accepted edit discards it.
            let mut edited=None;
            for (b,filtered) in windows {
                if Instant::now()>=deadline {return "deadline";}
                if let Some(a)=self.patch_window(a,b,filtered,mode,grid,deadline) {edited=Some(a);break;}
            }
            if let Some(changed)=edited {
                pass_improved=true;
                if !self.trace.rebuild_from(changed,&self.ops,grid,deadline) {return "trace_budget";}
                let before=self.ops.len();
                let cleanup_deadline=deadline.min(Instant::now()+std::time::Duration::from_millis(2));
                self.ops=tail_cleanup(std::mem::take(&mut self.ops),initial,grid,cleanup_deadline,false,"after_local");
                if self.ops.len()<before {
                    self.stats.cleanup_saved+=before-self.ops.len();self.version+=1;
                    let Some(trace)=TraceCache::build(initial,&self.ops,grid,deadline) else {return "trace_budget";};
                    self.trace=trace;
                }
                if self.ops.len()<3 {return "no_windows";}
                self.cursor=if self.reverse {(changed+3).min(self.ops.len()-3)} else {changed.saturating_sub(3)};
                continue;
            }
            let finished=if self.reverse {a==0} else {a==self.ops.len()-3};
            if finished {
                quiet_passes=if pass_improved {0} else {quiet_passes+1};
                if quiet_passes>=2 {return "no_new_work";}
                pass_improved=false;self.reverse=!self.reverse;
                self.cursor=if self.reverse {self.ops.len()-3} else {0};
            } else if self.reverse {self.cursor-=1;} else {self.cursor+=1;}
        }
    }
}
fn print_tail_stats(stats: &TailStats,entries: usize,elapsed: f64) {
    eprintln!("TAIL_PHASE: name=local trials={} accepted={} saved_ops={} elapsed_ms={:.3}",stats.windows,stats.accepted,stats.local_saved,elapsed);
    eprintln!("LOCAL_SEARCH: windows={} generated={} calls={} expanded={} terminal_checks={} first_half={} second_half={} reverse_windows={} forward_windows={}",
        stats.windows,stats.generated,stats.calls,stats.expanded,stats.terminal_checks,stats.first_half,stats.second_half,stats.reverse_windows,stats.forward_windows);
    eprintln!("LOCAL_RESULT: found={} exhausted={} node_limit={} time_limit={}",stats.found,stats.exhausted,stats.node_limit,stats.time_limit);
    eprintln!("WINDOW_CACHE: hits_found={} hits_exhausted={} transient_skips={} entries={} elapsed_ms={:.3}",
        stats.hits_found,stats.hits_exhausted,stats.transient_skips,entries,stats.cache_ns as f64/1e6);
    eprintln!("REV_SEARCH: eligible=0 attempted=0 found=0 saved_ops=0 elapsed_ms=0 enabled=0");
}
pub fn tail_optimize(checkpoint: Vec<Op>,initial: &BitBoard,grid: &Grid,deadline: Instant) -> Vec<Op> {
    tail_optimize_mode(checkpoint,initial,grid,deadline,tail_mode())
}
fn tail_optimize_mode(checkpoint: Vec<Op>,initial: &BitBoard,grid: &Grid,deadline: Instant,mode: TailMode) -> Vec<Op> {
    let begin=Instant::now();let before=checkpoint.len();
    let remaining=deadline.saturating_duration_since(begin);
    if mode==TailMode::Off || remaining<std::time::Duration::from_millis(5) {
        eprintln!("TAIL: before={} after={} elapsed_ms=0 stop_reason={}",before,before,if mode==TailMode::Off {"off"} else {"insufficient_time"});
        print_tail_stats(&TailStats::default(),0,0.0);return checkpoint;
    }
    let first_budget=(remaining/5).min(std::time::Duration::from_millis(15));
    let mut ops=tail_cleanup(checkpoint.clone(),initial,grid,begin+first_budget,true,"initial_geometry");
    if ops.len()>before || !valid_complete(initial,&ops,grid) {ops=checkpoint.clone();}
    let mut reason="deadline";let mut stats=TailStats::default();let mut entries=0;
    if let Some(trace)=TraceCache::build(initial,&ops,grid,deadline) {
        let cursor=ops.len().saturating_sub(3);
        let mut session=TailSession {ops,trace,cache:FastHashMap::default(),reverse:true,cursor,version:0,stats:TailStats::default()};
        let local_begin=Instant::now();
        reason=session.scan(mode,grid,initial,deadline);
        let local_elapsed=local_begin.elapsed().as_secs_f64()*1000.0;
        print_tail_stats(&session.stats,session.cache.len(),local_elapsed);
        stats=session.stats;entries=session.cache.len();ops=session.ops;
    } else {print_tail_stats(&stats,entries,0.0);}
    // One final full replay protects the checkpoint even if a future tail
    // implementation violates an internal invariant. Reserve 1745..1750ms.
    if ops.len()>before || !valid_complete(initial,&ops,grid) {ops=checkpoint;reason="checkpoint_fallback";}
    debug_assert!(ops.len()<=before);
    let _=(stats,entries);
    eprintln!("TAIL: before={} after={} elapsed_ms={:.3} stop_reason={}",before,ops.len(),begin.elapsed().as_secs_f64()*1000.0,reason);
    ops
}
