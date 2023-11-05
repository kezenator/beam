use std::collections::HashMap;

use float_ord::FloatOrd;
use itertools::Itertools;

use crate::geom::{Aabb, AabbBoundedSurface, BoundingSurface, Surface};
use crate::intersection::SurfaceIntersection;
use crate::math::Scalar;
use crate::ray::{Ray, RayRange};
use crate::vec::Point3;

#[derive(Clone)]
pub struct Octree<S: AabbBoundedSurface + Clone + 'static>
{
    bounds: Aabb,
    items: Vec<S>,
    tree: OctreeNode,
}

impl<S: AabbBoundedSurface + Clone + 'static> Octree<S>
{
    pub fn new(items: Vec<S>, target_leaf_size: usize) -> Self
    {
        let Split{ bounds, sub_tree } = build_complete_octree(&items, target_leaf_size);

        let result = Octree
        {
            bounds,
            items,
            tree: sub_tree
        };

        let stats = result.get_stats();
        println!("Created octree: {} items, {} nodes, {} total-item-refs, {} depth, ({}..{}) leaf-size, {} target-leaf-size",
            result.items.len(), stats.num_nodes, stats.total_item_refs, stats.max_depth, stats.smallest_node, stats.largest_node, target_leaf_size);

        result
    }

    pub fn get_stats(&self) -> OctreeStats
    {
        let mut result = OctreeStats::new();
        self.tree.add_stats(&mut result, 1);
        result
    }
}

impl<S: AabbBoundedSurface + Clone + 'static> Surface for Octree<S>
{
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
    {
        if self.bounds.may_intersect_in_range(ray, range)
        {
            let mut range = range.clone();
            let mut closest = None;

            self.tree.closest_intersection_in_range(&self.items, ray, &mut range, &mut closest);

            closest
        }
        else
        {
            None
        }
    }
}

impl<S: AabbBoundedSurface + Clone + 'static> AabbBoundedSurface for Octree<S>
{
    fn get_bounding_aabb(&self) -> Aabb
    {
        self.bounds.clone()
    }
}

#[derive(Clone)]
enum OctreeNode
{
    Leaf(Vec<usize>),
    Split(Vec<Split>),
}

impl OctreeNode
{
    fn add_stats(&self, stats: &mut OctreeStats, depth: usize)
    {
        match self
        {
            OctreeNode::Leaf(indexes) =>
            {
                stats.add_node(depth, Some(indexes.len()));
            },
            OctreeNode::Split(splits) =>
            {
                stats.add_node(depth, None);

                for split in splits.iter()
                {
                    split.sub_tree.add_stats(stats, depth + 1);
                }
            },
        }
    }

    fn closest_intersection_in_range<'r, S:AabbBoundedSurface + Clone + 'static>(&self, items: &Vec<S>, ray: &'r Ray, range: &mut RayRange, closest: &mut Option<SurfaceIntersection<'r>>)
    {
        match self
        {
            OctreeNode::Leaf(indexes) =>
            {
                for i in indexes.iter()
                {
                    if let Some(intersection) = items[*i].closest_intersection_in_range(ray, &range)
                    {
                        range.set_max(intersection.distance);
                        *closest = Some(intersection);
                    }
                }
            },
            OctreeNode::Split(splits) =>
            {
                for split in splits.iter()
                {
                    if split.bounds.may_intersect_in_range(ray, range)
                    {
                        split.sub_tree.closest_intersection_in_range(items, ray, range, closest);
                    }
                }
            },
        }
    }
}

#[derive(Clone)]
struct Split
{
    bounds: Aabb,
    sub_tree: OctreeNode,
}

pub struct OctreeStats
{
    num_nodes: usize,
    max_depth: usize,
    smallest_node: usize,
    largest_node: usize,
    total_item_refs: usize,
}

impl OctreeStats
{
    fn new() -> Self
    {
        OctreeStats
        {
            num_nodes: 0,
            max_depth: 0,
            smallest_node: usize::max_value(),
            largest_node: 0,
            total_item_refs: 0,
        }
    }

    fn add_node(&mut self, depth: usize, size: Option<usize>)
    {
        self.num_nodes += 1;
        self.max_depth = self.max_depth.max(depth);
        if let Some(size) = size
        {
            self.smallest_node = self.smallest_node.min(size);
            self.largest_node = self.largest_node.max(size);
            self.total_item_refs += size;
        }
    }
}

struct BuildInfo<'a, S: AabbBoundedSurface + Clone + 'static>
{
    items: &'a Vec<S>,
    item_bounds: Vec<Aabb>,
    target_leaf_size: usize,
}

