#[cfg(test)]
mod c02_tests {
    use super::*;
    fn budget() -> Instant {Instant::now()+std::time::Duration::from_secs(10)}
    fn fixture() -> (Grid,BitBoard) {
        let mut raw=vec![vec!['.';5];5];raw[0][2]='A';raw[4][4]='B';
        Grid::new(5,2,&raw)
    }
    fn put(board: &mut BitBoard,p: usize,colors: &[usize]) {
        board.colors[p]=0;board.lens[p]=colors.len() as u8;
        for (i,&c) in colors.iter().enumerate() {board.colors[p]|=(c as u32)<<(i*4);}
        if colors.is_empty() {board.clear_occupied(p);} else {board.set_occupied(p);}
    }
    fn enumerate_one(s: LocalState,g: LocalState,geometry: &LocalGeometry) -> bool {
        if s==g {return true;}
        for i in 0..geometry.region.len() {for j in 0..geometry.region.len() {
            let Some((_,l))=geometry.rays[i][j] else {continue;};
            for k in 0..(s.cells[i]>>32) as usize {
                if local_move(s,i,j,k,l,&geometry.nests)==Some(g) {return true;}
            }
        }}false
    }
    fn direct(s: LocalState,g: LocalState,geometry: &LocalGeometry) -> Option<Vec<Op>> {
        let mut b=SearchBudget{deadline:budget(),node_limit:2000,counters:SearchCounters::default(),stop:None};
        solve_one_move(&s,&g,geometry,&mut b)
    }
    #[test]
    fn direct_one_move_handles_one_difference_and_source_cleanup() {
        let (mut grid,mut board)=fixture();let region=[0,1,2,3];
        put(&mut board,0,&[0]);put(&mut board,1,&[1]);grid.nest_color_at[1]=Some(0);
        let geometry=LocalGeometry::new(&region,&grid);let initial=local_state(&board,&region);
        let goal=local_move(initial,0,1,0,1,&geometry.nests).unwrap();
        assert_eq!(diff_count(&initial,&goal),1);
        assert_eq!(direct(initial,goal,&geometry).unwrap().len(),1);
        put(&mut board,0,&[0,0,1]);put(&mut board,1,&[]);
        grid.nest_color_at[0]=Some(0);grid.nest_color_at[3]=Some(1);
        let geometry=LocalGeometry::new(&region,&grid);let initial=local_state(&board,&region);
        let goal=local_move(initial,0,3,2,3,&geometry.nests).unwrap();
        assert_eq!(goal.cells[0]>>32,0);
        let path=direct(initial,goal,&geometry).unwrap();assert_eq!(path[0].k,2);
    }
    #[test]
    fn fast_transition_diff_and_counts_match_reference() {
        let (mut grid,mut board)=fixture();grid.nest_color_at[0]=Some(0);grid.nest_color_at[1]=Some(1);
        let region=[0,1,2,3,4,5,6,7,8,9,10,11];let geometry=LocalGeometry::new(&region,&grid);
        let mut rng=5511779933u64;let mut transitions=0;
        for _ in 0..150 {
            for &p in &region {
                rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;let h=(rng%9) as usize;
                let colors:Vec<_>=(0..h).map(|i|((rng>>(i*4))&1) as usize).collect();put(&mut board,p,&colors);
            }
            let initial=local_state(&board,&region);let mut goal=initial;
            goal.cells[(rng as usize)%region.len()]=0;
            let counts=color_counts(&initial);let diff=diff_count(&initial,&goal);
            for i in 0..region.len() {for j in 0..region.len() {
                let Some((d,l))=geometry.rays[i][j] else {continue;};
                for k in 0..=8 {
                    let expected=local_move(initial,i,j,k,l,&geometry.nests);
                    let actual=fast_transition(&initial,i,j,k,d,l,&geometry.nests);
                    assert_eq!(expected.is_some(),actual.is_some());
                    if let Some(candidate)=actual {
                        let next=apply_candidate(initial,&candidate);assert_eq!(Some(next),expected);
                        assert_eq!(candidate_diff(&initial,&goal,diff,&candidate),diff_count(&next,&goal));
                        assert_eq!(counts_after(counts,&candidate),color_counts(&next));
                    }
                    transitions+=1;
                }
            }}
        }
        assert!(transitions>10000);
    }
    #[test]
    fn direct_solver_matches_exhaustive_one_move() {
        let (grid,mut board)=fixture();let region=[0,1,2,5,6,7];let geometry=LocalGeometry::new(&region,&grid);
        for config in 0..100usize {
            for (i,&p) in region.iter().enumerate() {
                let colors:Vec<_>=(0..(config+i)%4).map(|k|(config+i+k)%2).collect();put(&mut board,p,&colors);
            }
            let initial=local_state(&board,&region);
            for i in 0..region.len() {for j in 0..region.len() {
                let Some((_,l))=geometry.rays[i][j] else {continue;};
                for k in 0..(initial.cells[i]>>32) as usize {
                    let Some(goal)=local_move(initial,i,j,k,l,&geometry.nests) else {continue;};
                    assert!(enumerate_one(initial,goal,&geometry));assert!(direct(initial,goal,&geometry).is_some());
                    let mut changed=goal;changed.cells[(i+2)%region.len()]=0;
                    assert_eq!(direct(initial,changed,&geometry).is_some(),enumerate_one(initial,changed,&geometry));
                }
            }}
        }
    }
    #[test]
    fn fast_search_reuses_buffers_and_matches_reference_reachability() {
        let (grid,mut board)=fixture();put(&mut board,0,&[0]);
        let region=[0,1,2,5,6,7];let geometry=LocalGeometry::new(&region,&grid);
        let initial=local_state(&board,&region);let mut goal=initial;goal.cells[0]=0;
        let (outcome,_)=search_fast(initial,goal,&geometry,4,None,budget(),2000);
        let SearchOutcome::Found(path)=outcome else {panic!("two-step route must exist");};
        assert_eq!(path.len(),2);
        let mut full=board;for op in path {apply_op(&mut full,op,&grid).unwrap();}
        assert_eq!(local_state(&full,&region),goal);
        let (outcome,_)=search_fast(initial,goal,&geometry,1,None,budget(),2000);
        assert_eq!(outcome,SearchOutcome::Exhausted{max_depth:1});
        // Branch ordering and child recursion must not overwrite the parent buffer.
        let expected=compact_window(&board,&full,&region,5,&grid,budget()).unwrap();assert_eq!(expected.len(),2);
    }
    #[test]
    fn cache_respects_region_depth_and_partial_proofs() {
        let (grid,mut board)=fixture();put(&mut board,0,&[0]);let mut goal=board;put(&mut goal,0,&[]);
        let (key,region)=WindowKey::new(vec![2,0],&board,&goal);
        let (same,_) = WindowKey::new(vec![0,2],&board,&goal);assert!(key==same);
        let (different,_) = WindowKey::new(vec![0,1,2],&board,&goal);assert!(key!=different);
        let geometry=LocalGeometry::new(&region,&grid);
        let (outcome,_) = search_fast(key.start,key.goal,&geometry,4,None,budget(),0);
        let SearchOutcome::NodeLimit{completed_depth}=outcome else {panic!("node limit expected");};
        assert_eq!(completed_depth,Some(0));
        let mut entry=WindowEntry::default();update_entry(&mut entry,&outcome,7,4);
        assert_eq!(entry.exhausted_depth,Some(0));assert_eq!(entry.transient,Some((7,4,2000,1000)));
        let (timed,_) = search_fast(key.start,key.goal,&geometry,4,None,Instant::now(),2000);
        assert_eq!(timed,SearchOutcome::TimeLimit{completed_depth:None});
        update_entry(&mut entry,&SearchOutcome::Exhausted{max_depth:1},8,1);
        assert_eq!(entry.exhausted_depth,Some(1));assert!(!entry.exhausted_depth.is_some_and(|d|d>=4));
    }
    #[test]
    fn tail_timeout_off_and_completed_session_preserve_checkpoint() {
        let (mut grid,mut initial)=fixture();put(&mut initial,0,&[0]);grid.initial_slimes=1;
        let route=vec![Op{r:0,c:0,k:0,d:1,l:1},Op{r:1,c:0,k:0,d:3,l:1},
            Op{r:1,c:1,k:0,d:3,l:1},Op{r:1,c:2,k:0,d:0,l:1}];
        assert!(valid_complete(&initial,&route,&grid));
        for mode in [TailMode::Off,TailMode::Legacy,TailMode::CompactFast] {
            assert_eq!(tail_optimize_mode(route.clone(),&initial,&grid,Instant::now(),mode),route);
            let result=tail_optimize_mode(route.clone(),&initial,&grid,budget(),mode);
            assert!(result.len()<=route.len());assert!(valid_complete(&initial,&result,&grid));
        }
    }
    #[test]
    fn session_edit_invalidates_queue_and_keeps_trace_current() {
        let (mut grid,mut initial)=fixture();put(&mut initial,0,&[0]);put(&mut initial,20,&[1]);grid.initial_slimes=2;
        let mut route=vec![Op{r:0,c:0,k:0,d:1,l:1},Op{r:1,c:0,k:0,d:3,l:1},
            Op{r:1,c:1,k:0,d:3,l:1},Op{r:1,c:2,k:0,d:0,l:1}];
        for c in 0..4 {route.push(Op{r:4,c,k:0,d:3,l:1});}
        let trace=TraceCache::build(&initial,&route,&grid,budget()).unwrap();
        let mut session=TailSession{ops:route,trace,cache:FastHashMap::default(),reverse:true,cursor:5,version:0,stats:TailStats::default()};
        let edited=session.patch_window(0,4,false,TailMode::CompactFast,&grid,budget()).unwrap();
        assert_eq!(session.version,1);assert_eq!(session.ops.len(),6);
        assert!(session.trace.rebuild_from(edited,&session.ops,&grid,budget()));
        assert_eq!(session.trace.boards.len(),session.ops.len()+1);
        assert_eq!(session.trace.endpoints.len(),session.ops.len());assert!(valid_complete(&initial,&session.ops,&grid));
        for (b,_) in tail_windows(&session.trace,0,session.ops.len()) {assert!(b<=session.ops.len());}
    }
}
