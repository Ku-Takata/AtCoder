#[cfg(test)]
mod c01_tests {
    use super::*;
    fn fixture(n:usize) -> (Grid,BitBoard) {
        let mut raw=vec![vec!['.';n];n]; raw[0][2]='A'; raw[n-1][n-1]='B';
        Grid::new(n,2,&raw)
    }
    fn put(board:&mut BitBoard,p:usize,colors:&[usize]) {
        board.colors[p]=0; board.lens[p]=colors.len() as u8;
        for (i,&c) in colors.iter().enumerate() {board.colors[p]|=(c as u32)<<(i*4);}
        if colors.is_empty() {board.clear_occupied(p);} else {board.set_occupied(p);}
    }
    fn long_route() -> Vec<Op> {
        vec![Op{r:0,c:0,k:0,d:1,l:1},Op{r:1,c:0,k:0,d:3,l:1},
            Op{r:1,c:1,k:0,d:3,l:1},Op{r:1,c:2,k:0,d:0,l:1}]
    }
    fn budget() -> Instant {Instant::now()+std::time::Duration::from_secs(5)}
    #[test]
    fn capacity_and_both_nests() {
        let (mut grid,mut board)=fixture(5);
        put(&mut board,0,&[0,0,1]); grid.nest_color_at[0]=Some(0); grid.nest_color_at[3]=Some(1);
        assert_eq!(apply_op(&mut board,Op{r:0,c:0,k:2,d:3,l:3},&grid),Some(3));
        assert!(board.lens.iter().all(|&h|h==0));
        put(&mut board,0,&[1,0]); put(&mut board,1,&[1,1,1,1,1,1]);
        assert_eq!(apply_op(&mut board,Op{r:0,c:0,k:0,d:3,l:1},&grid),Some(0));
        assert_eq!(board.lens[1],8);
        put(&mut board,0,&[1]); grid.nest_color_at[1]=Some(1);
        let original=board;
        assert_eq!(apply_op(&mut board,Op{r:0,c:0,k:0,d:3,l:1},&grid),None);
        assert!(board==original);
    }
    #[test]
    fn compact_transition_matches_reference() {
        let (grid,mut board)=fixture(5); let mut rng=887766554433u64;
        let region=[0,1,2,3,4,5,6,7,8,9,10,11];
        let mut nests=[None;12]; for i in 0..12 {nests[i]=grid.nest_color_at[region[i]];}
        let mut samples=0;
        for _ in 0..200 {
            for &p in &region {
                rng^=rng<<13; rng^=rng>>7; rng^=rng<<17;
                let h=(rng%9) as usize;
                let colors:Vec<usize>=(0..h).map(|i|((rng>>(i*4))&1) as usize).collect();
                put(&mut board,p,&colors);
            }
            let s=local_state(&board,&region);
            for i in 0..12 {for j in 0..12 {
                let Some((d,l))=ray(region[i],region[j],&grid) else {continue;};
                for k in 0..=8 {
                    let mut full=board;
                    let result=apply_op(&mut full,Op{r:region[i]/5,c:region[i]%5,k,d,l},&grid);
                    let compact=local_move(s,i,j,k,l,&nests);
                    assert_eq!(result.is_some(),compact.is_some());
                    if let Some(next)=compact {assert_eq!(next,local_state(&full,&region));}
                    samples+=1;
                }
            }}
        }
        assert!(samples>10000);
        put(&mut board,0,&[0;8]);
        assert_ne!(pack_cell(&board,0),0);
    }
    #[test]
    fn differential_validation_matches_full_replay() {
        let (grid,mut initial)=fixture(5); put(&mut initial,0,&[0]); put(&mut initial,20,&[1;8]);
        let mut board=initial; let mut ops=long_route();
        for &op in &ops {apply_op(&mut board,op,&grid).unwrap();}
        let mut rng=9988776655u64;
        for _ in 0..45 {
            let mut moves=Vec::new(); generate_valid_moves_into(&board,None,&grid,&mut moves);
            if moves.is_empty() {break;}
            rng^=rng<<13; rng^=rng>>7; rng^=rng<<17;
            let op=moves[(rng as usize)%moves.len()]; apply_op(&mut board,op,&grid).unwrap(); ops.push(op);
        }
        let trace=TraceCache::build(&initial,&ops,&grid,budget()).unwrap();
        let replacements=vec![vec![],vec![Op{r:0,c:0,k:0,d:3,l:1},Op{r:0,c:1,k:0,d:3,l:1}],
            vec![Op{r:0,c:0,k:7,d:3,l:1}]];
        let mut checks=0;
        for a in 0..ops.len() {for b in a+1..=(a+8).min(ops.len()) {
            let unchanged=ops[a..b].to_vec();
            for patch in replacements.iter().chain(std::iter::once(&unchanged)) {
                for target in [0,1,grid.initial_slimes+9] {
                    let mut full=ops.clone();full.splice(a..b,patch.iter().copied());
                    let expected=simulate_all(initial,&full,&grid).is_some_and(|c|c>=target);
                    assert_eq!(validate_patch(&ops,&trace,a,b,patch,&grid,target,budget())==CheckResult::Accept,expected);
                    checks+=1;
                }
            }
        }}
        assert!(checks>1000);
        let original=ops.clone();
        assert_eq!(validate_patch(&ops,&trace,0,4,&[],&grid,1,Instant::now()),CheckResult::BudgetExpired);
        assert_eq!(ops,original);
        assert_eq!(complete_2hop_jumps(ops.clone(),&initial,&grid,1,Instant::now()),ops);
        assert_eq!(compact_optimize(ops.clone(),&initial,&grid,Instant::now(),true),ops);
    }
    #[test]
    fn empty_straight_bridge_and_exact_compact_search() {
        let (grid,mut initial)=fixture(5); put(&mut initial,0,&[0]);
        let ops=long_route();
        let short=complete_2hop_jumps(ops.clone(),&initial,&grid,1,budget());
        assert_eq!(short.len(),2); assert_eq!(simulate_all(initial,&short,&grid),Some(1));
        let mut goal=initial;for &op in &ops {apply_op(&mut goal,op,&grid).unwrap();}
        let region=[0,5,6,7,2,1];
        let compact=compact_window(&initial,&goal,&region,4,&grid,budget()).unwrap();
        assert_eq!(compact.len(),2);
        let mut end=initial;for op in compact {apply_op(&mut end,op,&grid).unwrap();} assert!(end==goal);
    }
    #[test]
    fn dependency_closure_and_preserved_independent_move() {
        let pairs=[(0,1),(4,5),(1,4),(2,3)];
        let (r,chosen)=closure_region(&pairs,0,4);
        assert!(r.contains(&5));assert_eq!(chosen,vec![0,1,2]);
        let (grid,mut initial)=fixture(5);put(&mut initial,0,&[0]);put(&mut initial,20,&[1]);
        let mut ops=long_route();ops.insert(1,Op{r:4,c:0,k:0,d:3,l:1});
        let trace=TraceCache::build(&initial,&ops,&grid,budget()).unwrap();
        let (mut r,chosen)=closure_region(&trace.endpoints,0,5);
        assert_eq!(chosen.len(),4); assert!(!r.contains(&20));
        add_auxiliary(&mut r,&initial,&trace.endpoints,&grid);
        let mut patch=compact_window(&initial,&trace.boards[5],&r,4,&grid,budget()).unwrap();
        patch.push(ops[1]);
        let mut end=initial;for op in patch {apply_op(&mut end,op,&grid).unwrap();}
        assert!(end==trace.boards[5]);
    }
    #[test]
    fn external_tower_can_be_overflown_but_not_landed_on() {
        let (grid,mut initial)=fixture(5);put(&mut initial,0,&[1,0,1]);put(&mut initial,1,&[1;8]);
        let region=[0,2];let mut nests=[None;12];nests[1]=Some(0);
        let s=local_state(&initial,&region);
        let next=local_move(s,0,1,1,2,&nests).unwrap();
        let mut full=initial;assert!(apply_op(&mut full,Op{r:0,c:0,k:1,d:3,l:2},&grid).is_some());
        assert_eq!(next,local_state(&full,&region));assert_eq!(full.lens[1],8);
        let goal=full;
        let answer=compact_window(&initial,&goal,&region,3,&grid,budget()).unwrap();
        assert!(answer.iter().all(|op|region.contains(&endpoint(*op,&grid).1)));
    }
}