impl<'a, S: AabbBoundedSurface + Clone + 'static> BuildInfo<'a, S>
{
    fn new(items: &'a Vec<S>, target_leaf_size: usize) -> Self
    {
        let item_bounds = items.iter().map(|i| i.get_bounding_aabb()).collect();

        BuildInfo { items, item_bounds, target_leaf_size }
    }
}

struct CurSplitState
{
    bounds: Aabb,
    included_indexes: Vec<usize>,
    name: String,
}

impl CurSplitState
{
    fn new_root<'a, S: AabbBoundedSurface + Clone + 'static>(info: &'a BuildInfo<'a, S>) -> Self
    {
        assert!(!info.items.is_empty());

        let mut bounds = info.items[0].get_bounding_aabb();

        for i in info.items.iter()
        {
            bounds = bounds.union(&i.get_bounding_aabb());
        }

        let included_indexes = (0..info.items.len()).collect();

        let name = "root".to_owned();

        CurSplitState { bounds, included_indexes, name }
    }

    fn new_sub(&self, dim_name: &'static str, split_index: usize, bounds: Aabb, included_indexes: Vec<usize>) -> Self
    {
        let name = format!("{}.{}{}", self.name, dim_name, split_index);
        CurSplitState { bounds, included_indexes, name }
    }
}

fn build_complete_octree<S: AabbBoundedSurface + Clone + 'static>(items: &Vec<S>, target_leaf_size: usize) -> Split
{
    let info = BuildInfo::new(items, target_leaf_size);
    let root_split = CurSplitState::new_root(&info);

    let root_bounds = root_split.bounds.clone();
    let root_tree = create_node(&info, root_split);

    Split
    {
        bounds: root_bounds,
        sub_tree: root_tree,
    }
}

fn create_node<'a, S>(info: &BuildInfo<'a, S>, cur_split: CurSplitState) -> OctreeNode
    where S: AabbBoundedSurface + Clone + 'static
{
    if cur_split.included_indexes.len() > info.target_leaf_size
    {
        let split_x = create_split_dim(info, &cur_split, "x", |v| v.x, |v, s| Point3::new(s, v.y, v.z));
        let split_y = create_split_dim(info, &cur_split, "y", |v| v.y, |v, s| Point3::new(v.x, s, v.z));
        let split_z = create_split_dim(info, &cur_split, "z", |v| v.z, |v, s| Point3::new(v.x, v.y, s));

        //println!("CreateSplit: {} Summary x {:5}/{:5}/{:5} y {:5}/{:5}/{:5} z {:5}/{:5}/{:5}",
        //    cur_split.name,
        //    split_x.items_0.len(), split_x.items_1.len(), split_x.best_sum,
        //    split_y.items_0.len(), split_y.items_1.len(), split_y.best_sum,
        //    split_z.items_0.len(), split_z.items_1.len(), split_z.best_sum);

        let best_split = if (split_x.best_sum <= split_y.best_sum) && (split_x.best_sum <= split_z.best_sum)
        {
            split_x
        }
        else if (split_y.best_sum <= split_x.best_sum) && (split_y.best_sum <= split_z.best_sum)
        {
            split_y
        }
        else
        {
            split_z
        };

        if best_split.made_progress
        {
            //println!("CreateSplit: {} Result: Split {}", cur_split.name, best_split.dim_name);

            let sub_split_0 = cur_split.new_sub(best_split.dim_name, 0, best_split.bounds_0, best_split.items_0);
            let sub_split_1 = cur_split.new_sub(best_split.dim_name, 1, best_split.bounds_1, best_split.items_1);

            let bounds_0 = sub_split_0.bounds.clone();
            let bounds_1 = sub_split_1.bounds.clone();

            let node_0 = create_node(info, sub_split_0);
            let node_1 = create_node(info, sub_split_1);

            return OctreeNode::Split(vec![
                Split { bounds: bounds_0, sub_tree: node_0 },
                Split { bounds: bounds_1, sub_tree: node_1 },
            ]);
        }
    }

    // Either:
    // 1. The number of items is already below the target leaf size,
    // 2. No good progress could be made
    // Just return a leaf containing everything

    //println!("CreateSplit: {} Result: Leaf", cur_split.name);

    OctreeNode::Leaf(cur_split.included_indexes)
}

struct SplitOption
{
    made_progress: bool,
    best_sum: usize,
    dim_name: &'static str,
    bounds_0: Aabb,
    items_0: Vec<usize>,
    bounds_1: Aabb,
    items_1: Vec<usize>,
}

#[derive(Default)]
struct SplitDimPointInfo
{
    num_enter: usize,
    num_leave: usize,
}

fn create_split_dim<'a, S, Extract, Combine>(info: &BuildInfo<'a, S>, cur_split: &CurSplitState, dim_name: &'static str, extract: Extract, combine: Combine) -> SplitOption
    where S: AabbBoundedSurface + Clone + 'static,
        Extract: Fn(&Point3) -> Scalar + 'static,
        Combine: Fn(&Point3, Scalar) -> Point3 + 'static
{
    let cur_split_min = extract(&cur_split.bounds.min);
    let cur_split_max = extract(&cur_split.bounds.max);
    let cur_split_num = cur_split.included_indexes.len();

    // Calculate the extreme points for each item in the current split.
    // When we put these in order:
    // Items with the point as the bounds min will ENTER the set of contained objects as we get to that point.
    // Items with the point as the bounds max will LEAVE the set of contained objects as we get to that point.

    let mut point_info = HashMap::<FloatOrd<Scalar>, SplitDimPointInfo>::new();

    for i in cur_split.included_indexes.iter()
    {
        let i_bounds = &info.item_bounds[*i];
        let min = extract(&i_bounds.min);
        let max = extract(&i_bounds.max);

        point_info.entry(FloatOrd(min)).or_default().num_enter += 1;
        point_info.entry(FloatOrd(max)).or_default().num_leave += 1;
    }

    // Sort the bounds points into order

    let mut points = point_info.keys().map(|fo| fo.0).collect_vec();
    float_ord::sort(points.as_mut_slice());

    //println!("CreateSplitDim: {}.{}: {} items, {} unique bounds, [{:8.3}..{:8.3}] (size {:8.3})",
    //    cur_split.name, dim_name, cur_split.included_indexes.len(),
    //    points.len(), points[0], points[points.len() - 1], points[points.len() - 1] - points[0]);

    // Now - step through each item bounds point in order,
    // 1. Collect the total number of items if we split at that point:
    //    i.e. For each point, the number of items ENTERING are added to the after set,
    //         and the number of items LEAVING are removed from the before set.
    // 2. Work-out the best place to split - based on a huristic
    //    Currently: Sum = Diff + Extra is lowest, where:
    //               Diff = difference between number before split and number after split
    //                      i.e. 0 when we exactly cut the number of objects in two.
    //               Extra = number of objects duplicated into both before and after the split

    let mut count_before = cur_split_num;
    let mut count_after = 0;
    let mut best_p = points[0];
    let mut best_sum = usize::MAX;

    for p in points.iter()
    {
        let p_info = point_info.get(&FloatOrd(*p)).unwrap();

        count_after += p_info.num_enter;
        assert!(count_after <= cur_split_num);

        let diff = count_before.max(count_after) - count_before.min(count_after);
        assert!(diff <= cur_split_num);

        let extra = count_before + count_after - cur_split_num;
        assert!(extra <= cur_split_num);

        let sum = diff + extra;

        let new_best = sum < best_sum;
        if new_best
        {
            best_p = *p;
            best_sum = sum;
        }

        //println!("   {:8.3} => {:5} enter, {:5} leave, {:5} min-size, {:5} max-side, {:5} diff, {:5} extra, {:5} sum, {}", p, p_info.num_enter, p_info.num_leave, count_before, count_after, diff, extra, sum, new_best);

        count_before -= p_info.num_leave;
        assert!(count_before <= cur_split_num);
    }

    // Ensure our accounting has worked our correctly

    assert!(count_before == 0);
    assert!(count_after == cur_split_num);

    // Now - lets check if we think the split is worthwhile

    if (best_p > cur_split_min)
        && (best_p < cur_split_max)
        && (best_sum < (cur_split_num / 2))
    {
        // The split meets the following criteria:
        // 1. It's actually a split within the current bounds
        // 2. It makes some progress

        let bounds_0 = Aabb::new(
            cur_split.bounds.min,
            combine(&cur_split.bounds.max, best_p));

        let bounds_1 = Aabb::new(
                combine(&cur_split.bounds.min, best_p),
                cur_split.bounds.max);

        let items_0 = cur_split.included_indexes.iter()
            .cloned()
            .filter(|i| bounds_0.intersects(&info.item_bounds[*i]))
            .collect_vec();
    
        let items_1 = cur_split.included_indexes.iter()
            .cloned()
            .filter(|i| bounds_1.intersects(&info.item_bounds[*i]))
            .collect_vec();

        // Double check we have the same results

        let num_0 = items_0.len();
        let num_1 = items_1.len();
        let diff = num_0.max(num_1) - num_0.min(num_1);
        let extra = num_0 + num_1 - cur_split_num;
        let final_sum = diff + extra;
        assert!(final_sum == best_sum);
    
        return SplitOption
        {
            made_progress: true,
            best_sum,
            dim_name,
            bounds_0,
            items_0,
            bounds_1,
            items_1,
        };
    }

    // The split doens't make good progress

    return SplitOption
    {
        made_progress: false,
        best_sum: usize::MAX,
        dim_name,
        bounds_0: cur_split.bounds.clone(),
        items_0: cur_split.included_indexes.clone(),
        bounds_1: cur_split.bounds.clone(),
        items_1: Vec::new(),
    };
}
