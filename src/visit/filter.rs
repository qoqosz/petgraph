
use prelude::*;

use fixedbitset::FixedBitSet;
use std::collections::HashSet;

use visit::{
    GraphBase,
    IntoNeighbors,
    IntoNodeIdentifiers,
    IntoNeighborsDirected,
    NodeIndexable,
    Visitable,
    VisitMap,
    GraphProp,
};

/// A graph filter for nodes.
pub trait FilterNode<N>
{
    fn include_node(&self, node: N) -> bool { let _ = node; true }
}

impl<F, N> FilterNode<N> for F
    where F: Fn(N) -> bool,
{
    fn include_node(&self, n: N) -> bool {
        (*self)(n)
    }
}

/// This filter includes the nodes that are contained in the set.
impl<N> FilterNode<N> for FixedBitSet
    where FixedBitSet: VisitMap<N>,
{
    fn include_node(&self, n: N) -> bool {
        self.is_visited(&n)
    }
}

/// This filter includes the nodes that are contained in the set.
impl<N, S> FilterNode<N> for HashSet<N, S>
    where HashSet<N, S>: VisitMap<N>,
{
    fn include_node(&self, n: N) -> bool {
        self.is_visited(&n)
    }
}

/// A filtered adaptor of a graph.
#[derive(Copy, Clone, Debug)]
pub struct Filtered<G, F>(pub G, pub F);

impl<G, F> GraphBase for Filtered<G, F> where G: GraphBase {
    type NodeId = G::NodeId;
    type EdgeId = G::EdgeId;
}

impl<'a, G, F> IntoNeighbors for &'a Filtered<G, F>
    where G: IntoNeighbors,
          F: FilterNode<G::NodeId>,
{
    type Neighbors = FilteredNeighbors<'a, G::Neighbors, F>;
    fn neighbors(self, n: G::NodeId) -> Self::Neighbors {
        FilteredNeighbors {
            include_source: self.1.include_node(n),
            iter: self.0.neighbors(n),
            f: &self.1,
        }
    }
}

/// A filtered neighbors iterator.
pub struct FilteredNeighbors<'a, I, F: 'a>
{
    include_source: bool,
    iter: I,
    f: &'a F,
}

impl<'a, I, F> Iterator for FilteredNeighbors<'a, I, F>
    where I: Iterator,
          I::Item: Copy,
          F: FilterNode<I::Item>,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let f = self.f;
        if !self.include_source {
            None
        } else {
            (&mut self.iter).filter(move |&target| f.include_node(target)).next()
        }
    }
}

impl<'a, G, F> IntoNeighborsDirected for &'a Filtered<G, F>
    where G: IntoNeighborsDirected,
          F: FilterNode<G::NodeId>,
{
    type NeighborsDirected = FilteredNeighbors<'a, G::NeighborsDirected, F>;
    fn neighbors_directed(self, n: G::NodeId, dir: Direction)
        -> Self::NeighborsDirected {
        FilteredNeighbors {
            include_source: self.1.include_node(n),
            iter: self.0.neighbors_directed(n, dir),
            f: &self.1,
        }
    }
}

impl<'a, G, F> IntoNodeIdentifiers for &'a Filtered<G, F>
    where G: IntoNodeIdentifiers,
          F: FilterNode<G::NodeId>,
{
    type NodeIdentifiers = FilteredNeighbors<'a, G::NodeIdentifiers, F>;
    fn node_identifiers(self) -> Self::NodeIdentifiers {
        FilteredNeighbors {
            include_source: true,
            iter: self.0.node_identifiers(),
            f: &self.1,
        }
    }
}

macro_rules! access0 {
    ($e:expr) => ($e.0)
}

NodeIndexable!{delegate_impl [[G, F], G, Filtered<G, F>, access0]}
GraphProp!{delegate_impl [[G, F], G, Filtered<G, F>, access0]}
Visitable!{delegate_impl [[G, F], G, Filtered<G, F>, access0]}
