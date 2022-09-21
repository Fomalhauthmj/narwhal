use std::{collections::BTreeSet, num::NonZeroUsize};

use consensus::consensus::Dag;
use criterion::{criterion_group, criterion_main, Criterion};
use fastcrypto::Hash;
use pprof::criterion::{Output, PProfProfiler};
use rayon::prelude::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use test_utils::CommitteeFixture;
use types::{Certificate, CertificateDigest};
struct TestEnvironment {
    committee: CommitteeFixture,
    last_round: u64,
    dag: Dag,
}
impl TestEnvironment {
    pub fn new(committee_size: usize) -> Self {
        let committee = CommitteeFixture::builder()
            .committee_size(NonZeroUsize::new(committee_size).unwrap())
            .build();
        let mut dag = Dag::new();
        let genesis = Certificate::genesis(&committee.committee())
            .into_par_iter()
            .map(|x| (x.origin(), (x.digest(), x)))
            .collect();
        dag.insert(0, genesis);
        Self {
            committee,
            last_round: 0,
            dag,
        }
    }
    pub fn generate_round(&mut self) {
        let parents: BTreeSet<CertificateDigest> = self
            .dag
            .get(&self.last_round)
            .unwrap()
            .values()
            .map(|(d, _)| *d)
            .collect();
        let (r, headers) = self.committee.headers_round(self.last_round, &parents);
        let vertexs = headers
            .par_iter()
            .map(|h| {
                let cert = self.committee.certificate(h);
                (cert.origin(), (cert.digest(), cert))
            })
            .collect();
        self.dag.insert(r, vertexs);
        self.last_round = r;
    }
    pub fn random_leader_vertex(&self, round: u64) -> &Certificate {
        let mut candidates = self.dag.get(&round).unwrap().values();
        let len = candidates.len();
        let (_, leader_vertex) = candidates.nth(rand::random::<usize>() % len).unwrap();
        leader_vertex
    }
}
pub fn linked_bench(c: &mut Criterion) {
    let mut linked_group = c.benchmark_group("linked");
    linked_group.sample_size(10);
    // f = [1, 3, 100, 1000]
    static COMMITTEE_SIZES: [usize; 4] = [4, 10, 301, 3001];
    static ROUND_GAPS: [usize; 3] = [2, 4, 6];
    for committee_size in &COMMITTEE_SIZES {
        let mut te = TestEnvironment::new(*committee_size);
        for round_gap in &ROUND_GAPS {
            while te.last_round < (1 + round_gap) as u64 {
                te.generate_round();
            }
            let prev_leader = te.random_leader_vertex(1);
            let leader = te.random_leader_vertex((1 + round_gap) as u64);
            linked_group.bench_function(
                format!("Committee size:{} Round gap:{}", committee_size, round_gap),
                |b| {
                    b.iter(|| {
                        linked(leader, prev_leader, &te.dag);
                    })
                },
            );
        }
    }
}
/// Checks if there is a path between two leaders.
fn linked(leader: &Certificate, prev_leader: &Certificate, dag: &Dag) -> bool {
    let mut parents = vec![leader];
    for r in (prev_leader.round()..leader.round()).rev() {
        parents = dag
            .get(&(r))
            .expect("We should have the whole history by now")
            .values()
            .filter(|(digest, _)| parents.iter().any(|x| x.header.parents.contains(digest)))
            .map(|(_, certificate)| certificate)
            .collect();
    }
    parents.contains(&prev_leader)
}
criterion_group! {
    name = linked_group;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = linked_bench
}
criterion_main!(linked_group);
